pub mod connection;
pub mod message;
pub mod server;

use crate::compat::xf::session::get_user_from_request;
use actix::Addr;
use actix_web::{get, web, web::Data, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use askama_actix::{Template, TemplateToResponse};
use sea_orm::DatabaseConnection;
use std::time::{Duration, Instant};

/// How often heartbeat pings are sent
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Entry point for our websocket route
pub async fn service(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let db = req
        .app_data::<Data<DatabaseConnection>>()
        .expect("No chat server.");
    let session = get_user_from_request(db, &req).await;

    println!("Session: {:?}", session);

    ws::start(
        connection::Connection {
            id: usize::MIN, // mutated by server
            session,
            hb: Instant::now(),
            room: "Main".to_owned(),
            addr: req
                .app_data::<Addr<server::ChatServer>>()
                .expect("No chat server.")
                .clone(),
        },
        &req,
        stream,
    )
}

#[derive(Template)]
#[template(path = "chat.html")]
struct ChatTestTemplate {}

#[get("/test-chat")]
pub async fn view_chat() -> impl Responder {
    ChatTestTemplate {}.to_response()
}
