use crate::attachment::AttachmentSize;
use crate::orm::{attachments, user_avatars, user_names, users};
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

impl ClientUser {
    pub async fn fetch_by_user_id(db: &DatabaseConnection, id: i32) -> Option<Self> {
        users::Entity::find_by_id(id)
            .select_only()
            .column(users::Column::Id)
            .left_join(user_names::Entity)
            .column(user_names::Column::Name)
            .into_model::<ClientUser>()
            .one(db)
            .await
            .unwrap_or(None)
    }
}

pub fn find_also_user<E, C>(sel: Select<E>, col: C) -> SelectTwo<E, users::Entity>
where
    E: EntityTrait<Column = C>,
    C: IntoSimpleExpr + ColumnTrait,
{
    sel.select_also(users::Entity)
        .join(
            JoinType::LeftJoin,
            E::belongs_to(users::Entity)
                .from(col)
                .to(users::Column::Id)
                .into(),
        )
        .join(JoinType::LeftJoin, users::Relation::UserName.def())
        .column_as(user_names::Column::Name, "B_name")
        .join(JoinType::LeftJoin, users::Relation::UserAvatar.def())
        .join(
            JoinType::LeftJoin,
            user_avatars::Relation::Attachments.def(),
        )
        .column_as(attachments::Column::Filename, "B_avatar_filename")
        .column_as(attachments::Column::FileHeight, "B_avatar_height")
        .column_as(attachments::Column::FileWidth, "B_avatar_width")
}

/// A struct to hold all information for a user, including relational information.
#[derive(Clone, Debug, FromQueryResult)]
pub struct Profile {
    pub id: i32,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub password_cipher: crate::orm::users::Cipher,
    pub avatar_filename: Option<String>,
    pub avatar_height: Option<i32>,
    pub avatar_width: Option<i32>,
}

impl Profile {
    /// Returns a fully qualified user profile by id.
    pub async fn get_by_id(
        db: &DatabaseConnection,
        id: i32,
    ) -> Result<Option<Self>, sea_orm::DbErr> {
        users::Entity::find_by_id(id)
            .left_join(user_names::Entity)
            .column_as(user_names::Column::Name, "name")
            .left_join(attachments::Entity)
            .column_as(attachments::Column::Filename, "avatar_filename")
            .column_as(attachments::Column::FileHeight, "avatar_height")
            .column_as(attachments::Column::FileWidth, "avatar_width")
            .into_model::<Self>()
            .one(db)
            .await
    }

    /// Provides semantically correct HTML for an avatar.
    pub fn get_avatar_html(&self, size: AttachmentSize) -> Option<String> {
        if let (Some(filename), Some(width), Some(height)) = (
            self.avatar_filename.as_ref(),
            self.avatar_width,
            self.avatar_width,
        ) {
            Some(crate::attachment::get_avatar_html(
                &filename,
                (width, height),
                size,
            ))
        } else {
            None
        }
    }

    /// Provides a URL token for this resource.
    pub fn get_url_token(&self) -> UrlToken<'static> {
        UrlToken {
            id: Some(self.id),
            name: self.name.to_owned(),
            base_url: RESOURCE_URL,
            class: "username",
        }
    }
}

pub async fn get_user_id_from_name(db: &DatabaseConnection, name: &str) -> Option<i32> {
    user_names::Entity::find()
        .filter(user_names::Column::Name.eq(name))
        .one(db)
        .await
        .unwrap_or(None)
        .map(|user_name| user_name.user_id)
}
