use crate::frontend::TemplateToPubResponse;
use crate::init::get_db_pool;
use crate::orm::{user_names, users};
use crate::user::UserProfile;
use actix_web::{error, get, Responder};
use askama_actix::Template;
use sea_orm::{entity::*, query::*};

#[derive(Template)]
#[template(path = "members.html")]
pub struct MembersTemplate {
    pub users: Vec<crate::user::UserProfile>,
}

#[get("/members")]
pub async fn view_members() -> impl Responder {
    match users::Entity::find()
        .left_join(user_names::Entity)
        .column_as(user_names::Column::Name, "name")
        .into_model::<UserProfile>()
        .all(get_db_pool())
        .await
    {
        Ok(users) => Ok(MembersTemplate { users }.to_pub_response()),
        Err(e) => {
            log::error!("error {:?}", e);
            Err(error::ErrorInternalServerError("Couldn't load users"))
        }
    }
}
