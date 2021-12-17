use crate::init::get_db_pool;
use crate::middleware::ClientCtx;
use crate::orm::users;
use crate::session;
use crate::session::{authenticate_by_cookie, get_argon2, get_sess};
use crate::template::LoginTemplate;
use crate::user::get_user_id_from_name;
use actix_web::{error, get, post, web, Error, Responder};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use askama_actix::TemplateToResponse;
use sea_orm::{entity::*, FromQueryResult};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: String,
}

async fn login(name: &str, pass: &str) -> Result<i32, Error> {
    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        id: i32,
        password: String,
    }

    let db = get_db_pool();
    let user_id: i32 = get_user_id_from_name(db, name)
        .await
        .ok_or_else(|| error::ErrorBadRequest("User not found or password is incorrect."))?;

    let user = users::Entity::find_by_id(user_id)
        .into_model::<SelectResult>()
        .one(db)
        .await
        .map_err(|e| {
            log::error!("Login: {}", e);
            error::ErrorInternalServerError("DB error")
        })?
        .ok_or_else(|| {
            error::ErrorInternalServerError("User not found or password is incorrect.")
        })?;

    let parsed_hash = PasswordHash::new(&user.password).unwrap();
    get_argon2()
        .verify_password(pass.as_bytes(), &parsed_hash)
        .map_err(|_| error::ErrorInternalServerError("User not found or password is incorrect."))?;
    Ok(user.id)
}

#[post("/login")]
pub async fn post_login(
    client: ClientCtx,
    cookies: actix_session::Session,
    form: web::Form<FormData>,
) -> Result<impl Responder, Error> {
    // TODO: Sanitize input and check for errors.
    let user_id = login(&form.username, &form.password).await?;
    let uuid = session::new_session(get_sess(), user_id)
        .await
        .map_err(|e| {
            log::error!("error {:?}", e);
            error::ErrorInternalServerError("DB error")
        })?
        .to_string();

    cookies
        .insert("logged_in", true)
        .map_err(|_| error::ErrorInternalServerError("middleware error"))?;

    cookies
        .insert("token", uuid.to_owned())
        .map_err(|_| error::ErrorInternalServerError("middleware error"))?;

    Ok(LoginTemplate {
        client,
        user_id: Some(user_id),
        logged_in: true,
        username: Some(&form.username),
        token: Some(&uuid),
    }
    .to_response())
}

#[get("/login")]
pub async fn view_login(
    client: ClientCtx,
    cookies: actix_session::Session,
) -> Result<impl Responder, Error> {
    let mut tmpl = LoginTemplate {
        client,
        user_id: None,
        logged_in: false,
        username: None,
        token: None,
    };

    let uuid_str: String;
    if let Some((uuid, session)) = authenticate_by_cookie(&cookies).await {
        tmpl.user_id = Some(session.user_id);
        tmpl.logged_in = true;
        uuid_str = uuid.to_string();
        tmpl.token = Some(&uuid_str);
    }

    Ok(tmpl.to_response())
}
