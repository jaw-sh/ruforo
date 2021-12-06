use crate::frontend::TemplateToPubResponse;
use crate::templates::IndexTemplate;
use actix_session::Session;
use actix_web::{get, Responder};

#[get("/")]
async fn index(_session: Session) -> impl Responder {
    IndexTemplate {
        logged_in: true,
        username: None,
    }
    .to_pub_response()
}
