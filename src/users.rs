use crate::session::MainData;
use actix_web::{error, get, web, Responder};
use crate::orm::users;
use askama_actix::Template;
use sea_orm::entity::*;
use crate::frontend::TemplateToPubResponse;

#[derive(Template)]
#[template(path = "users.html")]
pub struct UsersTemplate {
    pub users: Vec<users::Model>,
}

#[get("/users")]
pub async fn list_users(data: web::Data<MainData<'static>>) -> impl Responder {
    match users::Entity::find().all(&data.pool).await {
        Ok(users) => {
            return Ok(UsersTemplate{ users }.to_pub_response());
        }
        Err(e) => {
            log::error!("error {:?}", e);
            return Err(error::ErrorInternalServerError("Couldn't load users"));
        }
    }
}
