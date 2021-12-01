use crate::templates::LoginTemplate;
use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use askama_actix::TemplateToResponse;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use ruforo::MyAppData;
use serde::Deserialize;

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
    session: Session,
    form: web::Form<FormData>,
    my: web::Data<MyAppData<'static>>,
) -> impl Responder {
    // don't forget to sanitize kek and add error handling
    let pass_match = web::block(move || {
        let conn = my.pool.get().expect("couldn't get db connection from pool");
        login(&conn, &form.username, &form.password, &my)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    });
    match pass_match {
        Ok(pass_match) => match pass_match {
            Ok(pass_match) => {
                println!("Pass: {:?}", pass_match);
                if pass_match {
                    match session.insert("logged_in", true) {
                        Ok(_) => HttpResponse::Ok().finish(),
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
