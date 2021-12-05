pub mod css;

use actix_web::{error::ErrorInternalServerError, HttpResponse};
use askama_actix::{Template, TemplateToResponse};
use bytes::BytesMut;
use chrono::prelude::{NaiveDateTime, Utc};

#[derive(Debug)]
pub struct Context {
    pub request_start: NaiveDateTime,
}

impl Context {
    /// Returns human readable request time.
    pub fn request_time(&self) -> Option<i64> {
        (self.request_start - Utc::now().naive_utc()).num_microseconds()
    }
}

#[derive(Template)]
#[template(path = "container/public.html", escape = "none")]
pub struct PublicPageTemplate<'a> {
    context: &'a Context,
    content: String,
}

pub trait TemplateToPubResponse {
    fn to_pub_response(&self) -> HttpResponse;
}

// Produces an actix-web HttpResponse with a partial template that will be inset with the public container.
impl<T: askama::Template> TemplateToPubResponse for T {
    fn to_pub_response(&self) -> HttpResponse {
        // there is conceivably a way to do this with a byte buffer but for now i cant be bothered
        // the issue is that there's no BytesMut display implementation.
        //
        //let mut buffer = BytesMut::with_capacity(self.size_hint());
        //if self.render_into(&mut buffer).is_err() {
        //    return ErrorInternalServerError("Template rendering error (public)").error_response();
        //}
        //PublicPageTemplate { content: buffer }.to_response()
        dbg!(HttpResponse::Ok().extensions().get::<Context>());

        PublicPageTemplate {
            context: HttpResponse::Ok().extensions().get::<Context>().unwrap(),
            content: self.render().unwrap(),
        }
        .to_response()
    }
}
