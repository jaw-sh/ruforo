use crate::templates::CreateUserTemplate;
use actix_web::{get, post, web, HttpResponse, Responder};
use argon2::PasswordHasher;
use askama_actix::TemplateToResponse;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use ruforo::models::{NewUser, User};
use ruforo::DbPool;
use ruforo::MyAppData;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: String,
}

type DbError = Box<dyn std::error::Error + Send + Sync>;

fn insert_new_user(_db: &PgConnection, _username: &str, _password: &str) -> Result<User, DbError> {
    use ruforo::schema::users::dsl::*;

    let user = NewUser {
        created_at: diesel::dsl::now,
        name: _username,
        password: _password,
    };

    diesel::insert_into(users)
        .values(&user)
        .execute(_db)
        .expect("Error inserting person");

    let user = users
        .filter(name.eq(&user.name))
        .first::<User>(_db)
        .expect("Error loading person");

    Ok(user)
}

#[get("/create_user")]
pub async fn create_user_get() -> impl Responder {
    CreateUserTemplate {
        logged_in: true,
        username: None,
    }
    .to_response()
}
#[post("/create_user")]
pub async fn create_user_post(
    pool: web::Data<DbPool>,
    form: web::Form<FormData>,
    my: web::Data<MyAppData<'static>>,
) -> impl Responder {
    // don't forget to sanitize kek and add error handling
    let _user = web::block(move || {
        let conn = pool.get().expect("couldn't get db connection from pool");
        let password_hash = my
            .argon2
            .hash_password(form.password.as_bytes(), &my.salt)
            .unwrap()
            .to_string();
        insert_new_user(&conn, &form.username, &password_hash)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    });
    HttpResponse::Ok().finish()
}
