use crate::frontend::TemplateToPubResponse;
use crate::session::MainData;
use crate::user::Client;
use actix_identity::Identity;
use actix_web::{get, web, Responder};
use askama_actix::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub client: &'a Client,
    pub start_time: &'a chrono::NaiveDateTime,
}

#[get("/")]
async fn view_index(id: Identity, data: web::Data<MainData<'_>>) -> impl Responder {
    let client: Client = data.client_from_identity(id);

    for (key, value) in &*data.cache.sessions.read().unwrap() {
        println!(
            "Session: {} / {:?}",
            key,
            value.expire.format("%Y-%m-%d %H:%M:%S").to_string()
        );
    }

    IndexTemplate {
        client: &client,
        start_time: &data.cache.start_time,
    }
    .to_pub_response()
}
