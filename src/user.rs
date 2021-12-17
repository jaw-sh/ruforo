use crate::orm::user_names;
use sea_orm::{entity::*, query::*, DatabaseConnection, FromQueryResult};

/// Represents information about this request's client.
#[derive(Clone, Debug)]
pub struct Client {
    pub user: Option<ClientUser>,
}

impl Client {
    pub fn new() -> Self {
        Self { user: None }
    }
}

/// A mini struct for holding only what information we need about a client.
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
}

/// Produces a client object for a specific identity.
// pub async fn get_client_from_identity(id: &Identity) -> Client {
//     Client {
//         user: match id.identity() {
//             Some(id) => match authenticate_by_uuid_string(id).await {
//                 Some((_uuid, session)) => users::Entity::find_by_id(session.user_id)
//                     .select_only()
//                     .column(users::Column::Id)
//                     .left_join(user_names::Entity)
//                     .column(user_names::Column::Name)
//                     .into_model::<ClientUser>()
//                     .one(get_db_pool())
//                     .await
//                     .unwrap_or(None),
//                 None => None,
//             },
//             None => None,
//         },
//     }
// }

pub async fn get_user_id_from_name(db: &DatabaseConnection, name: &str) -> Option<i32> {
    user_names::Entity::find()
        .filter(user_names::Column::Name.eq(name))
        .one(db)
        .await
        .unwrap_or(None)
        .map(|user_name| user_name.user_id)
}
