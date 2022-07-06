use actix::Actor;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::http::StatusCode;
use actix_web::middleware::{ErrorHandlers, Logger};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use env_logger::Env;
use rand::{distributions::Alphanumeric, Rng};
use ruforo::db::{get_db_pool, init_db};
use ruforo::middleware::ClientCtx;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_lib_mods();
    init_our_mods();
    init_db(std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.")).await;

    let permissions = ruforo::permission::new()
        .await
        .expect("Permission System failed to initialize.");

    let secret_key = match std::env::var("SECRET_KEY") {
        Ok(key) => Key::from(key.as_bytes()),
        Err(err) => {
            let random_string: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(128)
                .map(char::from)
                .collect();
            log::warn!("SECRET_KEY was invalid. Reason: {:?}\r\nThis means the key used for signing session cookies will invalidate every time the application is restarted. A secret key must be at least 64 bytes to be accepted.\r\n\r\nNeed a key? How about:\r\n{}", err, random_string);
            Key::from(random_string.as_bytes())
        }
    };

    let layer = Arc::new(ruforo::web::chat::implement::default::Layer {
        db: get_db_pool().to_owned(),
    });
    let chat = ruforo::web::chat::server::ChatServer::new(layer.clone())
        .await
        .start();

    HttpServer::new(move || {
        let layer_data: Data<Arc<dyn ruforo::web::chat::implement::ChatLayer>> =
            Data::new(layer.clone());

        // Order of middleware IS IMPORTANT and is in REVERSE EXECUTION ORDER.
        // However, services are read top->down, higher traffic routes should be
        // placed higher
        App::new()
            .app_data(Data::new(get_db_pool()))
            .app_data(Data::new(permissions.clone()))
            .app_data(layer_data)
            .app_data(chat.clone())
            .wrap(
                ErrorHandlers::new()
                    .handler(StatusCode::NOT_FOUND, ruforo::web::error::render_404)
                    .handler(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ruforo::web::error::render_500,
                    ),
            )
            .wrap(ClientCtx::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .wrap(Logger::new("%a %{User-Agent}i"))
            .configure(ruforo::web::configure)
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
    ruforo::global::init();
    ruforo::session::init();
    ruforo::filesystem::init();
}
