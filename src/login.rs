use crate::frontend::TemplateToPubResponse;
use crate::orm::users;
use crate::session;
use crate::session::{authenticate_by_cookie, MainData};
use crate::template::LoginTemplate;
use crate::user::get_user_id_from_name;
use actix_identity::Identity;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use sea_orm::{entity::*, DatabaseConnection, FromQueryResult};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: String,
}

async fn login(
    db: &DatabaseConnection,
    name: &str,
    pass: &str,
    my: &web::Data<MainData<'_>>,
) -> Result<i32, Error> {
    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        id: i32,
        password: String,
    }

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
    my.argon2
        .verify_password(pass.as_bytes(), &parsed_hash)
        .map_err(|_| error::ErrorInternalServerError("User not found or password is incorrect."))?;
    Ok(user.id)
}

#[post("/login")]
pub async fn post_login(
    id: Identity,
    cookies: actix_session::Session,
    form: web::Form<FormData>,
    my: web::Data<MainData<'_>>,
) -> Result<HttpResponse, Error> {
    // TODO: Sanitize input and check for errors.
    let user_id = login(&my.pool, &form.username, &form.password, &my).await?;

    let uuid = session::new_session(&my.pool, &my.cache.sessions, user_id)
        .await
        .map_err(|e| {
            log::error!("error {:?}", e);
            error::ErrorInternalServerError("DB error")
        })?;

    cookies
        .insert("logged_in", true)
        .map_err(|_| error::ErrorInternalServerError("middleware error"))?;

    cookies
        .insert("token", uuid)
        .map_err(|_| error::ErrorInternalServerError("middleware error"))?;

    id.remember(uuid.to_string());
    Ok(HttpResponse::Ok().finish())
}

#[get("/login")]
pub async fn view_login(
    my: web::Data<MainData<'_>>,
    cookies: actix_session::Session,
) -> Result<impl Responder, Error> {
    let uuid;
    let mut tmpl = LoginTemplate {
        user_id: None,
        logged_in: false,
        username: None,
        token: None,
    };

    if let Some(session) = authenticate_by_cookie(&my.cache.sessions, &cookies) {
        tmpl.user_id = Some(session.session.user_id);
        tmpl.logged_in = true;
        uuid = session.uuid.to_string();
        tmpl.token = Some(&uuid);
    }

    Ok(tmpl.to_pub_response())
}
