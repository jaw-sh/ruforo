use super::orm::{chat_message, chat_room, user};
use super::session::avatar_uri;
use ruforo::web::chat::{implement::Author, message};
use sea_orm::{entity::*, query::*, DatabaseConnection, QueryFilter};

pub async fn get_room_list(db: &DatabaseConnection) -> Vec<chat_room::Model> {
    chat_room::Entity::find()
        .order_by_asc(chat_room::Column::DisplayOrder)
        .all(db)
        .await
        .expect("Unable to fetch room list")
}

pub async fn get_room_history(
    db: &DatabaseConnection,
    id: usize,
    count: usize,
) -> Vec<message::ClientMessage> {
    chat_message::Entity::find()
        .filter(chat_message::Column::RoomId.eq(id as u32))
        .order_by_desc(chat_message::Column::MessageId)
        .limit(count as u64)
        .find_also_related(user::Entity)
        .all(db)
        .await
        .unwrap_or(Vec::default())
        .into_iter()
        .rev()
        .map(|(message, user)| message::ClientMessage {
            id: 0,
            message_id: message.message_id,
            message_date: message.message_date.try_into().unwrap(),
            message: message.message_text.to_owned(),
            sanitized: false,
            room_id: message.room_id as usize,
            author: match user {
                Some(user) => Author {
                    id: user.user_id as u32,
                    username: user.username.to_owned(),
                    avatar_url: avatar_uri(user.user_id, user.avatar_date),
                },
                None => Author {
                    id: 0,
                    username: "Guest".to_owned(),
                    avatar_url: String::new(),
                },
            },
        })
        .collect()
}
