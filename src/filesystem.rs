use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpResponse, Responder};
use futures::{StreamExt, TryStreamExt};
use std::io::Write;

#[post("/fs/upload-file")]
pub async fn put_file(mut payload: Multipart) -> Result<impl Responder, Error> {
    // see: https://users.rust-lang.org/t/file-upload-in-actix-web/64871/3

    Ok(HttpResponse::Ok()
        .content_type("text/css")
        .body("update_succeeded"))
}
