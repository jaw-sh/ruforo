mod xf;

use actix::Actor;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use env_logger::Env;
use sea_orm::{ConnectOptions, Database};
use std::sync::Arc;
use std::time::Duration;

// Binary made compatible with XF2.
// Temporary part of the project.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().expect("DotEnv failed to initialize.");
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let mysql = {
        let mut options = ConnectOptions::new(
            std::env::var("XF_MYSQL_URL").expect("XF_MYSQL_URL required for chat binary."),
        );
        options
            .max_connections(1024)
            .min_connections(16)
            .connect_timeout(Duration::from_secs(1))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true);
        Database::connect(options)
            .await
            .expect("XF MySQL connection failed.")
    };
    let (redis_cfg, redis) = match redis::Client::open(
        std::env::var("XF_REDIS_URL").expect("XF_REDIS_URL required for chat binary."),
    ) {
        Ok(client) => (
            client.clone(),
            client
                .get_multiplexed_async_connection()
                .await
                .expect("XF Redis connection failed."),
        ),
        Err(err) => {
            panic!("{:?}", err);
        }
    };

    let layer = Arc::new(xf::XfLayer { db: mysql.clone() });
    let chat = ruforo::web::chat::server::ChatServer::new(layer.clone())
        .await
        .start();

    HttpServer::new(move || {
        // Downcast so we can store in app_data
        // See: https://stackoverflow.com/questions/65645622/how-do-i-pass-a-trait-as-application-data-to-actix-web
        use ruforo::web::chat::implement::ChatLayer;
        let layer_data: Data<Arc<dyn ChatLayer>> = Data::new(layer.clone());

        App::new()
            .app_data(layer_data)
            .app_data(Data::new(redis_cfg.clone()))
            .app_data(Data::new(redis.clone()))
            .app_data(Data::new(mysql.clone()))
            .app_data(chat.clone())
            .service(ruforo::web::chat::view_chat_socket)
            .service(ruforo::web::chat::view_chat_shim)
    })
    .bind(std::env::var("CHAT_WS_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_owned()))?
    .run()
    .await
}
