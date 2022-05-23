use crate::attachment::AttachmentSize;
use crate::init::get_db_pool;
use crate::orm::{attachments, user_names, users};
use crate::url::UrlToken;
use sea_orm::{entity::*, query::*, DatabaseConnection, FromQueryResult};

/// Base URL fragment for resource.
pub static RESOURCE_URL: &str = "members";

/// ORM user data for the request cycle.
#[derive(Clone, Debug, FromQueryResult)]
pub struct ClientUser {
    pub id: i32,
    pub name: String,
}

/// A struct to hold all information for a user, including relational information.
#[derive(Clone, Debug, FromQueryResult)]
pub struct UserProfile {
    pub id: i32,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub password_cipher: crate::orm::users::Cipher,
    pub avatar_filename: Option<String>,
    pub avatar_height: Option<i32>,
    pub avatar_width: Option<i32>,
}

impl UserProfile {
    pub fn get_url_token(&self) -> UrlToken<'static> {
        UrlToken {
            id: Some(self.id),
            name: self.name.to_owned(),
            base_url: RESOURCE_URL,
            class: "username",
        }
    }
}

pub fn get_avatar_html_for_user(user: &UserProfile, size: AttachmentSize) -> Option<String> {
    if user.avatar_filename.is_some() && user.avatar_width.is_some() && user.avatar_height.is_some()
    {
        Some(crate::attachment::get_avatar_html(
            &user.avatar_filename.to_owned().unwrap(),
            (
                &user.avatar_width.to_owned().unwrap(),
                &user.avatar_height.to_owned().unwrap(),
            ),
            size,
        ))
    } else {
        None
    }
}

pub async fn get_profile_by_id(id: i32) -> Option<UserProfile> {
    users::Entity::find_by_id(id)
        .left_join(user_names::Entity)
        .column_as(user_names::Column::Name, "name")
        .left_join(attachments::Entity)
        .column_as(attachments::Column::Filename, "avatar_filename")
        .column_as(attachments::Column::FileHeight, "avatar_height")
        .column_as(attachments::Column::FileWidth, "avatar_width")
        .into_model::<UserProfile>()
        .one(get_db_pool())
        .await
        .map_err(|e| log::error!("get_profile: {}", e))
        .unwrap_or(None)
}

pub async fn get_user_id_from_name(db: &DatabaseConnection, name: &str) -> Option<i32> {
    user_names::Entity::find()
        .filter(user_names::Column::Name.eq(name))
        .one(db)
        .await
        .unwrap_or(None)
        .map(|user_name| user_name.user_id)
}
