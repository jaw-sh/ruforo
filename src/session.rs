use crate::proof::sessions;
use crate::proof::sessions::Entity as Sessions;
use chrono::Utc;
use ruforo::SessionMap;
use sea_orm::{entity::*, DatabaseConnection, DbErr};
use uuid::Uuid;

pub async fn new_session(
    db: &DatabaseConnection,
    ses_map: &SessionMap,
    user_id_: i32,
) -> Result<Uuid, DbErr> {
    let ses = ruforo::Session {
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

    let user = sessions::ActiveModel {
        id: Set(uuid.to_string()),
        user_id: Set(user_id_),
        expires_at: Set(Utc::now().naive_utc()),
    };
    sessions::Entity::insert(user).exec(db).await?;

    Ok(uuid)
}

pub async fn get_session(
    db: &DatabaseConnection,
    session: &Uuid,
) -> Result<Option<sessions::Model>, DbErr> {
    Sessions::find_by_id(session.to_string()).one(db).await
}
