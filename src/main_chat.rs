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

mod compat;

extern crate dotenv;
extern crate ffmpeg_next;

// Binary made compatible with XF2.
// Temporary part of the project.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use crate::web::chat::server::ChatServer;
    use actix::Actor;
    use actix_web::web::{resource, Data};
    use actix_web::{App, HttpServer};
    use env_logger::Env;
    use sea_orm::{ConnectOptions, Database};
    use std::time::Duration;

    dotenv::dotenv().expect("DotEnv failed to initialize.");
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let mysql = {
        let mut options = ConnectOptions::new(
            std::env::var("XF_MYSQL_URL").expect("XF_MYSQL_URL required for chat binary."),
        );
        options
            .max_connections(256)
            .min_connections(5)
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
    let chat = ChatServer::new_from_xf(mysql.clone(), redis.clone())
        .await
        .start();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(redis_cfg.clone()))
            .app_data(Data::new(redis.clone()))
            .app_data(Data::new(mysql.clone()))
            .app_data(chat.clone())
            .service(resource("/chat").to(crate::web::chat::service))
            .service(crate::web::chat::view_chat)
    })
    .bind(std::env::var("CHAT_WS_BIND").unwrap_or("127.0.0.1:8080".to_owned()))?
    .run()
    .await
}
