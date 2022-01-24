use crate::attachment::{get_attachment_by_hash, update_attachment_last_seen};
use crate::init::get_db_pool;
use crate::orm::{attachments, ugc_attachments};
use crate::s3::S3Bucket;
use actix_multipart::{Field, Multipart};
use actix_web::{error, get, http::header::ContentType, post, web, Error, HttpResponse, Responder};
use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use mime::Mime;
use once_cell::sync::OnceCell;
use sea_orm::{entity::*, query::*, DbErr, FromQueryResult, JsonValue, QueryFilter};
use serde::Deserialize;
use serde::Serialize;
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
        ("jfif", "jpeg"),
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

#[derive(Deserialize)]
pub struct FileHashFormData {
    pub hash: String,
}

pub struct UploadPayload {
    data: Vec<u8>,
    filename: String,
    hash: blake3::Hash,
    tmp_path: PathBuf,
    mime: Mime,
}

#[derive(Debug, FromQueryResult, Serialize)]
pub struct UploadResponse {
    pub id: i32,
    pub hash: String,
}

#[post("/fs/check-file")]
pub async fn post_file_hash(form: web::Json<FileHashFormData>) -> Result<impl Responder, Error> {
    // TODO: I do not know why .len() returns 64 when it should be 32.
    if form.hash.len() != 64 {
        // note: .len() is byte count
        return Err(error::ErrorBadRequest(format!(
            "Malformed BLAKE3 hash (b{}).",
            form.hash.len()
        )));
    };

    let file = attachments::Entity::find()
        .column(attachments::Column::Id)
        .column(attachments::Column::Hash)
        .filter(attachments::Column::Hash.eq(form.hash.to_owned()))
        .into_model::<UploadResponse>()
        .one(get_db_pool())
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(web::Json(file))
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
        error::ErrorInternalServerError("view_file: bad ID")
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
    let mut responses: Vec<UploadResponse> = Vec::new();

    // Iterate over multipart stream
    while let Ok(Some(mut field)) = mutipart.try_next().await {
        match insert_field_as_attachment(&mut field).await {
            Ok(response) => responses.push(response),
            Err(err) => log::debug!("Threw out field: {}", err),
        }
    }

    Ok(web::Json(responses))
}

/// Accepts an upload payload and attempts to deduplicate it.
pub async fn deduplicate_payload(payload: &UploadPayload) -> Option<UploadResponse> {
    // Look for an existing database entry
    let model = get_attachment_by_hash(payload.hash.to_string()).await;

    match model {
        // Attachment exists in storage and we can skip processing
        Some(attachment) => {
            // Bump last_seen date on new thread.
            actix_web::rt::spawn(update_attachment_last_seen(attachment.id));
            // Return response now.
            Some(UploadResponse {
                id: attachment.id,
                hash: attachment.hash,
            })
        }
        // Attachment is new and we need to process it
        None => None,
    }
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

#[derive(Debug, FromQueryResult)]
pub struct SelectFilename {
    pub filename: String,
}

pub fn get_file_url_by_filename(filename: &String) -> String {
    format!(
        "http://{}/{}/{}/{}", // TODO something
        get_s3().pub_url,     // Is this legal? I think it's legal.
        &filename[0..2],
        &filename[2..4],
        filename
    )
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

// Direct way of converting an actix_multipart field into an upload response.
pub async fn insert_field_as_attachment(field: &mut Field) -> Result<UploadResponse, Error> {
    // Save the file to a temporary location and get payload data.
    let payload = save_field_as_temp_file(field).await?;
    // Pass file through deduplication and receive a response..
    match deduplicate_payload(&payload).await {
        Some(response) => Ok(response),
        None => insert_payload_as_attachment(payload, None).await,
    }
}

/// Receives a request payload and inserts it into the database and the s3 bucket.
pub async fn insert_payload_as_attachment(
    payload: UploadPayload,
    constraints: Option<fn(i64, (Option<i32>, Option<i32>)) -> Result<bool, Error>>,
) -> Result<UploadResponse, Error> {
    log::info!("Filename: {}", payload.filename);
    log::info!("BLAKE3: {}", payload.hash);
    log::info!("MIME: {}", payload.mime);

    let dimensions: (Option<i32>, Option<i32>);
    let extension: Option<String>;

    match crate::ffmpeg::open_with_ffmpeg(&payload.tmp_path) {
        Some(ffmpeg) => {
            dimensions = match crate::ffmpeg::get_dimensions_from_input(&ffmpeg) {
                Some(xy) => (Some(xy.0 as i32), Some(xy.1 as i32)),
                None => (None, None),
            };
            extension = match crate::ffmpeg::get_extension_from_input(&ffmpeg) {
                Some(ffext) => Some(ffext),
                None => get_extension(&payload.filename, &payload.mime),
            };
        }
        None => {
            dimensions = (None, None);
            extension = get_extension(&payload.filename, &payload.mime);
        }
    };

    let s3_filename = match extension {
        Some(extension) => format!("{}.{}", payload.hash, extension),
        None => payload.hash.to_string(),
    };

    let filesize: i64 = payload.data.len().try_into().map_err(|e| {
        log::error!(
            "put_file: failed convert filesize from usize to i64, too big?: {}",
            e
        );
        actix_web::error::ErrorInternalServerError("put_file: file too large")
    })?;

    // Custom constraint checks
    // Before we insert into the database and save the file, ask the specific implementation
    // if this file meets our requirements.
    if let Some(constraint_fn) = constraints {
        match constraint_fn(filesize, dimensions) {
            Ok(_) => {}
            Err(err) => {
                log::error!("put_file constraints failed: {}", err);
                return Err(actix_web::error::ErrorBadRequest(err));
            }
        }
    }

    let now = Utc::now().naive_utc();
    let hash = &payload.hash.to_string();
    let new_attachment = attachments::ActiveModel {
        // This is our canonical filename, not the user's filename.
        // User's filename belongs in ugc_attachments.
        filename: Set(s3_filename.to_owned()),
        hash: Set(hash.to_owned()),
        first_seen_at: Set(now),
        last_seen_at: Set(now),
        filesize: Set(filesize),
        file_width: Set(dimensions.0),
        file_height: Set(dimensions.1),
        mime: Set(payload.mime.to_string()),
        meta: Set(JsonValue::Null),
        ..Default::default()
    };

    // Insert the attachment into the database.
    let res = attachments::Entity::insert(new_attachment)
        .exec(get_db_pool())
        .await
        .map_err(|e| {
            log::error!("put_file: failed to put_object: {}", e);
            actix_web::error::ErrorInternalServerError("put_file: failed to store file")
        })?;

    let bucket = get_s3();
    let list = bucket.list_objects_v2(&s3_filename).await.map_err(|e| {
        log::error!("put_file: failed to list_objects_v2: {}", e);
        actix_web::error::ErrorInternalServerError("put_file: failed to check if file exists")
    })?;

    // Check for existing s3 data.
    let count = list.key_count.ok_or_else(|| {
        log::error!("put_file: key_count, I don't think this should ever happen");
        actix_web::error::ErrorInternalServerError("put_file: failed to check if file exists")
    })?;

    // s3 key count should only ever be 0 or 1, otherwise there's something wrong with the prefix
    if count == 0 {
        // Insert the file data into s3.
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

    // !!! WARNING !!! we delete a file, be mindful and don't fucking delete my porn folder
    log::warn!("Deleting Tmp File: {:#?}", payload.tmp_path);
    std::fs::remove_file(payload.tmp_path).map_err(|e| {
        log::error!("put_file: delete tmp file error: {}", e);
        actix_web::error::ErrorInternalServerError("put_file: failed to store file")
    })?;

    Ok(UploadResponse {
        id: res.last_insert_id,
        hash: hash.to_owned(),
    })
}

/// Accepts a multipart field, stores it on the disk, and returns indetifying information about it.
pub async fn save_field_as_temp_file(field: &mut Field) -> Result<UploadPayload, Error> {
    let content_type = field.content_disposition();
    let filename = content_type
        .get_filename()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("put_file: missing filename"))?
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

    Ok(UploadPayload {
        data: buf,
        filename,
        tmp_path: filepath, // Warning: This is deleted at the end of processing.
        hash: hasher.finalize(),
        mime: field.content_type().to_owned(),
    })
}
