use super::orm::chat_message;
use ruforo::web::chat::message;
use sea_orm::{entity::*, prelude::*, DatabaseConnection};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn insert_chat_message(
    db: &DatabaseConnection,
    message: &message::ClientMessage,
) -> message::ClientMessage {
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
    .expect("Failed to insert chat_message into XF database.");

    message::ClientMessage {
        id: message.id,
        room_id: message.room_id,
        message_id: model.message_id,
        message_date: model.message_date.try_into().unwrap(),
        author: message.author.to_owned(),
        message: message.message.to_owned(),
        sanitized: false,
    }
}
