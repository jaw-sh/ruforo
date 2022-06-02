use super::orm::chat_room;
use sea_orm::entity::prelude::*;
use sea_orm::{query::*, DatabaseConnection};

pub async fn get_room_list(db: &DatabaseConnection) -> Vec<chat_room::Model> {
    chat_room::Entity::find()
        .order_by_asc(chat_room::Column::DisplayOrder)
        .all(db)
        .await
        .expect("Unable to fetch room list")
}
