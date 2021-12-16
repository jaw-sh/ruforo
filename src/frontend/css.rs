use actix_web::{get, Error, HttpResponse};
use once_cell::sync::OnceCell;
use rsass::{compile_scss_path, output};

static CSS_CACHE: OnceCell<Vec<u8>> = OnceCell::new();

#[inline(always)]
fn get_css_lookup() -> &'static [u8] {
    unsafe { CSS_CACHE.get_unchecked() }
}

pub fn init() {
    let path = "templates/css/main.scss".as_ref();
    let format = output::Format {
        style: output::Style::Compressed,
        ..Default::default()
    };
    CSS_CACHE
        .set(compile_scss_path(path, format).unwrap())
        .unwrap();
}

#[get("/style.css")]
pub async fn view_css() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/css")
        .body(get_css_lookup()))
}
