use crate::templates::IndexTemplate;
use actix_session::Session;
use actix_web::{get, Responder};
use askama_actix::TemplateToResponse;

#[get("/")]
async fn index(_session: Session) -> impl Responder {
    // if let Some(count) = session.get::<i32>("counter")? {
    //     session.insert("counter", count + 1)?;
    // } else {
    //     session.insert("counter", 1)?;
    // }

    IndexTemplate {
        logged_in: true,
        username: None,
    }
    .to_response()
}
