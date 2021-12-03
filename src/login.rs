// use crate::session::new_session;
use crate::proof::users;
use crate::proof::users::Entity as Users;
use crate::templates::LoginTemplate;
use actix_web::{get, post, web, HttpResponse, Responder};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use askama_actix::TemplateToResponse;
use ruforo::MyAppData;
use sea_orm::QueryFilter;
use sea_orm::{entity::*, DatabaseConnection, DbErr, InsertResult};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: String,
}

async fn login(
    db: &DatabaseConnection,
    name_: &str,
    pass_: &str,
    my: &web::Data<MyAppData<'static>>,
) -> Result<bool, DbErr> {
    // let password_hash = Users::find_by_id(1).one(db).await?;
    let password_hash = Users::find()
        .filter(users::Column::Name.eq(name_))
        .one(db)
        .await?;
    match password_hash {
        Some(password_hash) => {
            let parsed_hash = PasswordHash::new(&password_hash.password).unwrap();
            return Ok(my
                .argon2
                .verify_password(pass_.as_bytes(), &parsed_hash)
                .is_ok());
        }
        None => Ok(false),
    }
}

#[post("/login")]
pub async fn login_post(
    session: actix_session::Session,
    form: web::Form<FormData>,
    my: web::Data<MyAppData<'static>>,
) -> impl Responder {
    // don't forget to sanitize kek and add error handling
    // let pass_match = match login(&conn, &form.username, &form.password, &my2) {
    //     Ok(v) => v,
    //     Err(e) => {
    //         eprintln!("{}", e);
    //         HttpResponse::InternalServerError().finish();
    //     }
    // }
    match pass_match {
        Ok(pass_match) => match pass_match {
            Ok(pass_match) => {
                if pass_match {
                    match session.insert("logged_in", true) {
                        Ok(_) => {
                            let ses = ruforo::Session {
                                expire: chrono::Utc::now().naive_utc(),
                            };
                            let sessions = &mut *my.cache.sessions.write().unwrap();
                            loop {
                                let uuid = Uuid::new_v4();
                                if sessions.contains_key(&uuid) == false {
                                    sessions.insert(uuid, ses);
                                    break;
                                }
                            }
                            // new_session(my.pool.clone(), &my.cache.sessions, 0).await; // TODO replace user_id
                            HttpResponse::Ok().finish()
                        }
                        Err(_) => HttpResponse::InternalServerError().finish(),
                    }
                } else {
                    HttpResponse::Unauthorized().finish()
                }
            }
            Err(_) => HttpResponse::InternalServerError().finish(),
        },
        Err(e) => e,
    }
}

#[get("/login")]
pub async fn login_get() -> impl Responder {
    LoginTemplate {
        logged_in: true,
        username: None,
    }
    .to_response()
}
