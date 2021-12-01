use actix_web::{post, web, HttpResponse, Responder};
use argon2::PasswordHasher;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use ruforo::DbPool;
use ruforo::models::{User, NewUser};
use serde::Deserialize;
use ruforo::MyAppData;

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: String,
    email: String,
}

type DbError = Box<dyn std::error::Error + Send + Sync>;

fn insert_new_user(_db: &PgConnection, _username: &str, _password: &str, _email: Option<&str>) -> Result<User, DbError> {
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

#[post("/create_user")]
pub async fn create_user(pool: web::Data<DbPool>, form: web::Form<FormData>, my: web::Data<MyAppData<'static>>) -> impl Responder {
    // don't forget to sanitize kek and add error handling
    let user =
        web::block(move || {
            let conn = pool.get().expect("couldn't get db connection from pool");
            let password_hash = my.argon2.hash_password(form.password.as_bytes(), &my.salt).unwrap().to_string();
            insert_new_user(&conn, &form.username, &password_hash, Some(&form.email))
            // insert_new_user(&conn, &form.username, &form.password, Some(&form.email))
        })
            .await
            .map_err(|e| {
                eprintln!("{}", e);
                HttpResponse::InternalServerError().finish()
            });
    HttpResponse::Ok().finish()
}

