use crate::middleware::ClientCtx;
use crate::session::{get_sess, remove_session};
use actix_web::{get, Error, Responder};
use askama_actix::{Template, TemplateToResponse};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "logout.html")]
struct LogoutTemplate {
    client: ClientCtx,
}

#[get("/logout")]
pub async fn view_logout(
    client: ClientCtx,
    id: actix_identity::Identity,
    cookies: actix_session::Session,
) -> Result<impl Responder, Error> {
    // TODO: Needs mechanism to alter the HttpRequest.extensions stored Context and Client during this request cycle.
    match cookies.get::<String>("token") {
        Ok(Some(uuid)) => match Uuid::parse_str(&uuid) {
            Ok(uuid) => {
                if let Err(e) = remove_session(get_sess(), uuid).await {
                    log::error!("view_logout: remove_session() {}", e);
                }
            }
            Err(e) => {
                log::error!("view_logout: parse_str() {}", e);
            }
        },
        Ok(None) => {
            log::error!("view_logout: missing token");
        }
        Err(e) => {
            log::error!("view_logout: cookies.get() {}", e);
        }
    }

    id.forget();
    cookies.remove("logged_in");
    cookies.remove("token");
    Ok(LogoutTemplate { client }.to_response())
}
