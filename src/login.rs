// use crate::session::new_session;
use crate::orm::users;
use crate::orm::users::Entity as Users;
use crate::session;
use crate::templates::LoginTemplate;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use askama_actix::TemplateToResponse;
use ruforo::MainData;
use sea_orm::{entity::*, query::*, DatabaseConnection, FromQueryResult, QueryFilter};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: String,
}

async fn login(
    db: &DatabaseConnection,
    name_: &str,
    pass_: &str,
    my: &web::Data<MainData<'static>>,
) -> Result<i32, Error> {
    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        id: i32,
        password: String,
    }

    let select = Users::find()
        .select_only()
        .column(users::Column::Id)
        .column(users::Column::Password)
        .filter(users::Column::Name.eq(name_));

    let user = select
        .into_model::<SelectResult>()
        .one(db)
        .await
        .map_err(|e| {
            log::error!("Login: {}", e);
            error::ErrorInternalServerError("DB error")
        })?;

    match user {
        Some(user) => {
            let parsed_hash = PasswordHash::new(&user.password).unwrap();
            my.argon2
                .verify_password(pass_.as_bytes(), &parsed_hash)
                .map_err(|_| error::ErrorInternalServerError("user not found or bad password"))?;
            Ok(user.id)
        }
        None => Err(error::ErrorInternalServerError(
            "user not found or bad password",
        )),
    }
}

#[post("/login")]
pub async fn login_post(
    session: actix_session::Session,
    form: web::Form<FormData>,
    my: web::Data<MainData<'static>>,
) -> Result<HttpResponse, Error> {
    // don't forget to sanitize kek and add error handling
    let user_id = login(&my.pool, &form.username, &form.password, &my).await?;

    log::error!("test");
    let uuid = session::new_session(&my.pool, &my.cache.sessions, user_id)
        .await
        .map_err(|e| {
            log::error!("error {:?}", e);
            error::ErrorInternalServerError("DB error")
        })?;

    session
        .insert("logged_in", true)
        .map_err(|_| error::ErrorInternalServerError("middleware error"))?;

    session
        .insert("token", uuid)
        .map_err(|_| error::ErrorInternalServerError("middleware error"))?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/login")]
pub async fn login_get() -> impl Responder {
    LoginTemplate {
        logged_in: true,
        username: None,
    }
    .to_response()
}
