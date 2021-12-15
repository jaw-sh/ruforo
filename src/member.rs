use actix_web::{error, get, web, Responder};
use askama_actix::Template;
use ruforo::frontend::TemplateToPubResponse;
use ruforo::orm::{user_names, users};
use ruforo::session::MainData;
use ruforo::user::UserProfile;
use sea_orm::{entity::*, query::*};

#[derive(Template)]
#[template(path = "members.html")]
pub struct MembersTemplate {
    pub users: Vec<ruforo::user::UserProfile>,
}

#[get("/members")]
pub async fn view_members(data: web::Data<MainData<'static>>) -> impl Responder {
    match users::Entity::find()
        .left_join(user_names::Entity)
        .column_as(user_names::Column::Name, "name")
        .into_model::<UserProfile>()
        .all(&data.pool)
        .await
    {
        Ok(users) => Ok(MembersTemplate { users }.to_pub_response()),
        Err(e) => {
            log::error!("error {:?}", e);
            Err(error::ErrorInternalServerError("Couldn't load users"))
        }
    }
}
