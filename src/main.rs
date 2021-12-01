extern crate dotenv;

use actix_session::{CookieSession, Session};
use actix_web::middleware::Logger;
use actix_web::{get, web, App, Error, HttpResponse, HttpServer, Responder};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2;
use dotenv::dotenv;
use env_logger::Env;
use std::env;

mod chat;
mod templates;
mod thread;
mod ugc;
use templates::HelloTemplate;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

fn new_db_manager() -> r2d2::ConnectionManager<PgConnection> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    r2d2::ConnectionManager::<PgConnection>::new(database_url)
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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
    // Init logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Create connection pool
    dotenv().ok();
    let manager = new_db_manager();
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // Start HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
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
            .service(thread::create_reply)
            .service(thread::read_thread)
            //.service(create_user::create_user)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("127.0.0.1:8080")?
    .run();
    println!("Server running at http://{}/", "127.0.0.1:8080");

    server.await
}
