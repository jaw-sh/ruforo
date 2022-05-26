pub mod account;
pub mod asset;
pub mod error;
pub mod forum;
pub mod index;
pub mod login;
pub mod logout;
pub mod member;
pub mod post;
pub mod thread;

/// Configures the web app by adding services from each web file.
///
/// @see https://docs.rs/actix-web/4.0.1/actix_web/struct.App.html#method.configure
pub fn configure(conf: &mut actix_web::web::ServiceConfig) {
    use actix_web::http::StatusCode;
    use actix_web::middleware::ErrorHandlers;

    // Descending order. Order is important.
    // Route resolution will stop at the first match.
    index::configure(conf);
    account::configure(conf);
    asset::configure(conf);
    forum::configure(conf);
    login::configure(conf);
    logout::configure(conf);
    member::configure(conf);
    post::configure(conf);
    thread::configure(conf);

    conf.service(crate::create_user::create_user_get)
        .service(crate::create_user::create_user_post)
        .service(crate::auth_2fa::user_enable_2fa)
        .service(crate::filesystem::view_file_ugc)
        .service(crate::filesystem::view_file_canonical)
        .service(crate::filesystem::post_file_hash)
        .service(crate::filesystem::put_file)
        .service(crate::session::view_task_expire_sessions);
}
