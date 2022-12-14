//! SeaORM Entity. Generated by sea-orm-codegen 0.4.0

use crate::permission::flag::Flag;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "permission_values")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub permission_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub collection_id: i32,
    #[sea_orm(rs_type = "i32", db_type = "Enum")]
    pub value: Flag,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::permission_collections::Entity",
        from = "Column::CollectionId",
        to = "super::permission_collections::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    PermissionCollections,
    #[sea_orm(
        belongs_to = "super::permissions::Entity",
        from = "Column::PermissionId",
        to = "super::permissions::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Permissions,
}

impl Related<super::permission_collections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PermissionCollections.def()
    }
}

impl Related<super::permissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Permissions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
