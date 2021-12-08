use crate::frontend::TemplateToPubResponse;
use crate::orm::users;
use crate::session::MainData;
use actix_web::{error, get, web, Responder};
use askama_actix::Template;
use sea_orm::entity::*;

#[derive(Template)]
#[template(path = "members.html")]
pub struct MembersTemplate {
    pub users: Vec<users::Model>,
}

#[get("/members")]
pub async fn view_members(data: web::Data<MainData<'static>>) -> impl Responder {
    match users::Entity::find().all(&data.pool).await {
        Ok(users) => {
            return Ok(MembersTemplate { users }.to_pub_response());
        }
        Err(e) => {
            log::error!("error {:?}", e);
            return Err(error::ErrorInternalServerError("Couldn't load users"));
        }
    }
}
