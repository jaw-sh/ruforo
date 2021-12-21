use actix_web::{get, HttpResponse};
use once_cell::sync::OnceCell;
use rsass::{compile_scss_path, output};
use std::fs;

static CSS_CACHE: OnceCell<Vec<u8>> = OnceCell::new();
static JS: OnceCell<Vec<u8>> = OnceCell::new();

#[inline(always)]
fn get_css_data() -> &'static [u8] {
    unsafe { CSS_CACHE.get_unchecked() }
}
#[inline(always)]
fn get_global_js() -> &'static [u8] {
    unsafe { JS.get_unchecked() }
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

    let js_data = fs::read("public/global.js").expect("failed to read public/global.js");
    JS.set(js_data).unwrap();
}

#[get("/style.css")]
pub async fn view_css() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/css")
        .body(get_css_data())
}

#[get("/global.js")]
pub async fn view_js() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/javascript")
        .body(get_global_js())
}
