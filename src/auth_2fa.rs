use crate::init::get_db_pool;
use crate::middleware::ClientCtx;
use crate::orm::user_2fa;
use actix_web::{error, get, http::header::ContentType, Error, HttpResponse, Responder};
use google_authenticator::{ErrorCorrectionLevel, GoogleAuthenticator};
use sea_orm::{entity::*, query::*, DbErr, FromQueryResult, QueryFilter};

async fn db_user_enable_2fa(user_id: i32, secret: &str, email_reset: bool) -> Result<bool, DbErr> {
    let db = get_db_pool();
    let txn = db.begin().await?;

    let topt = user_2fa::Entity::find()
        .limit(1)
        .filter(user_2fa::Column::UserId.eq(user_id))
        .count(&txn)
        .await?;

    if topt > 0 {
        return Ok(false);
    }

    user_2fa::ActiveModel {
        user_id: Set(user_id),
        secret: Set(secret.to_owned()),
        email_reset: Set(email_reset),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    txn.commit().await?;

    Ok(true)
}

#[get("/user/enable_2fa")]
pub async fn user_enable_2fa(client: ClientCtx) -> Result<impl Responder, Error> {
    let auth = GoogleAuthenticator::new();
    let secret = auth.create_secret(32);
    let code = auth.get_code(&secret, 0).unwrap();
    let verify = auth.verify_code(&secret, &code, 0, 0);
    let qr = auth
        .qr_code(
            &secret,
            "name",
            "title",
            128,
            128,
            ErrorCorrectionLevel::Medium,
        )
        .unwrap();
    log::error!("qr: {}", qr);
    log::error!("secret: {}\ncode: {}\nverify: {}", secret, code, verify);

    let user_id = client.get_id().unwrap(); // TODO tmp unwrap
    let result = db_user_enable_2fa(user_id, &secret, false)
        .await
        .map_err(|e| {
            log::error!("Login: {}", e);
            error::ErrorInternalServerError("DB error")
        })?;

    if result {
        let body = format!(
            "<html><body><div>{}</div><div>{}</div></body></html>",
            secret, qr
        );
        Ok(HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(body))
    } else {
        let body = "<html><body>couldn't set 2fa</body></html>";
        Ok(HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(body))
    }
}
