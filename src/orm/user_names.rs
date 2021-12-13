//! SeaORM Entity. Generated by sea-orm-codegen 0.4.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_names")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: i32,
    #[sea_orm(column_type = "Text", unique)]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users,
    #[sea_orm(
        belongs_to = "super::user_names::Entity",
        from = "Column::UserId",
        to = "super::user_names::Column::UserId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    UserName,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
