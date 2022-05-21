//! SeaORM Entity. Generated by sea-orm-codegen 0.4.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "groups")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub label: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_groups::Entity")]
    UserGroups,
}

impl Related<super::user_groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserGroups.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
