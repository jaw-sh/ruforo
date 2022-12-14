//! SeaORM Entity. Generated by sea-orm-codegen 0.4.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "xf_user_ignored")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: u32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub ignored_user_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}
