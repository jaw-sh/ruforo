pub mod context;
pub mod css;

pub use self::context::Context;

use actix_web::{error, Error, HttpRequest, HttpResponse, Responder};
use askama_actix::{Template, TemplateToResponse};
use bytes::{Bytes, BytesMut};
use futures::future::{ready, Ready};
use serde::Serialize;

/// Page container for most public views.
#[derive(Template)]
#[template(path = "container/public.html", escape = "none")]
struct PublicPageTemplate<'a> {
    context: &'a Context,
    content: &'a str,
}

pub trait TemplateToPubResponse {
    fn to_pub_response(&self) -> Result<PublicPageResponder, Error>;
}

/// Produces an actix-web HttpResponse with a partial template that will be inset with the public container.
impl<T: askama::Template> TemplateToPubResponse for T {
    fn to_pub_response(&self) -> Result<PublicPageResponder, Error> {
        let mut buffer = String::new();
        if self.render_into(&mut buffer).is_err() {
            return Err(error::ErrorInternalServerError("Template parsing error"));
        }

        Ok(PublicPageResponder { content: buffer })
    }
}

///
pub struct PublicPageResponder {
    content: String,
}

///
impl actix_web::Responder for PublicPageResponder {
    fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        if !req.extensions().contains::<Context>() {
            return error::ErrorInternalServerError("Failed to pass context to container template")
                .error_response();
        }

        PublicPageTemplate {
            content: &self.content,
            context: req.extensions().get::<Context>().unwrap(),
        }
        .to_response()
    }
}
