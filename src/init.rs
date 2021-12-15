extern crate dotenv;
extern crate ffmpeg_next;

use crate::{middleware::AppendContext, post};
use actix::Actor;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use std::path::Path;

/// TODO break up into chunks
pub async fn init() -> std::io::Result<()> {
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

    let data = web::Data::new(crate::session::init_data().await);
    let chat = web::Data::new(crate::chat::ChatServer::new().start());

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
            .service(crate::index::view_index)
            .service(crate::create_user::create_user_get)
            .service(crate::create_user::create_user_post)
            .service(crate::login::view_login)
            .service(crate::login::post_login)
            .service(crate::logout::view_logout)
            .service(crate::member::view_members)
            .service(crate::filesystem::view_file_ugc)
            .service(crate::filesystem::view_file_canonical)
            .service(crate::filesystem::put_file)
            .service(post::delete_post)
            .service(post::destroy_post)
            .service(post::edit_post)
            .service(post::update_post)
            .service(post::view_post_by_id)
            .service(post::view_post_in_thread)
            .service(crate::forum::create_thread)
            .service(crate::forum::view_forum)
            .service(crate::frontend::css::view_css)
            .service(crate::thread::create_reply)
            .service(crate::thread::view_thread)
            .service(crate::thread::view_thread_page)
            .service(web::resource("/chat").to(crate::hub::chat_route))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
