use super::orm::{chat_message, user};
use ruforo::web::chat::implement;
use ruforo::web::chat::message;
use sea_orm::sea_query::Expr;
use sea_orm::{entity::*, prelude::*, query::*, Condition, DatabaseConnection, QueryFilter};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Soft-deletes a chat message, mirroring XF's Message::softDelete(): the row is
/// kept for moderation/evidence with deleted_date/deleted_user_id/deleted_username
/// set and message_update bumped. Permanent deletion is only done from XF.
pub async fn delete_message(db: &DatabaseConnection, uuid: Uuid, deleter: implement::Author) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp = Decimal::new(timestamp.as_micros() as i64, 6);

    match chat_message::Entity::update_many()
        .col_expr(chat_message::Column::DeletedDate, Expr::value(timestamp))
        .col_expr(chat_message::Column::DeletedUserId, Expr::value(deleter.id))
        .col_expr(chat_message::Column::DeletedUsername, Expr::value(deleter.username))
        .col_expr(chat_message::Column::MessageUpdate, Expr::value(timestamp))
        .filter(chat_message::Column::MessageUuid.eq(uuid.to_string()))
        .filter(chat_message::Column::DeletedDate.is_null())
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
        .filter(chat_message::Column::DeletedDate.is_null())
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
    active.last_edit_username = Set(Some(author.username));
    active.message_update = Set(timestamp);

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
        .filter(chat_message::Column::DeletedDate.is_null())
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
                recipient_id: msg.recipient_id,
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
        .filter(chat_message::Column::DeletedDate.is_null())
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
        recipient_id: Set(message.recipient_id),
        recipient_username: Set(message.recipient_username.to_owned()),
        ..Default::default()
    }
    .insert(db)
    .await;

    match result {
        Ok(model) => Ok(implement::Message::from(model)),
        Err(err) => Err(anyhow::Error::new(err)),
    }
}

/// Build an Author from an XF user row.
fn author_from_user(user: &user::Model) -> implement::Author {
    implement::Author {
        id: user.user_id,
        username: user.username.to_owned(),
        avatar_url: super::session::avatar_uri(
            user.user_id,
            user.avatar_date,
            user.avatar_format.as_deref(),
        ),
    }
}

/// Resolve a direct message recipient. Looks up by id when `user_id` is
/// nonzero, otherwise by username. Applies the same validity conditions as
/// session loading, so banned and unconfirmed accounts are not addressable.
pub async fn find_author(
    db: &DatabaseConnection,
    user_id: u32,
    username: &str,
) -> Option<implement::Author> {
    let filter = if user_id > 0 {
        user::Column::UserId.eq(user_id)
    } else if !username.is_empty() {
        user::Column::Username.eq(username.to_owned())
    } else {
        return None;
    };

    match user::Entity::find()
        .filter(filter)
        .filter(user::Column::UserState.eq("valid"))
        .filter(user::Column::IsBanned.eq(false))
        .one(db)
        .await
    {
        Ok(Some(user)) => Some(author_from_user(&user)),
        Ok(None) => None,
        Err(err) => {
            log::warn!("Error resolving direct message recipient: {:?}", err);
            None
        }
    }
}

/// Direct messages sent to or by `user_id`, newest `limit` rows no older than
/// `since` (unix seconds), returned oldest-first.
pub async fn get_direct_message_history(
    db: &DatabaseConnection,
    user_id: u32,
    limit: usize,
    since: i64,
) -> Vec<implement::DirectMessage> {
    if user_id == 0 {
        return Vec::new();
    }

    let since = Decimal::new(since.max(0) * 1_000_000, 6);

    // Anchored on room_id 0 so the idx_room_date (room_id, message_date) index
    // bounds the scan to direct messages inside the window, and yields them
    // already ordered. Without it the optimizer ranges over every direct
    // message ever sent and then filesorts.
    let rows = match chat_message::Entity::find()
        .filter(chat_message::Column::RoomId.eq(0u32))
        .filter(chat_message::Column::RecipientId.is_not_null())
        .filter(
            Condition::any()
                .add(chat_message::Column::UserId.eq(user_id))
                .add(chat_message::Column::RecipientId.eq(user_id)),
        )
        .filter(chat_message::Column::MessageDate.gte(since))
        .filter(chat_message::Column::DeletedDate.is_null())
        .order_by_desc(chat_message::Column::MessageDate)
        .limit(limit as u64)
        .all(db)
        .await
    {
        Ok(rows) => rows,
        Err(err) => {
            log::warn!("Error pulling XF direct message history: {:?}", err);
            return Vec::new();
        }
    };

    if rows.is_empty() {
        return Vec::new();
    }

    // Resolve every participant in one query so avatars are correct for both
    // sides of each conversation.
    let mut user_ids: Vec<u32> = Vec::with_capacity(rows.len() * 2);
    for row in &rows {
        if let Some(id) = row.user_id {
            user_ids.push(id);
        }
        if let Some(id) = row.recipient_id {
            user_ids.push(id);
        }
    }
    user_ids.sort_unstable();
    user_ids.dedup();

    let authors: HashMap<u32, implement::Author> = match user::Entity::find()
        .filter(user::Column::UserId.is_in(user_ids))
        .all(db)
        .await
    {
        Ok(users) => users
            .iter()
            .map(|user| (user.user_id, author_from_user(user)))
            .collect(),
        Err(err) => {
            log::warn!("Error resolving direct message participants: {:?}", err);
            HashMap::new()
        }
    };

    /// Falls back to the name stored on the row when the account is gone.
    fn author_or_stored(
        authors: &HashMap<u32, implement::Author>,
        id: u32,
        username: &str,
    ) -> implement::Author {
        authors
            .get(&id)
            .cloned()
            .unwrap_or_else(|| implement::Author {
                id,
                username: username.to_owned(),
                avatar_url: String::new(),
            })
    }

    rows.into_iter()
        .rev()
        .filter_map(|row| {
            let recipient_id = row.recipient_id?;
            let author_id = row.user_id.unwrap_or(0);

            Some(implement::DirectMessage {
                author: author_or_stored(&authors, author_id, &row.username),
                recipient: author_or_stored(
                    &authors,
                    recipient_id,
                    row.recipient_username.as_deref().unwrap_or_default(),
                ),
                message: implement::Message {
                    message: row.message_text,
                    message_uuid: Uuid::parse_str(&row.message_uuid).unwrap_or_default(),
                    message_date: row.message_date.try_into().unwrap_or(0),
                    message_edit_date: match row.last_edit_date {
                        Some(date) => date.try_into().unwrap_or(0),
                        None => 0,
                    },
                    room_id: row.room_id,
                    user_id: author_id,
                    recipient_id: Some(recipient_id),
                },
            })
        })
        .collect()
}
