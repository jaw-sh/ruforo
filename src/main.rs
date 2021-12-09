extern crate dotenv;
#[macro_use]
extern crate lazy_static;

use crate::session::{new_db_pool, reload_session_cache, MainData};
use actix::Actor;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use env_logger::Env;
use middleware::AppendContext;
use std::path::Path;

pub mod chat;
mod create_user;
pub mod filesystem;
mod forum;
pub mod frontend;
mod hub;
mod index;
mod login;
mod member;
mod middleware;
pub mod orm;
mod post;
pub mod s3;
pub mod session;
pub mod template;
mod thread;
pub mod ugc;
pub mod user;

lazy_static! {
    static ref SALT: SaltString = get_salt();
}

fn get_salt() -> SaltString {
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
}

async fn init_data<'key>(salt: &'key SaltString) -> MainData<'key> {
    let pool = new_db_pool().await.expect("Failed to create pool");
    let mut data = MainData::new(pool, salt);
    reload_session_cache(&data.pool, &mut data.cache.sessions)
        .await
        .expect("failed to reload_session_cache");
    data
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    // Check Cache Dir
    let cache_dir =
        std::env::var("CACHE_DIR").expect("missing CACHE_DIR environment variable (hint: './tmp')");
    let cache_path = Path::new(&cache_dir);
    if cache_path.exists() == false {
        std::fs::DirBuilder::new()
            .recursive(true)
            .create(cache_path)
            .expect("failed to create CACHE_DIR");
    }

    let data = web::Data::new(init_data(&SALT).await);
    let chat = web::Data::new(chat::ChatServer::new().start());
    let s3 = web::Data::new(s3::s3_test_client());

    // Start HTTP server
    HttpServer::new(move || {
        // Authentication policy
        let policy = CookieIdentityPolicy::new(&[0; 32]) // TODO: Set a 32B Salt
            .name("auth")
            .secure(true);

        App::new()
            .app_data(data.clone())
            .app_data(chat.clone())
            .app_data(s3.clone())
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
            .service(member::view_members)
            .service(filesystem::put_file)
            .service(post::edit_post)
            .service(post::update_post)
            .service(post::view_post_by_id)
            .service(post::view_post_in_thread)
            .service(forum::create_thread)
            .service(forum::view_forum)
            .service(frontend::css::view_css)
            .service(thread::create_reply)
            .service(thread::view_thread)
            .service(thread::view_thread_page)
            .service(web::resource("/chat").to(hub::chat_route))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
