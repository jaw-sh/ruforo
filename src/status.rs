use crate::frontend::TemplateToPubResponse;
use crate::session::MainData;
use crate::template::StatusTemplate;
use actix_web::{get, web, Responder};

#[get("/status")]
pub async fn view_status(my: web::Data<MainData<'_>>) -> impl Responder {
    for (key, value) in &*my.cache.sessions.read().unwrap() {
        println!(
            "Session: {} / {:?}",
            key,
            value.expire.format("%Y-%m-%d %H:%M:%S").to_string()
        );
    }
    StatusTemplate {
        start_time: &my.cache.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        logged_in: true,
        username: None,
    }
    .to_pub_response()
}
