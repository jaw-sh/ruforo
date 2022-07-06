use crate::middleware::ClientCtx;
use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::ServiceResponse;
use actix_web::http::{header, header::HeaderValue, StatusCode};
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::{Error, Result};
use askama_actix::Template;

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate<'a> {
    client: ClientCtx,
    status: StatusCode,
    error: Option<&'a Error>,
}

pub fn error_document<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let body = BoxBody::new(
        ErrorTemplate {
            client: ClientCtx::default(),
            status: res.status(),
            error: res.response().error(),
        }
        .to_string(),
    );
    let mut res: ServiceResponse<EitherBody<B>> =
        res.map_body(|_, _| EitherBody::<B, BoxBody>::right(body));

    // Headers must be manually set because Actix-Web renders no content by default.
    let headers = res.response_mut().headers_mut();
    // Web document
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
    // Proxies (Cloudflare) love to cache error pages permanently. Explicitly say not to do that.
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));

    Ok(ErrorHandlerResponse::Response(res))
}

pub fn render_400<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    error_document::<B>(res)
}

pub fn render_404<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    error_document::<B>(res)
}

pub fn render_500<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    error_document::<B>(res)
}
