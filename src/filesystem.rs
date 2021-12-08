use crate::s3::S3Bucket;
use actix_multipart::Multipart;
use actix_web::{post, web, Error, HttpResponse, Responder};
use futures::{StreamExt, TryStreamExt};

struct UploadPayload {
    data: Vec<u8>,
    filename: String,
    hash: blake3::Hash,
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

        let mut hasher = blake3::Hasher::new();
        let mut my_vec: Vec<u8> = Vec::with_capacity(1024); // TODO can we estimate a real size from the multipart?
        while let Some(chunk) = field.next().await {
            let my_bytes = chunk.map_err(|e| {
                log::error!("put_file: multipart read error: {}", e);
                actix_web::error::ErrorInternalServerError("put_file: error reading upload data")
            })?;
            hasher.update(&my_bytes);
            my_vec.extend(my_bytes);
        }

        let parsed = UploadPayload {
            data: my_vec,
            filename,
            hash: hasher.finalize(),
        };

        payloads.push(parsed);
    }

    for payload in payloads {
        log::error!("Filename: {}", payload.filename);
        log::error!("Content: {}", std::str::from_utf8(&payload.data).unwrap());
        log::error!("BLAKE3: {}", payload.hash);
        s3.put_object(payload.data, &payload.filename)
            .await
            .map_err(|e| {
                log::error!("put_file: failed to put_object: {}", e);
                actix_web::error::ErrorInternalServerError("put_file: failed to store file")
            })?;
    }

    Ok(HttpResponse::Ok()
        .content_type("text/css")
        .body("put_file: ok"))
}
