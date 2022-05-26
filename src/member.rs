use crate::get_db_pool;
use crate::middleware::ClientCtx;
use crate::orm::{attachments, user_names, users};
use crate::user::UserProfile;
use actix_web::{error, get, web, Error, Responder};
use askama_actix::{Template, TemplateToResponse};
use sea_orm::{entity::*, query::*};

#[get("/members/{user_id}/")]
pub async fn view_member(
    client: ClientCtx,
    path: web::Path<(i32,)>,
) -> Result<impl Responder, Error> {
    #[derive(Template)]
    #[template(path = "member.html")]
    pub struct MemberTemplate {
        pub client: ClientCtx,
        pub user: UserProfile,
    }

    let user_id = path.into_inner().0;

    match users::Entity::find_by_id(user_id)
        .left_join(user_names::Entity)
        .column_as(user_names::Column::Name, "name")
        .left_join(attachments::Entity)
        .column_as(attachments::Column::Filename, "avatar_filename")
        .column_as(attachments::Column::FileHeight, "avatar_height")
        .column_as(attachments::Column::FileWidth, "avatar_width")
        .into_model::<UserProfile>()
        .one(get_db_pool())
        .await
    {
        Ok(user) => match user {
            Some(user) => Ok(MemberTemplate { client, user }.to_response()),
            None => Err(error::ErrorNotFound("User not found.")),
        },
        Err(e) => {
            log::error!("error {:?}", e);
            Err(error::ErrorInternalServerError("Couldn't load user."))
        }
    }
}

#[get("/members")]
pub async fn view_members(client: ClientCtx) -> impl Responder {
    #[derive(Template)]
    #[template(path = "members.html")]
    pub struct MembersTemplate {
        pub client: ClientCtx,
        pub users: Vec<UserProfile>,
    }

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
