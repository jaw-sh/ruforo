use actix_web::{get, web, Responder};
use askama_actix::Template;
use ruforo::frontend::TemplateToPubResponse;
use ruforo::session::MainData;
use ruforo::user::Client;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub client: &'a Client,
    pub start_time: &'a chrono::NaiveDateTime,
}

#[get("/")]
async fn view_index(client: Client, data: web::Data<MainData<'_>>) -> impl Responder {
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
