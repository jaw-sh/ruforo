extern crate dotenv;
extern crate ffmpeg_next;

use crate::session::{get_sess, reload_session_cache};
use crate::{
    chat, create_user, filesystem, forum, frontend, index, login, logout, member,
    middleware::ClientCtx, post, session, thread,
};
use actix::Actor;
use actix_session::CookieSession;
use actix_web::middleware::{Compat, Logger};
use actix_web::{web, App, HttpServer};
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

    session::init();
    filesystem::init();
    frontend::css::init();
}

/// This MUST NOT be called before init_db()
///
/// TODO break up into chunks
pub async fn start() -> std::io::Result<()> {
    let chat = web::Data::new(chat::ChatServer::new().start());
    HttpServer::new(move || {
        App::new()
            .service(web::scope("/static").service(frontend::css::view_css))
            .service(
                web::scope("")
                    .app_data(chat.clone())
                    // Order of middleware IS IMPORTANT and is in REVERSE EXECUTION ORDER.
                    .wrap(ClientCtx::new())
                    .wrap(Compat::new(
                        CookieSession::signed(&[0; 32])
                            .secure(false) // TODO make some sort of debug toggle for this
                            .name("sneedessions"),
                    ))
                    .service(index::view_index)
                    .service(create_user::create_user_get)
                    .service(create_user::create_user_post)
                    .service(login::view_login)
                    .service(login::post_login)
                    .service(logout::view_logout)
                    .service(member::view_members)
                    .service(filesystem::view_file_ugc)
                    .service(filesystem::view_file_canonical)
                    .service(filesystem::put_file)
                    .service(post::delete_post)
                    .service(post::destroy_post)
                    .service(post::edit_post)
                    .service(post::update_post)
                    .service(post::view_post_by_id)
                    .service(post::view_post_in_thread)
                    .service(forum::create_thread)
                    .service(forum::view_forum)
                    .service(thread::create_reply)
                    .service(thread::view_thread)
                    .service(thread::view_thread_page)
                    .service(session::view_task_expire_sessions)
                    .service(web::resource("/chat").to(crate::hub::chat_route)),
            )
            .wrap(Logger::new("%a %{User-Agent}i"))
        // https://www.restapitutorial.com/lessons/httpmethods.html
        // GET    edit_ (get edit form)
        // PATCH  update_ (apply edit)
        // GET    view_ (read/view/render entity)
        // Note: PUT and PATCH were added, removed, and re-added(?) to the HTML5 spec for <form method="">
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
