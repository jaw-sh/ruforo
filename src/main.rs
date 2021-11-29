use actix_session::{CookieSession, Session};
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use env_logger::Env;
use chrono::{NaiveDateTime, NaiveTime, Utc, TimeZone};
use serde::Deserialize;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

mod chat;

mod templates;
use templates::HelloTemplate;

struct Board {
	id: u32,
	name: String,
	description: String,
}

struct User {
	id: u64,
	username: String,
	email: String,
	join_date: NaiveDateTime,
}

pub fn establish_connection() -> PgConnection {
	dotenv().ok();

	let database_url = env::var("DATABASE_URL")
		.expect("DATABASE_URL must be set");
	PgConnection::establish(&database_url)
		.expect(&format!("Error connecting to {}", database_url))
}

#[get("/")]
async fn hello() -> impl Responder {
	HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
	HttpResponse::Ok().body(req_body)
}

#[derive(Deserialize)]
struct FormData {
	username: String,
}

#[post("/create_user")]
async fn create_user(form: web::Form<FormData>) -> impl Responder {
	HttpResponse::Ok().body(format!("username: {}", form.username))
}

async fn manual_hello() -> impl Responder {
	HttpResponse::Ok().body("Hey there!")
}

async fn index(session: Session) -> Result<HttpResponse, Error> {
	// access session data
	if let Some(count) = session.get::<i32>("counter")? {
		session.insert("counter", count + 1)?;
	} else {
		session.insert("counter", 1)?;
	}

	Ok(HttpResponse::Ok().body(format!(
		"Count is {:?}!",
		session.get::<i32>("counter")?.unwrap()
	)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
	HttpServer::new(|| {
		App::new()
			.wrap(Logger::default())
			.wrap(Logger::new("%a %{User-Agent}i"))
			.wrap(
				CookieSession::signed(&[0; 32]) // <- create cookie based session middleware
					.secure(false),
			)
			.service(web::resource("/ws/").route(web::get().to(chat::ws_index)))
			.service(web::resource("/").to(index))
			.service(web::resource("/t").to(|| async { HelloTemplate { name: "nigger" } }))
			.service(hello)
			.service(echo)
			.route("/hey", web::get().to(manual_hello))
	})
	.bind("127.0.0.1:8080")?
	.run()
	.await
}
