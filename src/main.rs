extern crate dotenv;

use actix_session::CookieSession;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use env_logger::Env;
use ruforo::MyAppData;

mod chat;
mod create_user;
mod forum;
mod index;
mod login;
mod status;
pub mod templates;
mod thread;
pub mod ugc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Init logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    dotenv::dotenv().ok();
    let salt = match std::env::var("SALT") {
        Ok(v) => v,
        Err(e) => {
            let salt = SaltString::generate(&mut OsRng);
            panic!(
                "Missing SALT ({:?}) here's a freshly generated one: {}",
                e,
                salt.as_str()
            );
        }
    };
    let salt = SaltString::new(&salt).unwrap();

    let my_data = web::Data::new(MyAppData::new(salt));

    // // Argon2 with default params (Argon2id v19)
    // let argon2 = web::Data::new(Argon2::default());
    // // Hash password to PHC string ($argon2id$v=19$...)
    // let password_hash = argon2.hash_password(password, &salt).unwrap().to_string();
    // // Verify password against PHC string
    // let parsed_hash = PasswordHash::new(&password_hash).unwrap();
    // assert!(argon2.verify_password(password, &parsed_hash).is_ok());

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(my_data.clone())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(
                CookieSession::signed(&[0; 32]) // <- create cookie based session middleware
                    .secure(false),
            )
            .service(web::resource("/ws/").route(web::get().to(chat::ws_index)))
            .service(index::index)
            .service(create_user::create_user_get)
            .service(create_user::create_user_post)
            .service(login::login_get)
            .service(forum::create_thread)
            .service(forum::read_forum)
            .service(thread::create_reply)
            .service(thread::read_thread)
            .service(status::status_get)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
