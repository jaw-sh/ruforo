use crate::init::get_db_pool;
use crate::middleware::ClientCtx;
use crate::orm::{attachments, user_names, users};
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
        .left_join(attachments::Entity)
        .column_as(attachments::Column::Filename, "avatar_filename")
        .column_as(attachments::Column::FileHeight, "avatar_height")
        .column_as(attachments::Column::FileWidth, "avatar_width")
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
