use actix_web::{Error, post, web, HttpResponse};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use ruforo::DbPool;
use ruforo::models::{User, NewUser};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
	username: String,
}

type DbError = Box<dyn std::error::Error + Send + Sync>;

fn insert_new_user(_db: &PgConnection, _username: &str, _email: Option<&str>) -> Result<User, DbError> {
	use ruforo::schema::users::dsl::*;

	let user = NewUser {
		username: _username,
		join_date: diesel::dsl::now,
		email: _email,
	};

	diesel::insert_into(users)
		.values(&user)
		.execute(_db)
		.expect("Error inserting person");

	let user = users
		.filter(username.eq(&user.username))
		.first::<User>(_db)
		.expect("Error loading person");

	Ok(user)
}

#[post("/create_user")]
pub async fn create_user(pool: web::Data<DbPool>, form: web::Form<FormData>) -> Result<HttpResponse, Error> {
	// don't forget to sanitize kek and add error handling
	let user =
		web::block(move || {
			let conn = pool.get().expect("couldn't get db connection from pool");
			insert_new_user(&conn, &form.username, Some("yeet@email.com"))
		})
			.await
			.map_err(|e| {
				eprintln!("{}", e);
				HttpResponse::InternalServerError().finish()
			});
	Ok(HttpResponse::Ok().json(user.unwrap().unwrap()))
}

