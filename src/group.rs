use crate::orm::{groups, user_groups};
use crate::user::ClientUser;
use sea_orm::entity::prelude::{DeriveActiveEnum, EnumIter};
use sea_orm::{entity::*, query::*, DatabaseConnection, FromQueryResult};

/// Value set for a single permission.
/// Compatible with sea_orm enum type.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "group_type")]
pub enum GroupType {
    /// Not a system group (may be deleted).
    #[sea_orm(string_value = "normal")]
    Normal,
    /// System group for any anonymous connection (i.e. Tor)
    #[sea_orm(string_value = "system_anon")]
    SystemAnon,
    /// System group for guests and unconfirmed accounts.
    #[sea_orm(string_value = "system_guest")]
    SystemGuest,
    /// System group for signed-in, confirmed users.
    #[sea_orm(string_value = "system_user")]
    SystemUser,
}

/// Returns groups which apply to user/guest based on the connection.
pub async fn get_group_ids_for_client(
    db: &DatabaseConnection,
    client: &Option<ClientUser>,
) -> Vec<i32> {
    #[derive(FromQueryResult)]
    pub struct GroupId {
        pub id: i32,
    }

    match client {
        // Select `user_groups` where user_id is our client user.
        Some(user) => match user_groups::Entity::find()
            .select_only()
            .column_as(user_groups::Column::GroupId, "id")
            .filter(user_groups::Column::UserId.eq(user.id))
            .into_model::<GroupId>()
            .all(db)
            .await
        {
            Ok(group_result) => group_result.iter().map(|group| group.id).collect(),
            Err(e) => {
                log::warn!("DbErr pulling user_groups for client: {:?}", e);
                Vec::new()
            }
        },
        // Select `groups` id for the system guest type.
        None => match groups::Entity::find()
            .select_only()
            .column(groups::Column::Id)
            .filter(groups::Column::GroupType.eq(GroupType::SystemGuest))
            .into_model::<GroupId>()
            .all(db)
            .await
        {
            Ok(group_result) => group_result.iter().map(|group| group.id).collect(),
            Err(e) => {
                log::warn!("DbErr pulling groups for guest: {:?}", e);
                Vec::new()
            }
        },
    }
}
