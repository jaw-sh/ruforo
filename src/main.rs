#[macro_use]
extern crate lazy_static;
extern crate dotenv;
extern crate ffmpeg_next;

use actix::Actor;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use env_logger::Env;
use middleware::AppendContext;
use std::path::Path;

mod create_user;
mod forum;
mod hub;
mod index;
mod login;
mod logout;
mod member;
mod middleware;

lazy_static! {
    static ref SALT: SaltString = {
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
        SaltString::new(&salt).unwrap()
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    ffmpeg_next::init().expect("!!! ffmpeg Init Failure !!!");

    // Check Cache Dir
    let cache_dir = std::env::var("DIR_TMP")
        .expect("missing DIR_TMP environment variable (hint: 'DIR_TMP=./tmp')");
    let cache_path = Path::new(&cache_dir);
    if !cache_path.exists() {
        std::fs::DirBuilder::new()
            .recursive(true)
            .create(cache_path)
            .expect("failed to create DIR_TMP");
    }

    let data = web::Data::new(ruforo::session::init_data(&SALT).await);
    let chat = web::Data::new(ruforo::chat::ChatServer::new().start());

    // Start HTTP server
    HttpServer::new(move || {
        // Authentication policy
        let policy = CookieIdentityPolicy::new(&[0; 32]) // TODO: Set a 32B Salt
            .name("auth")
            .secure(true);

        App::new()
            .app_data(data.clone())
            .app_data(chat.clone())
            // Order of middleware IS IMPORTANT and is in REVERSE EXECUTION ORDER.
            .wrap(AppendContext::default())
            .wrap(IdentityService::new(policy))
            .wrap(Logger::new("%a %{User-Agent}i"))
            // https://www.restapitutorial.com/lessons/httpmethods.html
            // GET    edit_ (get edit form)
            // PATCH  update_ (apply edit)
            // GET    view_ (read/view/render entity)
            // Note: PUT and PATCH were added, removed, and re-added(?) to the HTML5 spec for <form method="">
            .service(index::view_index)
            .service(create_user::create_user_get)
            .service(create_user::create_user_post)
            .service(login::view_login)
            .service(login::post_login)
            .service(logout::view_logout)
            .service(member::view_members)
            .service(ruforo::filesystem::view_file_ugc)
            .service(ruforo::filesystem::view_file_canonical)
            .service(ruforo::filesystem::put_file)
            .service(ruforo::post::delete_post)
            .service(ruforo::post::destroy_post)
            .service(ruforo::post::edit_post)
            .service(ruforo::post::update_post)
            .service(ruforo::post::view_post_by_id)
            .service(ruforo::post::view_post_in_thread)
            .service(forum::create_thread)
            .service(forum::view_forum)
            .service(ruforo::frontend::css::view_css)
            .service(ruforo::thread::create_reply)
            .service(ruforo::thread::view_thread)
            .service(ruforo::thread::view_thread_page)
            .service(web::resource("/chat").to(hub::chat_route))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
