use crate::frontend;
use crate::frontend::TemplateToPubResponse;
use crate::templates::IndexTemplate;
use actix_session::Session;
use actix_web::{get, web, Responder};

#[get("/")]
async fn index(_session: Session, ctx: web::ReqData<frontend::Context>) -> impl Responder {
    IndexTemplate {
        logged_in: true,
        username: None,
    }
    .to_pub_response(&ctx)
}
