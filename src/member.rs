use crate::init::get_db_pool;
use crate::middleware::ClientCtx;
use crate::orm::{user_names, users};
use crate::user::UserProfile;
use actix_web::{error, get, Responder};
use askama_actix::{Template, TemplateToResponse};
use sea_orm::{entity::*, query::*};

#[derive(Template)]
#[template(path = "members.html")]
pub struct MembersTemplate {
    pub client: ClientCtx,
    pub users: Vec<UserProfile>,
}

#[get("/members")]
pub async fn view_members(client: ClientCtx) -> impl Responder {
    match users::Entity::find()
        .left_join(user_names::Entity)
        .column_as(user_names::Column::Name, "name")
        .into_model::<UserProfile>()
        .all(get_db_pool())
        .await
    {
        Ok(users) => Ok(MembersTemplate { client, users }.to_response()),
        Err(e) => {
            log::error!("error {:?}", e);
            Err(error::ErrorInternalServerError("Couldn't load users"))
        }
    }
}
