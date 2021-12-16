use crate::frontend::TemplateToPubResponse;
use crate::session::{get_start_time, get_sess};
use crate::user::Client;
use actix_web::{get, Responder};
use askama_actix::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub client: &'a Client,
    pub start_time: &'a chrono::NaiveDateTime,
}

#[get("/")]
async fn view_index(client: Client) -> impl Responder {
    for (key, value) in &*get_sess().read().unwrap() {
        println!(
            "Session: {} / {:?}",
            key,
            value.expires_at.format("%Y-%m-%d %H:%M:%S").to_string()
        );
    }

    IndexTemplate {
        client: &client,
        start_time: get_start_time(),
    }
    .to_pub_response()
}
