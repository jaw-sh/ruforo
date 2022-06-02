use super::orm::chat_message;
use crate::web::chat::message::ClientMessage;
use redis::{aio::MultiplexedConnection as RedisConnection, AsyncCommands, RedisError};
use sea_orm::{entity::*, prelude::*, DatabaseConnection};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn insert_chat_message(
    message: ClientMessage,
    db: &DatabaseConnection,
    redis: &mut RedisConnection,
) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp = Decimal::new(timestamp.as_micros() as i64, 6);

    // insert chat message into database
    let model = chat_message::ActiveModel {
        message_text: Set(message.message),
        message_date: Set(timestamp),
        message_update: Set(timestamp),
        room_id: Set(message.room_id as u32),
        user_id: Set(Some(message.author.id as u32)),
        username: Set(message.author.username),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to insert chat_messagemessage into XF database.");

    // add to redis store for old chat
    let history_key = format!("xf[hb.chat.room{}.messages][1]", message.room_id);
    let history_entry: redis::RedisResult<String> = redis.get(&history_key).await;

    let mut message_history = match history_entry {
        Ok(message_ids) => match serde_php::from_bytes::<VecDeque<u32>>(message_ids.as_bytes()) {
            Ok(deser) => deser,
            Err(err) => {
                log::warn!("FAILED to deserialize {:?}", err);
                Default::default()
            }
        },
        Err(err) => {
            log::warn!("FAILED to pull from redis {:?}", err);
            Default::default()
        }
    };

    message_history.extend([model.message_id]);

    while message_history.len() > 20 {
        message_history.pop_front();
    }

    let _: Result<(), RedisError> = redis
        .set(
            &history_key,
            &serde_php::to_vec(&message_history).expect("Failed to serialize message history."),
        )
        .await;
}
