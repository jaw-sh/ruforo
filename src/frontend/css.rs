use actix_web::{get, Error, HttpResponse};
use rsass::{compile_scss_path, output};

lazy_static! {
    static ref CSS_CACHE: Vec<u8> = {
        let path = "templates/css/main.scss".as_ref();
        let format = output::Format {
            style: output::Style::Compressed,
            ..Default::default()
        };
        compile_scss_path(path, format).unwrap()
    };
}

#[get("/style.css")]
pub async fn view_css() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/css")
        .body(CSS_CACHE.as_slice()))
}
