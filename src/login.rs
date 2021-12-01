use crate::templates::LoginTemplate;
use actix_web::{get, post, web, HttpResponse, Responder};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use askama_actix::TemplateToResponse;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use ruforo::MyAppData;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: String,
}

type DbError = Box<dyn std::error::Error + Send + Sync>;

fn login(
    db: &PgConnection,
    name_: &str,
    pass_: &str,
    my: &web::Data<MyAppData<'static>>,
) -> Result<bool, DbError> {
    use ruforo::schema::users::dsl::*;

    let password_hash = users
        .filter(name.eq(name_))
        .select(password)
        .first::<String>(db)?;

    let parsed_hash = PasswordHash::new(&password_hash).unwrap();
    return Ok(my
        .argon2
        .verify_password(pass_.as_bytes(), &parsed_hash)
        .is_ok());
}

#[post("/login")]
pub async fn login_post(
    session: actix_session::Session,
    form: web::Form<FormData>,
    my: web::Data<MyAppData<'static>>,
) -> impl Responder {
    // don't forget to sanitize kek and add error handling
    let my2 = my.clone();
    let pass_match = web::block(move || {
        let conn = my2
            .pool
            .get()
            .expect("couldn't get db connection from pool");
        login(&conn, &form.username, &form.password, &my2)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    });
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
