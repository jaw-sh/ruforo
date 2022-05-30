mod attachment;
mod auth_2fa;
mod bbcode;
mod create_user;
mod db;
mod ffmpeg;
mod filesystem;
mod global;
mod group;
mod middleware;
mod orm;
mod permission;
mod s3;
mod session;
mod template;
mod ugc;
mod url;
mod user;
mod web;

extern crate dotenv;
extern crate ffmpeg_next;

use crate::db::{get_db_pool, init_db};
use crate::middleware::ClientCtx;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_lib_mods();
    init_our_mods();
    init_db(std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.")).await;

    use actix_web::cookie::Key;
    use actix_web::http::StatusCode;
    use actix_web::middleware::{ErrorHandlers, Logger};
    use actix_web::web::Data;
    use actix_web::{App, HttpServer};

    // TODO: Chat stuff is not being used right now.
    //use actix::Actor;
    //let chat = chat::ChatServer::new().start();
    //.app_data(Data::new(chat.clone()))
    //.service(resource("/chat").to(crate::hub::chat_route))

    let permissions = crate::permission::new()
        .await
        .expect("Permission System failed to initialize.");
    let secret_key = Key::generate(); // TODO: Should be from .env file

    HttpServer::new(move || {
        // Order of middleware IS IMPORTANT and is in REVERSE EXECUTION ORDER.
        // However, services are read top->down, higher traffic routes should be
        // placed higher
        App::new()
            .app_data(Data::new(get_db_pool()))
            .app_data(Data::new(permissions.clone()))
            .wrap(ErrorHandlers::new().handler(
                StatusCode::INTERNAL_SERVER_ERROR,
                crate::web::error::render_500,
            ))
            .wrap(ClientCtx::new())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .wrap(Logger::new("%a %{User-Agent}i"))
            .configure(web::configure)
    })
    // https://www.restapitutorial.com/lessons/httpmethods.html
    // GET    edit_ (get edit form)
    // PATCH  update_ (apply edit)
    // GET    view_ (read/view/render entity)
    // Note: PUT and PATCH were added, removed, and re-added(?) to the HTML5 spec for <form method="">
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

/// Initialize third party crates we rely on but don't have control over.
pub fn init_lib_mods() {
    // This should be calls to crates without any transformative work applied.
    dotenv::dotenv().expect("DotEnv failed to initialize.");
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    ffmpeg_next::init().expect("FFMPEG failed to initialize.");
}

/// Initialize all local mods.
/// Panics
pub fn init_our_mods() {
    // This should be a list of simple function calls.
    // Each module should work mostly independent of others.
    // This way, we can unit test individual modules without loading the entire application.
    global::init();
    session::init();
    filesystem::init();
}
