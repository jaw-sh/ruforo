/// Configures the web app
///
/// @see https://docs.rs/actix-web/4.0.1/actix_web/struct.App.html#method.configure
pub fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(crate::index::view_index)
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
        .service(crate::session::view_task_expire_sessions);
}
