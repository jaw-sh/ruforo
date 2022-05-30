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

    HttpServer::new(move || {
        let redis = match redis::Client::open("redis://127.0.0.1/") {
            Ok(client) => client,
            Err(err) => {
                panic!("{:?}", err);
            }
        };
        let chat = crate::web::chat::server::ChatServer::new().start();

        App::new()
            .app_data(Data::new(redis))
            .app_data(Data::new(chat))
            .service(resource("/chat").to(crate::web::chat::service))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
