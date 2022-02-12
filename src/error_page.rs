use actix_web::middleware::{ErrorHandlers, Response};
use actix_web::{http, App, HttpRequest, HttpResponse, Result};

fn render_500<S>(_: &mut HttpRequest<S>, resp: HttpResponse) -> Result<Response> {
    let mut builder = resp.into_builder();
    builder.header(http::header::CONTENT_TYPE, "application/json");
    Ok(Response::Done(builder.into()))
}
