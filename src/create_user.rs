use crate::frontend::TemplateToPubResponse;
use crate::orm::users;
use crate::session::MainData;
use crate::template::CreateUserTemplate;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    PasswordHasher,
};
use chrono::Utc;
use sea_orm::{entity::*, DatabaseConnection, DbErr, InsertResult, QueryTrait};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: String,
}

async fn insert_new_user(
    db: &DatabaseConnection,
    name: &str,
    pass: &str,
) -> Result<InsertResult<users::ActiveModel>, DbErr> {
    let user = users::ActiveModel {
        created_at: Set(Utc::now().naive_utc()),
        name: Set(name.to_owned()),
        password: Set(pass.to_owned()),
        password_cipher: Set(users::Cipher::Argon2id),
        ..Default::default() // all other attributes are `Unset`
    };
    // let res = user.insert(conn).await.expect("Error inserting person");
    log::error!(
        "SQL: {}",
        users::Entity::insert(user.to_owned())
            .build(sea_orm::DatabaseBackend::Postgres)
            .to_string()
    );
    let res = users::Entity::insert(user).exec(db).await?;

    Ok(res)
}

#[get("/create_user")]
pub async fn create_user_get() -> impl Responder {
    CreateUserTemplate {
        logged_in: true,
        username: None,
    }
    .to_pub_response()
}
#[post("/create_user")]
pub async fn create_user_post(
    form: web::Form<FormData>,
    my: web::Data<MainData<'_>>,
) -> Result<HttpResponse, Error> {
    // don't forget to sanitize kek and add error handling
    let password_hash = my
        .argon2
        .hash_password(form.password.as_bytes(), &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();
    insert_new_user(&my.pool, &form.username, &password_hash)
        .await
        .map_err(|e| {
            log::error!("{}", e);
            error::ErrorInternalServerError("user not found or bad password")
        })?;
    Ok(HttpResponse::Ok().finish())
}
