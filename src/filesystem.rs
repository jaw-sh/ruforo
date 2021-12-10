use crate::ffmpeg::get_extension_ffmpeg;
use crate::s3::{get_extension, S3Bucket};
use actix_multipart::Multipart;
use actix_web::{post, web, Error, HttpResponse, Responder};
use futures::{StreamExt, TryStreamExt};
use mime::Mime;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

struct UploadPayload {
    data: Vec<u8>,
    filename: String,
    tmp_path: PathBuf,
    hash: blake3::Hash,
    mime: Mime,
}

#[post("/fs/upload-file")]
pub async fn put_file(
    mut mutipart: Multipart,
    s3: web::Data<S3Bucket>,
) -> Result<impl Responder, Error> {
    // see: https://users.rust-lang.org/t/file-upload-in-actix-web/64871/3
    let mut payloads: Vec<UploadPayload> = Vec::new(); // TODO can we count files from the multipart to reserve?

    // iterate over multipart stream
    while let Ok(Some(mut field)) = mutipart.try_next().await {
        let content_type = field.content_disposition();
        let filename = content_type
            .get_filename()
            .ok_or_else(|| {
                actix_web::error::ErrorInternalServerError("put_file: missing filename")
            })?
            .to_owned();

        let f = web::block(move || {
            let mut filepath;
            let mut uuid;
            loop {
                uuid = format!("{}/{}", crate::DIR_TMP.as_str(), Uuid::new_v4());
                filepath = Path::new(&uuid);
                match filepath.metadata() {
                    Ok(metadata) => {
                        log::error!(
                            "put_file: file already exists: {:#?}\n{:#?}",
                            filepath,
                            metadata
                        );
                    }
                    Err(e) => {
                        match e.kind() {
                            std::io::ErrorKind::NotFound => {
                                break;
                            }
                            std::io::ErrorKind::PermissionDenied => {
                                log::error!("put_file tmp permission error: {}", e);
                                return Err(e);
                            }
                            _ => {
                                log::error!("put_file unhandled fs error: {}", e);
                                return Err(e);
                            }
                        };
                    }
                }
            }
            log::info!(
                "put_file: creating tmp file: {}",
                filepath.to_str().unwrap()
            );
            Ok((File::create(filepath), filepath.to_owned()))
        });

        let mut hasher = blake3::Hasher::new();
        let mut buf: Vec<u8> = Vec::with_capacity(1024); // TODO can we estimate a real size from the multipart?

        let (f, filepath) = f
            .await
            .map_err(|e| {
                log::error!("put_file: {}", e);
                actix_web::error::ErrorInternalServerError("put_file: saving data")
            })?
            .map_err(|e| {
                log::error!("put_file: {}", e);
                actix_web::error::ErrorInternalServerError("put_file: saving data")
            })?;

        let mut f = f.map_err(|e| {
            log::error!("put_file: {}", e);
            actix_web::error::ErrorInternalServerError("put_file: saving data")
        })?;

        while let Some(chunk) = field.next().await {
            let bytes = chunk.map_err(|e| {
                log::error!("put_file: multipart read error: {}", e);
                actix_web::error::ErrorInternalServerError("put_file: error reading upload data")
            })?;
            hasher.update(&bytes);
            buf.extend(bytes.to_owned());
            f = web::block(move || f.write_all(&bytes.clone()).map(|_| f))
                .await
                .unwrap()?;
        }

        let parsed = UploadPayload {
            data: buf,
            filename,
            tmp_path: filepath, // WARNING we delete tmp_path at the end, don't screw up
            hash: hasher.finalize(),
            mime: field.content_type().to_owned(),
        };

        payloads.push(parsed);
    }

    for payload in payloads {
        log::info!("Filename: {}", payload.filename);
        log::info!("BLAKE3: {}", payload.hash);
        log::info!("MIME: {}", payload.mime);

        let extension = match get_extension_ffmpeg(&payload.tmp_path).await {
            Some(v) => Some(v),
            None => get_extension(&payload.filename, &payload.mime),
        };

        let s3_filename = match extension {
            Some(v) => format!("{}.{}", payload.hash, v),
            None => payload.hash.to_string(),
        };

        // TODO probably check DB instead of the S3 bucket, or both
        let list = s3.list_objects_v2(&s3_filename).await.map_err(|e| {
            log::error!("put_file: failed to list_objects_v2: {}", e);
            actix_web::error::ErrorInternalServerError("put_file: failed to check if file exists")
        })?;

        // TODO check and insert DB entry here

        let count = list.key_count.ok_or_else(|| {
            log::error!("put_file: key_count, I don't think this should ever happen");
            actix_web::error::ErrorInternalServerError("put_file: failed to check if file exists")
        })?;

        // count should only ever be 0 or 1, otherwise there's something wrong with the prefix
        if count == 0 {
            s3.put_object(payload.data, &s3_filename)
                .await
                .map_err(|e| {
                    log::error!("put_file: failed to put_object: {}", e);
                    actix_web::error::ErrorInternalServerError("put_file: failed to store file")
                })?;
        } else {
            log::info!("put_file: duplicate upload, skipping S3 put_object");
        }

        // WARNING we delete a file, be mindful and don't fucking delete my porn folder
        log::warn!("Deleting Tmp File: {:#?}", payload.tmp_path);
        std::fs::remove_file(payload.tmp_path).map_err(|e| {
            log::error!("put_file: delete tmp file error: {}", e);
            actix_web::error::ErrorInternalServerError("put_file: failed to store file")
        })?;
    }

    Ok(HttpResponse::Ok()
        .content_type("text/css")
        .body("put_file: ok"))
}
