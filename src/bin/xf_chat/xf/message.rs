use super::orm::chat_message;
use ruforo::web::chat::implement;
use ruforo::web::chat::message;
use sea_orm::{entity::*, prelude::*, DatabaseConnection, QueryFilter};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub async fn delete_message(db: &DatabaseConnection, uuid: Uuid) {
    match chat_message::Entity::delete_many()
        .filter(chat_message::Column::MessageUuid.eq(uuid.to_string()))
        .exec(db)
        .await
    {
        Ok(_) => {}
        Err(err) => {
            log::warn!("Unable to delete XF chat message: {:?}", err);
        }
    }
}

pub async fn edit_message(
    db: &DatabaseConnection,
    uuid: Uuid,
    author: implement::Author,
    message: String,
) -> Option<implement::Message> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp = Decimal::new(timestamp.as_micros() as i64, 6);

    let model: chat_message::Model = match chat_message::Entity::find()
        .filter(chat_message::Column::MessageUuid.eq(uuid.to_string()))
        .one(db)
        .await
    {
        Ok(model) => match model {
            Some(model) => model,
            None => {
                log::warn!("No result on XF chat message for update: {:?}", uuid);
                return None;
            }
        },
        Err(err) => {
            log::warn!("Failed to select XF chat message for update: {:?}", err);
            return None;
        }
    };

    let mut active: chat_message::ActiveModel = model.into();
    active.message_text = Set(message);
    active.last_edit_date = Set(Some(timestamp));
    active.last_edit_user_id = Set(Some(author.id));

    match active.update(db).await {
        Ok(model) => Some(implement::Message::from(model)),
        Err(err) => {
            log::warn!("Failed to update XF chat message: {:?}", err);
            None
        }
    }
}

pub async fn get_message_with_author(
    db: &DatabaseConnection,
    uuid: Uuid,
) -> Option<(implement::Author, implement::Message)> {
    match chat_message::Entity::find()
        .filter(chat_message::Column::MessageUuid.eq(uuid.to_string()))
        .find_also_related(super::orm::user::Entity)
        .one(db)
        .await
    {
        Ok(Some((msg, user))) => {
            let author = match user {
                Some(user) => implement::Author {
                    id: user.user_id,
                    username: user.username.to_owned(),
                    avatar_url: super::session::avatar_uri(user.user_id, user.avatar_date, user.avatar_format.as_deref()),
                },
                None => implement::Author {
                    id: msg.user_id.unwrap_or(0),
                    username: msg.username.to_owned(),
                    avatar_url: String::new(),
                },
            };
            let message = implement::Message {
                message: msg.message_text.to_owned(),
                message_uuid: Uuid::parse_str(&msg.message_uuid).unwrap_or_default(),
                message_date: msg.message_date.try_into().unwrap(),
                message_edit_date: match msg.last_edit_date {
                    Some(date) => date.try_into().unwrap(),
                    None => 0,
                },
                room_id: msg.room_id,
                user_id: msg.user_id.unwrap_or(0),
            };
            Some((author, message))
        }
        Ok(None) => None,
        Err(err) => {
            log::warn!("Error pulling XF chat message with author by UUID: {:?}", err);
            None
        }
    }
}

pub async fn get_message(db: &DatabaseConnection, uuid: Uuid) -> Option<implement::Message> {
    match chat_message::Entity::find()
        .filter(chat_message::Column::MessageUuid.eq(uuid.to_string()))
        .one(db)
        .await
    {
        Ok(res) => res.map(implement::Message::from),
        Err(err) => {
            log::warn!("Error pulling XF chat message by UUID: {:?}", err);
            None
        }
    }
}

pub async fn insert_chat_message(
    db: &DatabaseConnection,
    message: &message::Post,
) -> anyhow::Result<implement::Message, anyhow::Error> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp = Decimal::new(timestamp.as_micros() as i64, 6);

    // insert chat message into database
    let result = chat_message::ActiveModel {
        message_text: Set(message.message.to_owned()),
        message_uuid: Set(message.message_uuid.to_string()),
        message_date: Set(timestamp),
        message_update: Set(timestamp),
        room_id: Set(message.room_id as u32),
        user_id: Set(Some(message.session.id)),
        username: Set(message.session.username.to_owned()),
        ..Default::default()
    }
    .insert(db)
    .await;

    match result {
        Ok(model) => Ok(implement::Message::from(model)),
        Err(err) => Err(anyhow::Error::new(err)),
    }
}
