//! SeaORM Entity. Generated by sea-orm-codegen 0.4.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "chat_messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub chat_room_id: i32,
    pub ugc_id: i32,
    pub user_id: Option<i32>,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::chat_rooms::Entity",
        from = "Column::ChatRoomId",
        to = "super::chat_rooms::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    ChatRooms,
    #[sea_orm(
        belongs_to = "super::ugc::Entity",
        from = "Column::UgcId",
        to = "super::ugc::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Ugc,
    #[sea_orm(
        belongs_to = "super::ugc_deletions::Entity",
        from = "Column::UgcId",
        to = "super::ugc_deletions::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    UgcDeletions,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::chat_rooms::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChatRooms.def()
    }
}

impl Related<super::ugc::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ugc.def()
    }
}

impl Related<super::ugc_deletions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UgcDeletions.def()
    }
}

impl Related<super::ugc_revisions::Entity> for Entity {
    fn to() -> RelationDef {
        super::ugc::Relation::UgcRevisions.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::ugc::Relation::ChatMessages.def().rev())
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
