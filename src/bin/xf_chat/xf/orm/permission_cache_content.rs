//! SeaORM Entity. Generated by sea-orm-codegen 0.4.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "xf_permission_cache_content")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub permission_combination_id: u32,
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Custom(\"VARBINARY(25)\".to_owned())"
    )]
    pub content_type: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub content_id: u32,
    #[sea_orm(column_type = "Text")]
    pub cache_value: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::permission_combination::Entity",
        from = "Column::PermissionCombinationId",
        to = "super::permission_combination::Column::PermissionCombinationId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    PermissionCacheContent,
}

impl Related<super::permission_combination::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PermissionCacheContent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
