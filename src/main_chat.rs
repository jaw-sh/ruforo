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
    use actix::Actor;
    use actix_web::web::{resource, Data};
    use actix_web::{App, HttpServer};
    use env_logger::Env;
    use sea_orm::{ConnectOptions, Database};
    use std::time::Duration;

    dotenv::dotenv().expect("DotEnv failed to initialize.");
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let mysql = {
        let mut options = ConnectOptions::new("mysql://john:john@localhost/xenforo".to_owned());
        options
            .max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(1))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true);
        Database::connect(options).await.unwrap()
    };
    let redis = match redis::Client::open("redis://127.0.0.1/") {
        Ok(client) => client,
        Err(err) => {
            panic!("{:?}", err);
        }
    };
    let chat = crate::web::chat::server::ChatServer::new().start();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(redis.clone()))
            .app_data(Data::new(mysql.clone()))
            .app_data(chat.clone())
            .service(resource("/chat").to(crate::web::chat::service))
            .service(crate::web::chat::view_chat)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
