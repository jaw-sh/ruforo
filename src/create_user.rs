use crate::db::get_db_pool;
use crate::middleware::ClientCtx;
use crate::orm::users;
use crate::session::get_argon2;
use crate::template::CreateUserTemplate;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    PasswordHasher,
};
use askama_actix::TemplateToResponse;
use chrono::Utc;
use sea_orm::{entity::*, DbErr, InsertResult, TransactionTrait};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: String,
}

async fn insert_new_user(
    name: &str,
    pass: &str,
) -> Result<InsertResult<users::ActiveModel>, DbErr> {
    use crate::orm::{user_name_history, user_names};
    use futures::join;

    let db = get_db_pool();
    let txn = db.begin().await?;
    let now = Utc::now().naive_utc();

    // Insert user
    let user = users::ActiveModel {
        created_at: Set(now),
        password: Set(pass.to_owned()),
        password_cipher: Set(users::Cipher::Argon2id),
        ..Default::default() // all other attributes are `Unset`
    };
    let res = users::Entity::insert(user).exec(db).await?;

    let user_name_ins = user_names::ActiveModel {
        user_id: Set(res.last_insert_id),
        name: Set(name.to_owned()),
    };

    let user_name_history_ins = user_name_history::ActiveModel {
        user_id: Set(res.last_insert_id),
        created_at: Set(now),
        approved_at: Set(now),
        name: Set(name.to_owned()),
        is_public: Set(true),
        ..Default::default()
    };

    // exec secondary inserts
    let (un_result, unh_result) = join!(
        user_names::Entity::insert(user_name_ins).exec(db),
        user_name_history::Entity::insert(user_name_history_ins).exec(db)
    );

    if un_result.is_err() {
        return Err(un_result.unwrap_err());
    }
    if unh_result.is_err() {
        return Err(unh_result.unwrap_err());
    }
    txn.commit().await?;

    Ok(res)
}

#[get("/create_user")]
pub async fn create_user_get(client: ClientCtx) -> impl Responder {
    CreateUserTemplate {
        client,
        logged_in: true,
        username: None,
    }
    .to_response()
}
#[post("/create_user")]
pub async fn create_user_post(form: web::Form<FormData>) -> Result<HttpResponse, Error> {
    // don't forget to sanitize kek and add error handling
    let password_hash = get_argon2()
        .hash_password(form.password.as_bytes(), &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();
    insert_new_user(&form.username, &password_hash)
        .await
        .map_err(|e| {
            log::error!("{}", e);
            error::ErrorInternalServerError("user not found or bad password")
        })?;
    Ok(HttpResponse::Ok().finish())
}
