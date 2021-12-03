pub mod css {
    use actix_web::{get, Error, HttpResponse};
    use rsass::{compile_scss_path, output};

    // TODO
    // What we will want eventually is to precompile the CSS at `cargo run` and move it to /public/,
    // and allow CSS editing without restarting the application. For now, this works. As the CSS
    // becomes more complex this solution will be less acceptable.
    #[get("/style.css")]
    pub async fn read_css() -> Result<HttpResponse, Error> {
        let path = "templates/css/main.scss".as_ref();
        let format = output::Format {
            style: output::Style::Compressed,
            ..Default::default()
        };

        Ok(HttpResponse::Ok()
            .content_type("text/css")
            .body(compile_scss_path(path, format).unwrap()))
    }
}
