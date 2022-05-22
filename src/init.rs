extern crate dotenv;
extern crate ffmpeg_next;

use crate::session::{get_sess, reload_session_cache};
use crate::{chat, filesystem, global, middleware::ClientCtx, session};
use actix::Actor;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::middleware::Logger;
use actix_web::{cookie::Key, web, App, HttpServer};
use env_logger::Env;
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::path::Path;
use std::time::Duration;

static DB_POOL: OnceCell<DatabaseConnection> = OnceCell::new();

#[inline(always)]
pub fn get_db_pool() -> &'static DatabaseConnection {
    unsafe { DB_POOL.get_unchecked() }
}

/// This MUST be called before calling get_db_pool, which is unsafe code
pub async fn init_db() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true);
    let pool = Database::connect(opt).await.expect("Failed to create pool");
    DB_POOL.set(pool).unwrap();

    reload_session_cache(get_sess())
        .await
        .expect("failed to reload_session_cache");
}

pub fn init() {
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

    global::init();
    session::init();
    filesystem::init();
}

/// This MUST NOT be called before init_db()
///
/// TODO break up into chunks
pub async fn start() -> std::io::Result<()> {
    let chat = web::Data::new(chat::ChatServer::new().start());
    let secret_key = Key::generate(); // TODO: Should be from .env file

    HttpServer::new(move || {
        // Order of middleware IS IMPORTANT and is in REVERSE EXECUTION ORDER.
        // However, services are read top->down, higher traffic routes should be
        // placed higher
        App::new()
            .app_data(chat.clone())
            .wrap(ClientCtx::new())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(crate::index::view_index)
            .service(crate::account::update_avatar)
            .service(crate::account::view_account)
            .service(crate::create_user::create_user_get)
            .service(crate::create_user::create_user_post)
            .service(crate::auth_2fa::user_enable_2fa)
            .service(crate::asset::view_file)
            .service(crate::login::view_login)
            .service(crate::login::post_login)
            .service(crate::logout::view_logout)
            .service(crate::member::view_member)
            .service(crate::member::view_members)
            .service(crate::filesystem::view_file_ugc)
            .service(crate::filesystem::view_file_canonical)
            .service(crate::filesystem::post_file_hash)
            .service(crate::filesystem::put_file)
            .service(crate::post::delete_post)
            .service(crate::post::destroy_post)
            .service(crate::post::edit_post)
            .service(crate::post::update_post)
            .service(crate::post::view_post_by_id)
            .service(crate::post::view_post_in_thread)
            .service(crate::forum::create_thread)
            .service(crate::forum::view_forums)
            .service(crate::forum::view_forum)
            .service(crate::thread::create_reply)
            .service(crate::thread::view_thread)
            .service(crate::thread::view_thread_page)
            .service(crate::session::view_task_expire_sessions)
            .service(web::resource("/chat").to(crate::hub::chat_route))
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
