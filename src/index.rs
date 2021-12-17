use crate::middleware::ClientCtx;
use crate::session::{get_sess, get_start_time};
use actix_web::{get, Responder};
use askama_actix::{Template, TemplateToResponse};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub client: ClientCtx,
    pub start_time: &'a chrono::NaiveDateTime,
}

#[get("/")]
async fn view_index(client: ClientCtx) -> impl Responder {
    for (key, value) in &*get_sess().read().unwrap() {
        println!(
            "Session: {} / {:?}",
            key,
            value.expires_at.format("%Y-%m-%d %H:%M:%S").to_string()
        );
    }

    IndexTemplate {
        client,
        start_time: get_start_time(),
    }
    .to_response()
}
