use super::orm::{chat_message, chat_room, user};
use super::session::avatar_uri;
use ruforo::web::chat::implement;
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
    id: u32,
    count: usize,
) -> Vec<(implement::Author, implement::Message)> {
    chat_message::Entity::find()
        .filter(chat_message::Column::RoomId.eq(id as u32))
        .order_by_desc(chat_message::Column::MessageId)
        .limit(count as u64)
        .find_also_related(user::Entity)
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .rev()
        .map(|(message, user)| {
            (
                match user {
                    Some(user) => implement::Author {
                        id: user.user_id,
                        username: user.username.to_owned(),
                        avatar_url: avatar_uri(user.user_id, user.avatar_date),
                    },
                    None => implement::Author {
                        id: 0,
                        username: "Guest".to_owned(),
                        avatar_url: String::new(),
                    },
                },
                implement::Message {
                    message: message.message_text.to_owned(),
                    message_id: message.message_id,
                    message_date: message.message_date.try_into().unwrap(),
                    message_edit_date: match message.last_edit_date {
                        Some(date) => date.try_into().unwrap(),
                        None => 0,
                    },
                    room_id: message.room_id,
                    user_id: message.user_id.unwrap_or(0),
                },
            )
        })
        .collect()
}
