pub mod connection;
pub mod implement;
pub mod message;
pub mod server;

use actix::Addr;
use actix_web::{get, web, web::Data, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use askama_actix::Template;
use implement::{ChatLayer, Room};
use serde::Deserialize;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::middleware::ClientCtx;

/// How often heartbeat pings are sent
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);
/// How long before lack of client response causes a timeout
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(view_chat_socket).service(view_chat);
}
/// Entry point for our websocket route
#[get("/chat.ws")]
pub async fn view_chat_socket(
    client: ClientCtx,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let layer = req
        .app_data::<Data<Arc<dyn ChatLayer>>>()
        .expect("No chat layer.");

    let session = if let Some(id) = client.get_id() {
        layer.get_session_from_user_id(id as u32).await
    } else {
        let user_id = layer.get_user_id_from_request(&req);
        layer.get_session_from_user_id(user_id).await
    };

    ws::start(
        connection::Connection {
            id: usize::MIN, // mutated by server
            session,
            hb: Instant::now(),
            room: None,
            addr: req
                .app_data::<Addr<server::ChatServer>>()
                .expect("No chat server.")
                .clone(),
            last_command: Instant::now(),
        },
        &req,
        stream,
    )
}

#[derive(Template)]
#[template(path = "chat.html")]
struct ChatTemplate {
    client: ClientCtx,
    rooms: Vec<Room>,
    app_json: String,
}

/// Live chat in full application
#[get("/chat")]
pub async fn view_chat(client: ClientCtx, req: HttpRequest) -> impl Responder {
    let layer = req
        .app_data::<Data<Arc<dyn ChatLayer>>>()
        .expect("No chat layer.");
    let session = layer
        .get_session_from_user_id(client.get_id().unwrap_or(0) as u32)
        .await;

    ChatTemplate {
        client,
        app_json: format!(
            "{{
                chat_ws_url: \"{}\",
                user: {},
            }}",
            std::env::var("XF_WS_URL").expect("XF_WS_URL needs to be set in .env"),
            serde_json::to_string(&session).expect("XfSession stringify failed"),
        ),
        rooms: layer.get_room_list().await,
    }
}

#[derive(Template)]
#[template(path = "chat_shim.html")]
struct ChatTestTemplate {
    rooms: Vec<Room>,
    app_json: String,
    nonce: String,
    webpack_time: u64,
    style: String,
}

#[derive(Deserialize)]
pub struct ChatTestData {
    pub style: Option<String>,
}

/// Chat shim
#[get("/test-chat")]
pub async fn view_chat_shim(req: HttpRequest, query: web::Query<ChatTestData>) -> impl Responder {
    let webpack_time: u64 = match std::fs::metadata(format!(
        "{}/chat.js",
        std::env::var("CHAT_ASSET_DIR").unwrap_or(".".to_string())
    )) {
        Ok(metadata) => match metadata.modified() {
            Ok(time) => match time.duration_since(std::time::UNIX_EPOCH) {
                Ok(distance) => distance.as_secs(),
                Err(_) => {
                    log::warn!("Unable to do math on webpack chat.js modified at timestamp");
                    0
                }
            },
            Err(_) => {
                log::warn!("Unable to read metadata on webpack chat.js");
                0
            }
        },
        Err(_) => {
            log::warn!("Unable to open webpack chat.js for timestamp");
            0
        }
    };

    let layer = req
        .app_data::<Data<Arc<dyn ChatLayer>>>()
        .expect("No chat layer.");
    let user_id = layer.get_user_id_from_request(&req);
    let session = layer.get_session_from_user_id(user_id).await;
    let mut hasher = blake3::Hasher::new();

    // Hash: Salt
    match std::env::var("SALT") {
        Ok(v) => hasher.update(v.as_bytes()),
        Err(_) => hasher.update("NO_SALT".as_bytes()),
    };
    // Hash: Timestamp
    use std::time::{SystemTime, UNIX_EPOCH};
    hasher.update(
        &SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System close before 1970. Really?")
            .as_millis()
            .to_ne_bytes(),
    );
    // Hash: Session ID
    match req.cookie("xf_session") {
        Some(cookie) => hasher.update(cookie.value().as_bytes()),
        None => hasher.update("NO_SESSION_TO_HASH".as_bytes()),
    };

    ChatTestTemplate {
        rooms: layer.get_room_list().await,
        app_json: format!(
            "{{
                chat_ws_url: \"{}\",
                user: {},
            }}",
            std::env::var("XF_WS_URL").expect("XF_WS_URL needs to be set in .env"),
            serde_json::to_string(&session).expect("XfSession stringify failed"),
        ),
        nonce: hasher.finalize().to_string(),
        webpack_time,
        style: if query.style == Some("dark".to_string()) {
            "dark"
        } else {
            "light"
        }
        .to_string(),
    }
}
