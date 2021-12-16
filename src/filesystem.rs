use crate::ffmpeg::get_extension_ffmpeg;
use crate::init::get_db_pool;
use crate::orm::{attachments, ugc, ugc_attachments};
use crate::s3::S3Bucket;
use actix_multipart::Multipart;
use actix_web::{get, http::header::ContentType, post, web, Error, HttpResponse, Responder};
use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use mime::Mime;
use once_cell::sync::OnceCell;
use sea_orm::{
    entity::*, query::*, sea_query::Expr, DbErr, FromQueryResult, JsonValue, QueryFilter,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

static MIME_LOOKUP: OnceCell<HashMap<&'static str, &'static str>> = OnceCell::new();
static EXT_LOOKUP: OnceCell<HashMap<&'static str, &'static str>> = OnceCell::new();
static DIR_TMP: OnceCell<String> = OnceCell::new();
static S3BUCKET: OnceCell<S3Bucket> = OnceCell::new();

#[inline(always)]
fn get_mime_lookup() -> &'static HashMap<&'static str, &'static str> {
    unsafe { MIME_LOOKUP.get_unchecked() }
}
#[inline(always)]
fn get_ext_lookup() -> &'static HashMap<&'static str, &'static str> {
    unsafe { EXT_LOOKUP.get_unchecked() }
}
#[inline(always)]
fn get_dir_tmp() -> &'static str {
    unsafe { DIR_TMP.get_unchecked() }
}
#[inline(always)]
fn get_s3() -> &'static S3Bucket {
    unsafe { S3BUCKET.get_unchecked() }
}

/// MUST be called ONCE before using functions in this module
pub fn init() {
    DIR_TMP
        .set(
            std::env::var("DIR_TMP")
                .expect("missing DIR_TMP environment variable (hint: 'DIR_TMP=./tmp')"),
        )
        .unwrap();

    if S3BUCKET
        .set(S3Bucket::new(
            rusoto_core::Region::Custom {
                name: "localhost".to_owned(),
                endpoint: "http://localhost:9000".to_owned(),
            },
            "test0".to_owned(),
            "localhost:9000/test0".to_owned(),
        ))
        .is_err()
    {
        panic!("S3BUCKET");
    }

    let map: HashMap<&'static str, &'static str> = HashMap::from([
        ("aac", "aac"),
        ("apng", "apng"),
        ("avi", "avi"),
        ("avif", "avif"),
        ("bmp", "bmp"),
        ("djvu", "djvu"),
        ("flac", "flac"),
        ("gif", "gif"),
        ("htm", "html"),
        ("html", "html"),
        ("ico", "ico"),
        ("jpeg", "jpeg"),
        ("jpg", "jpeg"),
        ("json", "json"),
        ("ktx", "ktx"),
        ("m4a", "m4a"),
        ("mka", "mka"),
        ("mkv", "mkv"),
        ("mov", "mov"),
        ("mp3", "mp3"),
        ("mp4", "mp4"),
        ("ogg", "ogg"),
        ("ogv", "ogv"),
        ("pdf", "pdf"),
        ("png", "png"),
        ("rm", "rm"),
        ("sh", "sh"),
        ("svg", "svg"),
        ("txt", "txt"),
        ("weba", "weba"),
        ("webm", "webm"),
        ("webp", "webp"),
        ("xml", "xml"),
        ("zip", "zip"),
    ]);
    EXT_LOOKUP.set(map).unwrap();

    let map: HashMap<&'static str, &'static str> = HashMap::from([
        ("application/json", "json"),
        ("application/pdf", "pdf"),
        ("application/vnd.rn-realmedia", "rm"),
        ("application/x-sh", "sh"),
        ("application/zip", "zip"),
        ("audio/aac", "aac"),
        ("audio/flac", "flac"),
        ("audio/m4a", "m4a"),
        ("audio/mp4", "mp4"),
        ("audio/mpeg", "mp3"),
        ("audio/ogg", "ogg"),
        ("audio/webm", "weba"),
        ("audio/x-matroska", "mka"),
        ("image/apng", "apng"),
        ("image/avif", "avif"),
        ("image/bmp", "bmp"),
        ("image/gif", "gif"),
        ("image/jpeg", "jpeg"),
        ("image/ktx", "ktx"),
        ("image/png", "png"),
        ("image/svg+xml", "svg"),
        ("image/vnd.djvu", "djvu"),
        ("image/webp", "webp"),
        ("image/x-icon", "ico"),
        ("text/html", "html"),
        ("text/plain", "txt"),
        ("text/xml", "xml"),
        ("video/mp4", "mp4"),
        ("video/ogg", "ogv"),
        ("video/quicktime", "mov"),
        ("video/webm", "webm"),
        ("video/x-matroska", "mkv"),
        ("video/x-msvideo", "avi"),
    ]);
    MIME_LOOKUP.set(map).unwrap();
}

struct UploadPayload {
    data: Vec<u8>,
    filename: String,
    tmp_path: PathBuf,
    hash: blake3::Hash,
    mime: Mime,
}

#[get("/fs/{file_id}")]
pub async fn view_file_canonical(file_id: web::Path<i32>) -> Result<impl Responder, Error> {
    let result = get_file_url(get_s3(), *file_id).await.map_err(|e| {
        log::error!("view_file: get_filename_by_id: {}", e);
        actix_web::error::ErrorInternalServerError("view_file: bad ID")
    })?;
    let content = match result {
        Some(result) => result,
        None => "None".to_owned(),
    };
    let body = format!(
        "<html><body><div>{:?} - {}</div><div><img src=\"{}\"></div></body></html>",
        file_id, content, content
    );
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}

#[get("/fs/ugc/{file_id}")]
pub async fn view_file_ugc(file_id: web::Path<i32>) -> Result<impl Responder, Error> {
    let result = get_file_url_by_ugc(get_s3(), *file_id).await.map_err(|e| {
        log::error!("view_file: get_filename_by_id: {}", e);
        actix_web::error::ErrorInternalServerError("view_file: bad ID")
    })?;
    let content = match result {
        Some(result) => result,
        None => "None".to_owned(),
    };
    let body = format!(
        "<html><body><div>{:?} - {}</div><div><img src=\"{}\"></div></body></html>",
        file_id, content, content
    );
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(body))
}

#[post("/fs/upload-file")]
pub async fn put_file(mut mutipart: Multipart) -> Result<impl Responder, Error> {
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
                uuid = format!("{}/{}", get_dir_tmp(), Uuid::new_v4());
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
        let bucket = get_s3();
        let list = bucket.list_objects_v2(&s3_filename).await.map_err(|e| {
            log::error!("put_file: failed to list_objects_v2: {}", e);
            actix_web::error::ErrorInternalServerError("put_file: failed to check if file exists")
        })?;

        let filesize: i64 = payload.data.len().try_into().map_err(|e| {
            log::error!(
                "put_file: failed convert filesize from usize to i64, too big?: {}",
                e
            );
            actix_web::error::ErrorInternalServerError("put_file: file too large")
        })?;

        let (_file_id, canon_filename) = insert_attachment(
            &s3_filename,
            &payload.hash.to_string(),
            filesize,
            &payload.mime.to_string(),
            None,
            None,
        )
        .await
        .map_err(|e| {
            log::error!("put_file: failed select_attachment_by_filename_hash: {}", e);
            actix_web::error::ErrorInternalServerError("put_file: DB error")
        })?;

        if canon_filename.is_none() {
            let count = list.key_count.ok_or_else(|| {
                log::error!("put_file: key_count, I don't think this should ever happen");
                actix_web::error::ErrorInternalServerError(
                    "put_file: failed to check if file exists",
                )
            })?;

            // count should only ever be 0 or 1, otherwise there's something wrong with the prefix
            if count == 0 {
                bucket
                    .put_object(payload.data, &s3_filename)
                    .await
                    .map_err(|e| {
                        log::error!("put_file: failed to put_object: {}", e);
                        actix_web::error::ErrorInternalServerError("put_file: failed to store file")
                    })?;
            } else {
                log::info!("put_file: duplicate upload, skipping S3 put_object");
            }
        } else {
            log::info!("put_file: duplicate found in DB, skipping S3 put_object");
        }

        // !!! WARNING !!! we delete a file, be mindful and don't fucking delete my porn folder
        log::warn!("Deleting Tmp File: {:#?}", payload.tmp_path);
        std::fs::remove_file(payload.tmp_path).map_err(|e| {
            log::error!("put_file: delete tmp file error: {}", e);
            actix_web::error::ErrorInternalServerError("put_file: failed to store file")
        })?;
    }

    Ok(HttpResponse::Ok().body("put_file: ok"))
}

/// this is my fancy intelligent extension extractor
fn get_extension_guess(filename: &str) -> Option<String> {
    fn get_extension_guess_return(filename: &str, idx: usize) -> Option<String> {
        Some(filename[idx + 1..].to_ascii_lowercase())
    }
    const MAX_EXT_LEN: usize = 24; // tar.zst.sha256sum.gpg rounded up to divisible by 8 for autism
    const MAX_SUB_EXT_LEN: usize = 9; // longest extensions I can think of: sha256sum/gitignore

    // get and specially check the top-level extension, we intentionally skip some rules
    let mut begin_idx = match filename.rfind('.') {
        Some(idx) => {
            if idx == 0
                || idx == filename.len()
                || filename.len() - idx > MAX_SUB_EXT_LEN + 1 // +1 because we count the '.' here
                || !filename[idx + 1..].chars().all(|x| x.is_ascii_alphanumeric())
            {
                return None;
            }

            // we have a list of extensions that we're okay with just accepting
            match get_ext_lookup().get(&filename[idx + 1..]) {
                Some(ext) => {
                    log::error!("EXT_LOOKUP: {}", ext);
                    return Some(ext.to_string());
                }
                None => idx,
            }
        }
        None => return None,
    };

    loop {
        let sub_str = &filename[..begin_idx];
        log::error!("sub_str: {}", sub_str);

        // find beginning of next possible extension
        let new_idx = match sub_str.rfind('.') {
            Some(idx) => idx,
            None => return get_extension_guess_return(filename, begin_idx),
        };

        // check if double period
        if new_idx == begin_idx - 1 {
            log::info!("get_extension_greedy: found double");
            return get_extension_guess_return(filename, begin_idx);
        }

        if filename.len() - new_idx > MAX_EXT_LEN {
            log::info!("get_extension_greedy: more than MAX_EXT_LEN");
            return get_extension_guess_return(filename, begin_idx);
        }

        // new sub-extension
        let sub_ext = &sub_str[new_idx + 1..];

        // check if too long
        if sub_ext.len() > MAX_SUB_EXT_LEN {
            log::info!("get_extension_greedy: more than MAX_SUB_EXT_LEN");
            return get_extension_guess_return(filename, begin_idx);
        }

        // check if all numbers

        if sub_ext.parse::<u32>().is_ok() {
            log::info!("get_extension_greedy: all numbers");
            return get_extension_guess_return(filename, begin_idx);
        }

        // check if isn't ASCII
        if !sub_ext.chars().all(|x| x.is_ascii_alphanumeric()) {
            log::info!("get_extension_greedy: not ASCII");
            return get_extension_guess_return(filename, begin_idx);
        }

        begin_idx = new_idx;
    }
}

/// returns (file_id, true) if duplicate found, (file_id, false) if new
async fn insert_attachment(
    filename: &str,
    hash: &str,
    filesize: i64,
    mime: &str,
    user_id: Option<i32>,
    user_ip: Option<i32>,
) -> Result<(i32, Option<String>), DbErr> {
    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        id: i32,
        filename: String,
    }

    let db = get_db_pool();
    let txn = db.begin().await?;

    let select = attachments::Entity::find()
        .select_only()
        .column(attachments::Column::Id)
        .column(attachments::Column::Filename)
        .filter(attachments::Column::Hash.eq(hash));

    // Check for duplicate attachment and insert if new
    let now = Utc::now().naive_utc();
    let (attachment_id, canon_filename) = match select
        .into_model::<SelectResult>()
        .one(&txn)
        .await?
    {
        Some(res) => {
            // Update last_seen_at
            log::error!("Duplicate File Hash: {:#?} - {}", res, filename);
            // I use update_many because it seems cleaner than using an actual ActiveModel
            let rows_updated = attachments::Entity::update_many()
                .col_expr(attachments::Column::LastSeenAt, Expr::value(now))
                .filter(attachments::Column::Id.eq(res.id))
                .exec(db)
                .await?;
            if rows_updated.rows_affected != 1 {
                log::error!("insert_attachment: SANITY ERROR: more than 1 row updated on last_seen_at update: {:?}", rows_updated.rows_affected);
            }

            (res.id, Some(res.filename))
        }
        None => {
            // Insert attachment
            let new_attachment = attachments::ActiveModel {
                filename: Set(filename.to_owned()),
                hash: Set(hash.to_owned()),
                first_seen_at: Set(now),
                last_seen_at: Set(now),
                filesize: Set(filesize),
                mime: Set(mime.to_owned()),
                meta: Set(JsonValue::Null),
                ..Default::default()
            };
            let res = attachments::Entity::insert(new_attachment)
                .exec(&txn)
                .await?;
            (res.last_insert_id, None)
        }
    };

    // Insert UGC
    let new_ugc = ugc::ActiveModel {
        ugc_revision_id: Set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    // TODO add user ID stuff

    // Insert UGC Attachment
    let new_ugc_attachment = ugc_attachments::ActiveModel {
        attachment_id: Set(attachment_id),
        ugc_id: Set(new_ugc.id.unwrap()),
        user_id: Set(user_id),
        ip_id: Set(user_ip),
        created_at: Set(now),
        filename: Set(filename.to_owned()),
        ..Default::default()
    };
    let res = ugc_attachments::Entity::insert(new_ugc_attachment)
        .exec(&txn)
        .await?;
    txn.commit().await?;

    Ok((res.last_insert_id, canon_filename))
}

fn get_extension(filename: &str, mime: &Mime) -> Option<String> {
    // We check the MIME manually because the mime and mime_guess crates are both inadequate. We
    // are only looking for formats where we can assume it is the only relevant extension.
    // For example we'd never want to add a format like .gz to the hashmaps, we'd rely on _guess for that.
    let result = get_mime_lookup().get(mime.as_ref().to_ascii_lowercase().as_str());
    match result {
        Some(v) => {
            log::info!("MIME_LOOKUP: Found {}", v);
            Some(v.to_string())
        }
        None => get_extension_guess(filename),
    }

    // Old Method, static hashmap is probably faster than a jump table
    //
    // match mime.type_() {
    //     mime::IMAGE => match mime.subtype().as_str() {
    //         "apng" => Some("apng".to_owned()),
    //         "avif" => Some("avif".to_owned()),
    //         "bmp" => Some("bmp".to_owned()),
    //         "gif" => Some("gif".to_owned()),
    //         "jpeg" => Some("jpeg".to_owned()),
    //         "png" => Some("png".to_owned()),
    //         "svg+xml" => Some("svg".to_owned()),
    //         "webp" => Some("webp".to_owned()),
    //         _ => get_extension_guess(filename),
    //     },
    //     mime::VIDEO => match mime.subtype().as_str() {
    //         "x-msvideo" => Some("avi".to_owned()),
    //         "ogg" => Some("ogv".to_owned()),
    //         "webm" => Some("webm".to_owned()),
    //         "x-matroska" => Some("mkv".to_owned()),
    //         _ => get_extension_guess(filename),
    //     },
    //     mime::AUDIO => match mime.subtype().as_str() {
    //         "m4a" => Some("m4a".to_owned()),
    //         "ogg" => Some("ogg".to_owned()),
    //         "webm" => Some("webm".to_owned()),
    //         "x-matroska" => Some("mka".to_owned()),
    //         _ => get_extension_guess(filename),
    //     },
    //     _ => get_extension_guess(filename),
    // }
}

#[derive(Debug, FromQueryResult)]
pub struct SelectFilename {
    pub filename: String,
}

pub async fn get_filename_by_id(id: i32) -> Result<Option<SelectFilename>, DbErr> {
    Ok(attachments::Entity::find_by_id(id)
        .select_only()
        .column(attachments::Column::Filename)
        .into_model::<SelectFilename>()
        .one(get_db_pool())
        .await?)
}

pub async fn get_filename_by_ugc(ugc_id: i32) -> Result<Option<SelectFilename>, DbErr> {
    Ok(ugc_attachments::Entity::find()
        .select_only()
        .column(attachments::Column::Filename)
        .inner_join(attachments::Entity)
        .filter(ugc_attachments::Column::Id.eq(ugc_id))
        .into_model::<SelectFilename>()
        .one(get_db_pool())
        .await?)
}

pub async fn get_file_url(s3: &S3Bucket, id: i32) -> Result<Option<String>, DbErr> {
    match get_filename_by_id(id).await? {
        Some(result) => Ok(Some(format!(
            "http://{}/{}/{}/{}", // TODO something
            s3.pub_url,
            &result.filename[0..2],
            &result.filename[2..4],
            result.filename
        ))),
        None => Ok(None),
    }
}

pub async fn get_file_url_by_ugc(s3: &S3Bucket, ugc_id: i32) -> Result<Option<String>, DbErr> {
    match get_filename_by_ugc(ugc_id).await? {
        Some(result) => Ok(Some(format!(
            "http://{}/{}/{}/{}", // TODO something
            s3.pub_url,
            &result.filename[0..2],
            &result.filename[2..4],
            result.filename
        ))),
        None => Ok(None),
    }
}
