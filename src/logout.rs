use crate::frontend::TemplateToPubResponse;
use crate::session::{remove_session, MainData};
use actix_web::{get, web, Error, Responder};
use askama_actix::Template;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "logout.html")]
pub struct LogoutTemplate {}

#[get("/logout")]
pub async fn view_logout(
    id: actix_identity::Identity,
    cookies: actix_session::Session,
    data: web::Data<MainData>,
) -> Result<impl Responder, Error> {
    let tmpl = LogoutTemplate {};

    // TODO: Needs mechanism to alter the HttpRequest.extensions stored Context and Client during this request cycle.
    match cookies.get::<String>("token") {
        Ok(Some(uuid)) => match Uuid::parse_str(&uuid) {
            Ok(uuid) => {
                if let Err(e) = remove_session(&data.cache.sessions, uuid).await {
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
    Ok(tmpl.to_pub_response())
}
