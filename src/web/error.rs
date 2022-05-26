use crate::middleware::ClientCtx;
use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::ServiceResponse;
use actix_web::http::{
    header,
    header::{HeaderName, HeaderValue},
};
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::Result;
use askama_actix::{Template, TemplateToResponse};

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    client: Option<ClientCtx>,
}

#[allow(unused_variables)]
pub fn render_500<B>(mut res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let body = BoxBody::new(ErrorTemplate { client: None }.to_string());
    let mut res: ServiceResponse<EitherBody<B>> =
        res.map_body(|_, _| EitherBody::<B, BoxBody>::right(body));

    let headers = res.response_mut().headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));

    Ok(ErrorHandlerResponse::Response(res))
}
