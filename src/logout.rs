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
    data: web::Data<MainData<'_>>,
    cookies: actix_session::Session,
) -> Result<impl Responder, Error> {
    let tmpl = LogoutTemplate {};

    // TODO: Needs mechanism to alter the HttpRequest.extensions stored Context and Client during this request cycle.
    match cookies.get::<String>("token") {
        Ok(token) => match token {
            Some(uuid) => match Uuid::parse_str(&uuid) {
                Ok(uuid) => remove_session(&data.cache.sessions, uuid).await,
                Err(_) => None,
            },
            None => None,
        },
        _ => None,
    };

    Ok(tmpl.to_pub_response())
}
