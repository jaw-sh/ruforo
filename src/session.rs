use crate::orm::sessions;
use crate::orm::sessions::Entity as Sessions;
use chrono::Utc;
use ruforo::SessionMap;
use sea_orm::{entity::*, DatabaseConnection, DbErr};
use uuid::Uuid;

pub async fn new_session(
    db: &DatabaseConnection,
    ses_map: &SessionMap,
    user_id: i32,
) -> Result<Uuid, DbErr> {
    let ses = ruforo::Session {
        user_id,
        expire: chrono::Utc::now().naive_utc(),
    };
    let mut uuid;
    loop {
        uuid = Uuid::new_v4();
        let ses_map = &mut *ses_map.write().unwrap();
        if ses_map.contains_key(&uuid) == false {
            ses_map.insert(uuid, ses);
            break;
        }
    }

    let session = sessions::ActiveModel {
        id: Set(uuid.to_string().to_owned()),
        user_id: Set(user_id),
        expires_at: Set(Utc::now().naive_utc()),
    };
    sessions::Entity::insert(session).exec(db).await?;

    Ok(uuid)
}

/// copies a session out of the mutex protected hashmap
pub async fn get_session(ses_map: &SessionMap, uuid: &Uuid) -> Option<ruforo::Session> {
    match ses_map.read().unwrap().get(uuid) {
        Some(uuid) => Some(uuid.to_owned()), // TODO add expiration checking
        None => None,
    }
}

/// use get_session instead unless you have a really good reason to talk to the DB
pub async fn get_session_from_db(
    db: &DatabaseConnection,
    uuid: &Uuid,
) -> Result<Option<sessions::Model>, DbErr> {
    Sessions::find_by_id(uuid.to_string()).one(db).await
}
