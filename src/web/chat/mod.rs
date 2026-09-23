pub mod connection;
pub mod implement;
pub mod message;
pub mod server;
pub mod watcher;

use actix::Addr;
use actix_files as fs;
use actix_web::{get, web, web::Data, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use askama_actix::Template;
use implement::{ChatLayer, Room};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::middleware::ClientCtx;

/// How often heartbeat pings are sent
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(view_chat_socket).service(view_chat);
}

/// Statically configured browser origins permitted to open the chat WebSocket:
/// the origin of `XF_PUBLIC_URL` plus every entry of `CHAT_ALLOWED_ORIGINS`
/// (comma-separated, e.g. `https://example.com,http://example.onion`).
static STATIC_ALLOWED_ORIGINS: Lazy<Vec<String>> = Lazy::new(|| {
    let mut list: Vec<String> = std::env::var("XF_PUBLIC_URL")
        .ok()
        .map(|url| url_origin(&url))
        .filter(|s| !s.is_empty())
        .into_iter()
        .collect();
    if let Ok(v) = std::env::var("CHAT_ALLOWED_ORIGINS") {
        list.extend(v.split(',').map(normalize_origin).filter(|s| !s.is_empty()));
    }
    list
});

/// Origins discovered at runtime by the chat layer (e.g. the XF MultiSite domain
/// list), refreshed periodically via [`set_dynamic_allowed_origins`].
static DYNAMIC_ALLOWED_ORIGINS: Lazy<RwLock<Vec<String>>> = Lazy::new(|| RwLock::new(Vec::new()));

/// Replaces the runtime-discovered origin allow-list.
pub fn set_dynamic_allowed_origins(origins: Vec<String>) {
    let origins: Vec<String> = origins
        .iter()
        .map(|o| normalize_origin(o))
        .filter(|s| !s.is_empty())
        .collect();
    match DYNAMIC_ALLOWED_ORIGINS.write() {
        Ok(mut guard) => *guard = origins,
        Err(poisoned) => *poisoned.into_inner() = origins,
    }
}

fn is_origin_allowed(origin: &str) -> bool {
    if STATIC_ALLOWED_ORIGINS.iter().any(|allowed| allowed == origin) {
        return true;
    }
    match DYNAMIC_ALLOWED_ORIGINS.read() {
        Ok(guard) => guard.iter().any(|allowed| allowed == origin),
        Err(poisoned) => poisoned.into_inner().iter().any(|allowed| allowed == origin),
    }
}

fn normalize_origin(origin: &str) -> String {
    origin.trim().trim_end_matches('/').to_ascii_lowercase()
}

/// Turns a bare hostname into a browser origin: `https://host`, except Tor
/// onion services, which are served over plain `http://`.
pub fn host_to_origin(host: &str) -> String {
    let host = host.trim().trim_end_matches('/').to_ascii_lowercase();
    if host.is_empty() {
        return String::new();
    }
    let bare = host.split(':').next().unwrap_or("");
    if bare.ends_with(".onion") {
        format!("http://{}", host)
    } else {
        format!("https://{}", host)
    }
}

/// Reduces a URL like `https://host:port/path` to its origin `https://host:port`.
pub fn url_origin(url: &str) -> String {
    let url = url.trim();
    let origin = match url.find("://") {
        Some(idx) => {
            let after = idx + 3;
            match url[after..].find('/') {
                Some(slash) => &url[..after + slash],
                None => url,
            }
        }
        None => url,
    };
    normalize_origin(origin)
}

/// Cross-site WebSocket hijacking guard. Browsers always send `Origin` on a
/// WebSocket handshake, so a present-but-unlisted Origin is rejected. Requests
/// without an Origin (non-browser clients) cannot carry a victim's cookies
/// cross-site and are allowed.
fn check_ws_origin(req: &HttpRequest) -> Result<(), Error> {
    let origin = match req.headers().get(actix_web::http::header::ORIGIN) {
        None => return Ok(()),
        Some(value) => match value.to_str() {
            Ok(s) => normalize_origin(s),
            Err(_) => return Err(actix_web::error::ErrorForbidden("Invalid origin")),
        },
    };

    if is_origin_allowed(&origin) {
        Ok(())
    } else {
        log::warn!("Rejected chat WebSocket from disallowed origin {:?}", origin);
        Err(actix_web::error::ErrorForbidden("Origin not allowed"))
    }
}

/// Serializes a value to JSON that is safe to inline inside an HTML <script>.
/// serde_json does not escape `<`, so a username such as `</script>` would
/// otherwise terminate the script element.
fn script_safe_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .expect("JSON stringify failed")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

/// Entry point for our websocket route
#[get("/chat.ws")]
pub async fn view_chat_socket(
    client: ClientCtx,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    check_ws_origin(&req)?;

    let layer = req
        .app_data::<Data<Arc<dyn ChatLayer>>>()
        .expect("No chat layer.");

    let session = if let Some(id) = client.get_id() {
        layer.get_session_from_user_id(id as u32).await
    } else {
        let token = layer.get_session_key_from_request(&req);
        let user_id = layer.get_user_id_from_token(token).await;
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

/// Entry point for our websocket route (xf compat)
#[get("/chat.ws")]
pub async fn view_xf_chat_socket(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    check_ws_origin(&req)?;

    let layer = req
        .app_data::<Data<Arc<dyn ChatLayer>>>()
        .expect("No chat layer.");

    let token = layer.get_session_key_from_request(&req);
    let user_id = layer.get_user_id_from_token(token).await;
    let session = layer.get_session_from_user_id(user_id).await;

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
            std::env::var("CHAT_WS_URL").expect("CHAT_WS_URL needs to be set in .env"),
            script_safe_json(&session),
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
        std::env::var("CHAT_ASSET_DIR").unwrap_or_else(|_| ".".to_string())
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

    let token = layer.get_session_key_from_request(&req);
    let user_id = layer.get_user_id_from_token(token).await;
    let session = layer.get_session_from_user_id(user_id).await;
    let mut hasher = blake3::Hasher::new();

    // Hash: Salt
    match std::env::var("SALT") {
        Ok(v) => hasher.update(v.as_bytes()),
        Err(_) => hasher.update("NO_SALT".as_bytes()),
    };
    // Hash: Timestamp
    use actix_files as fs;
    use std::path::PathBuf;
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
            script_safe_json(&session),
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

/// Dynamically access public files through the webserver.
#[get("/assets/{filename:.*}")]
async fn view_public_file(req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let base_dir = std::env::var("CHAT_ASSET_DIR").unwrap_or_else(|_| ".".to_string());
    let mut base_path = PathBuf::from(&base_dir);

    let filename: String = req.match_info().query("filename").parse().unwrap();

    // Sanitize filename to prevent directory traversal
    let sanitized_filename = filename
        .split('/')
        .filter(|component| !component.is_empty() && *component != ".." && *component != ".")
        .collect::<Vec<&str>>()
        .join("/");

    if sanitized_filename.is_empty() {
        return Err(actix_web::error::ErrorBadRequest("Invalid filename"));
    }

    base_path.push(&sanitized_filename);

    // Canonicalize paths to resolve any remaining traversal attempts
    let canonical_base = std::fs::canonicalize(&base_dir)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Base directory not found"))?;
    let canonical_requested = std::fs::canonicalize(&base_path)
        .map_err(|_| actix_web::error::ErrorNotFound("File not found"))?;

    // Ensure the requested file is within the base directory
    if !canonical_requested.starts_with(&canonical_base) {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    let file = fs::NamedFile::open(canonical_requested)?;
    Ok(file.use_last_modified(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_origin_strips_path_and_case() {
        assert_eq!(url_origin("https://XF.test"), "https://xf.test");
        assert_eq!(url_origin("https://xf.test/"), "https://xf.test");
        assert_eq!(url_origin("https://xf.test:5443/forum/"), "https://xf.test:5443");
    }

    #[test]
    fn host_to_origin_schemes() {
        assert_eq!(host_to_origin("XF.pa.test"), "https://xf.pa.test");
        assert_eq!(host_to_origin("abcdef.onion"), "http://abcdef.onion");
        assert_eq!(host_to_origin(""), "");
    }

    #[test]
    fn dynamic_origins_are_checked() {
        assert!(!is_origin_allowed("https://mirror.example"));
        set_dynamic_allowed_origins(vec!["https://Mirror.example/".to_owned()]);
        assert!(is_origin_allowed("https://mirror.example"));
        set_dynamic_allowed_origins(vec![]);
        assert!(!is_origin_allowed("https://mirror.example"));
    }

    #[test]
    fn script_safe_json_escapes_markup() {
        let out = script_safe_json(&"</script><b>&");
        assert!(!out.contains('<') && !out.contains('>') && !out.contains('&'));
        assert_eq!(
            serde_json::from_str::<String>(&out).unwrap(),
            "</script><b>&"
        );
    }
}
