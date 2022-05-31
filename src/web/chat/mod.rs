pub mod client;
pub mod server;

use crate::compat::xf::session::get_user_from_request;
use crate::middleware::ClientCtx;
use actix::Addr;
use actix_web::{get, web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use askama_actix::{Template, TemplateToResponse};
use std::time::{Duration, Instant};

/// How often heartbeat pings are sent
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Entry point for our websocket route
pub async fn service(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        client::WsChatSession {
            id: get_user_from_request(&req),
            hb: Instant::now(),
            room: "Main".to_owned(),
            name: None,
            addr: srv.get_ref().clone(),
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
