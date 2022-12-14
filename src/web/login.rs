use crate::db::get_db_pool;
use crate::middleware::ClientCtx;
use crate::orm::{user_2fa, user_names, users};
use crate::session;
use crate::session::{authenticate_by_cookie, get_argon2, get_sess};
use actix_web::{error, get, post, web, Error, Responder};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use askama::Template;
use askama_actix::TemplateToResponse;
use google_authenticator::GoogleAuthenticator;
use sea_orm::{entity::*, query::*, DbErr, FromQueryResult, QueryFilter};
use serde::Deserialize;

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(post_login).service(view_login);
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    pub client: ClientCtx,
    pub logged_in: bool,
    pub user_id: Option<i32>,
    pub username: Option<&'a str>,
    pub token: Option<&'a str>,
}

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: String,
    totp: Option<String>,
}

#[derive(Debug)]
pub enum LoginResultStatus {
    Success,
    BadName,
    BadPassword,
    Bad2FA,
    Missing2FA,
}

pub struct LoginResult {
    result: LoginResultStatus,
    user_id: Option<i32>,
}

impl LoginResult {
    fn success(user_id: i32) -> Self {
        Self {
            result: LoginResultStatus::Success,
            user_id: Some(user_id),
        }
    }
    fn fail(result: LoginResultStatus) -> Self {
        Self {
            result,
            user_id: None,
        }
    }
}

pub async fn login<S: AsRef<str>>(
    name: &str,
    pass: &str,
    totp: &Option<S>,
) -> Result<LoginResult, DbErr> {
    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        id: i32,
        password: String,
    }

    let db = get_db_pool();
    let user_id = user_names::Entity::find()
        .filter(user_names::Column::Name.eq(name))
        .one(db)
        .await?;

    let user_id = match user_id {
        Some(user) => user.user_id,
        None => return Ok(LoginResult::fail(LoginResultStatus::BadName)),
    };

    let user = users::Entity::find_by_id(user_id)
        .into_model::<SelectResult>()
        .one(db)
        .await?;

    let user = match user {
        Some(user) => user,
        None => return Ok(LoginResult::fail(LoginResultStatus::BadName)),
    };

    let parsed_hash = PasswordHash::new(&user.password).unwrap();
    if get_argon2()
        .verify_password(pass.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(LoginResult::fail(LoginResultStatus::BadPassword));
    }

    let totp_exists = user_2fa::Entity::find()
        .limit(1)
        .filter(user_2fa::Column::UserId.eq(user_id))
        .count(db)
        .await?;

    if totp_exists > 0 {
        if let Some(totp) = totp {
            let secret = user_2fa::Entity::find_by_id(user_id).one(db).await?;
            if let Some(secret) = secret {
                let auth = GoogleAuthenticator::new();
                let verify = auth.verify_code(&secret.secret, totp.as_ref(), 60, 0);
                if verify {
                    return Ok(LoginResult::success(user.id));
                }
                return Ok(LoginResult::fail(LoginResultStatus::Bad2FA));
            }
        }
        return Ok(LoginResult::fail(LoginResultStatus::Missing2FA));
    }

    Ok(LoginResult::success(user.id))
}

#[post("/login")]
pub async fn post_login(
    client: ClientCtx,
    cookies: actix_session::Session,
    form: web::Form<FormData>,
) -> Result<impl Responder, Error> {
    // TODO: Sanitize input and check for errors.
    let user_id = login(&form.username, &form.password, &form.totp)
        .await
        .map_err(|e| {
            log::error!("error {:?}", e);
            error::ErrorInternalServerError("DB error")
        })?;

    let user_id = match user_id.result {
        LoginResultStatus::Success => user_id.user_id.unwrap(),
        LoginResultStatus::Missing2FA => {
            // TODO: finish this
            return Err(error::ErrorForbidden("2FA required."));
        }
        _ => {
            log::debug!("login failure: {:?}", user_id.result);
            return Err(error::ErrorInternalServerError(
                "User not found or password is incorrect.",
            ));
        }
    };

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
        client: ClientCtx::from_session(&cookies, client.get_permissions().clone()).await,
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
