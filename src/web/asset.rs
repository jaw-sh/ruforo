use actix_files as fs;
use actix_web::http::{header, header::ContentEncoding, StatusCode};
use actix_web::{get, Error, HttpRequest, HttpResponse, Responder};
use std::path::PathBuf;

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(view_file_by_hash).service(view_public_file);
}

/// Route for passing local assets through the webserver.
/// /content/9e0834c0d3dd1f6a775b9af7523eff7b35e750afb8fcd2753eef06735e13c46f/whatever.jpg
#[get("/content/{hash:.*}/{filename:.*}")]
async fn view_file_by_hash(req: HttpRequest) -> impl Responder {
    let hash: String = req.match_info().query("hash").parse().expect("Bad hash.");
    let key: String = match crate::attachment::get_attachment_by_hash(hash).await {
        Some(attachment) => attachment.filename,
        None => {
            return HttpResponse::NotFound().body("404 - Resource not found");
        }
    };

    //let name: String = req
    //    .match_info()
    //    .query("filename")
    //    .parse()
    //    .expect("Bad filename.");

    // Multimedia range
    let range: Option<String> = req
        .headers()
        .get("Range")
        .and_then(|r| r.to_str().ok())
        .map(From::from);

    let res = match crate::filesystem::get_s3().get_object(&key, range).await {
        Ok(output) => output,
        Err(err) => {
            log::debug!("{:?}", err);
            return HttpResponse::NotFound().body("404 - Content not found");
        }
    };

    //let body = res.body.expect("No body for response").map(Bytes::from).map_err(Error::from);
    let body = res.body.expect("No body for response");
    let mut builder = HttpResponse::Ok();

    if let Some(content_length) = res.content_length {
        builder.append_header((header::CONTENT_LENGTH, content_length as u64));
    }
    if let Some(content_type) = res.content_type {
        // Don't gzip media files
        if content_type.starts_with("audio")
            || content_type.starts_with("video")
            || content_type.starts_with("image")
        {
            builder.append_header((header::CONTENT_ENCODING, ContentEncoding::Identity));
        }
        if content_type == "binary/octet-stream" || content_type == "application/octet-stream" {
            //if let Some(extension) = Path::new(&hash).extension().and_then(|s| s.to_str()) {
            //    let mime = mime_guess::get_mime_type(extension);
            //    let mime = mime.as_ref();
            //    builder.content_type(mime);
            //}
        } else {
            builder.content_type(content_type.as_str());
        }
    }
    if let Some(e_tag) = res.e_tag {
        builder.append_header((header::ETAG, e_tag));
    }
    if let Some(content_range) = res.content_range {
        builder.append_header((header::CONTENT_RANGE, content_range));
        builder.status(StatusCode::PARTIAL_CONTENT);
    }
    if let Some(accept_ranges) = res.accept_ranges {
        builder.append_header((header::ACCEPT_RANGES, accept_ranges));
    }
    if let Some(last_modified) = res.last_modified {
        builder.append_header((header::LAST_MODIFIED, last_modified));
    }

    builder.append_header(("Cache-Control", "public, max-age=31536000"));

    builder.streaming(body)
}

/// Dynamically access public files through the webserver.
#[get("/public/assets/{filename:.*}")]
async fn view_public_file(req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let mut path: PathBuf = PathBuf::from("public/assets/");
    let req_path: PathBuf = req.match_info().query("filename").parse().unwrap();
    path.push(req_path.file_name().unwrap());

    let file = fs::NamedFile::open(path)?;

    Ok(file.use_last_modified(true))
}
