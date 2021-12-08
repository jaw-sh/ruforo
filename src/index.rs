use crate::frontend::TemplateToPubResponse;
use crate::session::{Client, MainData};
use actix_web::{get, web, Responder};
use askama_actix::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub client: &'a Client<'a>,
    pub start_time: &'a chrono::NaiveDateTime,
}

#[get("/")]
async fn view_index(my: web::Data<MainData<'_>>) -> impl Responder {
    for (key, value) in &*my.cache.sessions.read().unwrap() {
        println!(
            "Session: {} / {:?}",
            key,
            value.expire.format("%Y-%m-%d %H:%M:%S").to_string()
        );
    }

    let client = Client { user: None };

    IndexTemplate {
        client: &client,
        start_time: &my.cache.start_time,
    }
    .to_pub_response()
}
