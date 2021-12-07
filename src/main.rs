extern crate dotenv;
#[macro_use]
extern crate lazy_static;

use crate::session::{new_db_pool, reload_session_cache, MainData};
use actix_session::CookieSession;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use env_logger::Env;

mod chat;
mod users;
mod create_user;
mod forum;
pub mod frontend;
mod index;
mod login;
mod middleware;
pub mod orm;
mod post;
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
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let data = web::Data::new(init_data(&SALT).await);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // There is theoretically a way to enforce trailing slashes, but this fuckes
            // with pseudofiles like style.css
            .app_data(data.clone())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(
                CookieSession::signed(&[0; 32]) // <- create cookie based session middleware
                    .secure(false),
            )
            .wrap(middleware::AppendContext {})
            // https://www.restapitutorial.com/lessons/httpmethods.html
            // GET    edit_ (get edit form)
            // PATCH  update_ (apply edit)
            // GET    view_ (read/view/render entity)
            // Note: PUT and PATCH were added, removed, and re-added(?) to the HTML5 spec for <form method="">
            .service(index::view_index)
            .service(create_user::create_user_get)
            .service(create_user::create_user_post)
            .service(login::login_get)
            .service(login::login_post)
            .service(users::list_users)
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
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
