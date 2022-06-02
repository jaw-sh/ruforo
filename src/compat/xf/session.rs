// This is dark magic which interprets the XF2 PHP-serialized session keys.

use super::orm::user;
use actix_web::{web::Data, HttpRequest};
use redis::Commands;
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, FromQueryResult, QuerySelect};
use serde::{Deserialize, Serialize};
use serde_php::from_bytes;
use std::time::Duration;

#[derive(Clone, Debug, FromQueryResult, Serialize)]
pub struct XfSession {
    pub id: u32,
    pub username: String,
    pub avatar_date: u32,
}

impl XfSession {
    pub fn can_send_message(&self) -> bool {
        self.id > 0
    }

    pub fn get_avatar_uri(&self) -> String {
        format!(
            "{}/data/avatars/m/{}/{}.jpg?{}",
            std::env::var("XF_PUBLIC_URL").expect("XF_PUBLIC_URL must be set in .env"),
            self.id / 1000,
            self.id,
            self.avatar_date
        )
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Eq, PartialEq)]
struct XfSessionData {
    userId: u32,
}

pub async fn get_user_from_request(db: &DatabaseConnection, req: &HttpRequest) -> XfSession {
    let id = match req.cookie("xf_session") {
        Some(cookie) => {
            let session_value: redis::RedisResult<String> = {
                let mut client = req
                    .app_data::<Data<redis::Client>>()
                    .expect("No Redis client!")
                    .get_connection_with_timeout(Duration::new(1, 0))
                    .expect("No Redis connection!");

                client.get(format!("xf[session_{}][1]", cookie.value()))
            };

            match session_value {
                Ok(session) => match from_bytes::<XfSessionData>(session.as_bytes()) {
                    Ok(deser) => {
                        log::debug!("Client authorized as User {:?}", deser);
                        deser.userId
                    }
                    Err(err) => {
                        log::warn!("FAILED to deserialize {:?}", err);
                        0
                    }
                },
                Err(err) => {
                    log::warn!("FAILED to pull from redis {:?}", err);
                    0
                }
            }
        }
        None => 0,
    };

    println!("Session id: {:?}", id);

    if id > 0 {
        match user::Entity::find_by_id(id)
            .select_only()
            .column_as(user::Column::UserId, "id")
            .column(user::Column::Username)
            .column(user::Column::AvatarDate)
            .filter(user::Column::UserId.eq(id))
            .into_model::<XfSession>()
            .one(db)
            .await
        {
            Ok(res) => match res {
                Some(session) => return session,
                None => {
                    println!("No result for user id {:?}", id);
                }
            },
            Err(err) => {
                println!("MySQL Error: {:?}", err);
            }
        };
    }

    XfSession {
        id: 0,
        username: "Guest".to_owned(),
        avatar_date: 0,
    }
}
