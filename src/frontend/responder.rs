use super::Context;
use actix_web::{error, Error, HttpRequest, HttpResponse};
use askama_actix::{Template, TemplateToResponse};

/// Page container to wrap public views.
#[derive(Template)]
#[template(path = "container/public.html", escape = "none")]
struct PublicTemplate<'a> {
    context: &'a Context,
    content: &'a str,
}

pub trait TemplateToPubResponse {
    fn to_pub_response(&self) -> Result<PublicResponse, Error>;
}

/// Produces an actix-web HttpResponse with a partial template that will be inset with the public container.
impl<T: askama::Template> TemplateToPubResponse for T {
    fn to_pub_response(&self) -> Result<PublicResponse, Error> {
        let mut buffer = String::new();
        if self.render_into(&mut buffer).is_err() {
            return Err(error::ErrorInternalServerError("Template parsing error"));
        }

        Ok(PublicResponse { content: buffer })
    }
}

/// PublicResponder wraps content from an inner template for the outer public Page Container.
/// It implements the actix_web::Responder trait so that it can be returned as a result in actix_web functions.
/// When returned to actix_web as the result of controller logic, it can access the HttpRequest and its extensions and pass it as context to the PublicTemplate.
pub struct PublicResponse {
    content: String,
}

impl actix_web::Responder for PublicResponse {
    fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        if !req.extensions().contains::<Context>() {
            return error::ErrorInternalServerError("Failed to pass context to container template")
                .error_response();
        }

        PublicTemplate {
            content: &self.content,
            context: req.extensions().get::<Context>().unwrap(),
        }
        .to_response()
    }
}
