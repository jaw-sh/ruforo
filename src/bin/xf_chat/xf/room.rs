use crate::xf::permission::get_permissions;

use super::orm::{chat_message, chat_room, permission_cache_content, permission_combination, user};
use super::session::avatar_uri;
use ruforo::web::chat::implement;
use sea_orm::{entity::*, query::*, DatabaseConnection, FromQueryResult, QueryFilter};

#[derive(FromQueryResult)]
struct Nothing {}

#[derive(FromQueryResult)]
struct XfPermissionCache {
    cache_value: serde_json::Value,
}

#[derive(FromQueryResult)]
struct XfPermissionId {
    permission_combination_id: u32,
}

pub async fn can_read_room(db: &DatabaseConnection, user_id: u32, room_id: u32) -> bool {
    let pc_filter = if let Ok(Some(pc)) = user::Entity::find_by_id(user_id)
        .select_only()
        .column(user::Column::PermissionCombinationId)
        .into_model::<XfPermissionId>()
        .one(db)
        .await
    {
        Condition::all().add(
            permission_combination::Column::PermissionCombinationId
                .eq(pc.permission_combination_id),
        )
    } else {
        Condition::all()
            .add(permission_combination::Column::UserId.eq(0 as u32))
            .add(permission_combination::Column::UserGroupList.eq("1".to_owned()))
    };

    match permission_cache_content::Entity::find()
        .filter(permission_cache_content::Column::ContentType.eq("hb_chat_room"))
        .filter(permission_cache_content::Column::ContentId.eq(room_id))
        .filter(pc_filter)
        .find_also_related(permission_combination::Entity)
        .select_only()
        .column_as(
            permission_cache_content::Column::CacheValue,
            "A_cache_value",
        )
        .into_model::<XfPermissionCache, Nothing>()
        .one(db)
        .await
    {
        Ok(val) => match val {
            Some((val, _)) => {
                let perm = get_permissions()
                    .borrow_item_by_label("hbChatRoomView")
                    .expect("No permission category??");
                let perms = super::permission::json_to_values(val.cache_value);

                return perms.can(perm.position);
            }
            None => {}
        },
        Err(err) => log::warn!("Failed to fetch XF permissions: {:?}", err),
    }

    false
}

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
