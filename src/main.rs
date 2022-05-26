mod account;
mod asset;
mod attachment;
mod auth_2fa;
mod bbcode;
mod chat;
mod create_user;
mod ffmpeg;
mod filesystem;
mod forum;
mod global;
mod group;
mod hub;
mod index;
mod login;
mod logout;
mod member;
mod middleware;
mod orm;
mod permission;
mod post;
mod s3;
mod session;
mod template;
mod thread;
mod ugc;
mod url;
mod user;
mod web;

extern crate dotenv;
extern crate ffmpeg_next;

use crate::middleware::ClientCtx;
use crate::session::{get_sess, reload_session_cache};
use actix::Actor;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use env_logger::Env;
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::path::Path;
use std::time::Duration;

static DB_POOL: OnceCell<DatabaseConnection> = OnceCell::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init();
    init_db().await;
    start().await
}

pub fn init() {
    dotenv::dotenv().expect("DotEnv failed to initialize.");
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    ffmpeg_next::init().expect("FFMPEG failed to initialize.");

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

    global::init();
    session::init();
    filesystem::init();
}

#[inline(always)]
pub fn get_db_pool() -> &'static DatabaseConnection {
    unsafe { DB_POOL.get_unchecked() }
}

/// This MUST be called before calling get_db_pool, which is unsafe code
pub async fn init_db() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true);

    let pool = Database::connect(opt)
        .await
        .expect("Database connection was not established.");
    DB_POOL.set(pool).unwrap();

    reload_session_cache(get_sess())
        .await
        .expect("failed to reload_session_cache");
}

/// This MUST NOT be called before init_db()
///
/// TODO break up into chunks
pub async fn start() -> std::io::Result<()> {
    use actix_web::middleware::Logger;
    use actix_web::web::{resource, Data};
    use actix_web::{cookie::Key, App, HttpServer};

    let chat = chat::ChatServer::new().start();
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
            .app_data(Data::new(chat.clone()))
            .app_data(Data::new(permissions.clone()))
            .wrap(ClientCtx::new())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .wrap(Logger::new("%a %{User-Agent}i"))
            .configure(web::configure)
            .service(resource("/chat").to(crate::hub::chat_route))
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
