use super::orm::chat_message;
use crate::web::chat::message::ClientMessage;
use redis::{aio::MultiplexedConnection as RedisConnection, AsyncCommands, RedisError};
use sea_orm::{entity::*, prelude::*, query::*, DatabaseConnection, QueryFilter};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn get_chat_room_history(
    db: &DatabaseConnection,
    id: &u32,
    count: usize,
) -> Vec<chat_message::Model> {
    chat_message::Entity::find()
        .filter(chat_message::Column::RoomId.eq(id.to_owned()))
        .order_by_desc(chat_message::Column::MessageId)
        .limit(count as u64)
        .all(db)
        .await
        .unwrap_or(Vec::default())
        .into_iter()
        .rev()
        .collect()
}

pub async fn insert_chat_message(
    message: &ClientMessage,
    db: &DatabaseConnection,
    redis: &mut RedisConnection,
) -> ClientMessage {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp = Decimal::new(timestamp.as_micros() as i64, 6);

    // insert chat message into database
    let model = chat_message::ActiveModel {
        message_text: Set(message.message.to_owned()),
        message_date: Set(timestamp),
        message_update: Set(timestamp),
        room_id: Set(message.room_id as u32),
        user_id: Set(Some(message.author.id as u32)),
        username: Set(message.author.username.to_owned()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to insert chat_messagemessage into XF database.");

    ClientMessage {
        id: message.id,
        room_id: message.room_id,
        message_id: model.message_id,
        author: message.author.to_owned(),
        message: message.message.to_owned(),
    }
}
