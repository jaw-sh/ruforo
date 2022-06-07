// This is dark magic which interprets the XF2 PHP-serialized session keys.

use super::orm::user;
use super::orm::user_ignored;
use actix_web::{web::Data, HttpRequest};
use redis::Commands;
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, FromQueryResult, QuerySelect};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize)]
pub struct XfAuthor {
    pub id: u32,
    pub username: String,
    pub avatar_date: u32,
}

impl From<&XfSession> for XfAuthor {
    fn from(session: &XfSession) -> Self {
        Self {
            id: session.id,
            username: session.username.to_owned(),
            avatar_date: session.avatar_date,
        }
    }
}

impl XfAuthor {
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

#[derive(Clone, Debug, Serialize)]
pub struct XfSession {
    pub id: u32,
    pub username: String,
    pub avatar_date: u32,
    pub ignored_users: Vec<u32>,
}

impl Default for XfSession {
    fn default() -> Self {
        Self {
            id: 0,
            username: "Guest".to_owned(),
            avatar_date: 0,
            ignored_users: Default::default(),
        }
    }
}

#[derive(FromQueryResult)]
struct XfSessionDatabase {
    pub id: u32,
    pub username: String,
    pub avatar_date: u32,
}

impl Default for XfSessionDatabase {
    fn default() -> Self {
        Self {
            id: 0,
            username: "Guest".to_owned(),
            avatar_date: 0,
        }
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Eq, PartialEq)]
struct XfSessionSerialized {
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
                Ok(session) => {
                    //use serde_php::from_bytes;
                    //match from_bytes::<XfSessionSerialized>(str::replace(&session, "\\", "").as_bytes()) {
                    match regex::Regex::new(r#"s:6:\\?"?userId\\?"?;i:(?P<user_id>\d+);"#) {
                        Ok(ex) => match ex.captures(&session) {
                            Some(captures) => {
                                log::debug!("Client authorized as User {:?}", &captures["user_id"]);
                                captures["user_id"].parse::<u32>().unwrap()
                            }
                            None => {
                                log::warn!("FAILED to find a user ID in session");
                                0
                            }
                        },
                        Err(err) => {
                            log::warn!("FAILED to parse regex {:?}", err);
                            //log::warn!("FAILED to deserialize {:?}", err);
                            0
                        }
                    }
                }
                Err(err) => {
                    log::warn!("FAILED to pull from redis {:?}", err);
                    0
                }
            }
        }
        None => 0,
    };

    println!("Session id: {:?}", id);

    // Fetch basic user info
    let session = if id > 0 {
        match user::Entity::find_by_id(id)
            .select_only()
            .column_as(user::Column::UserId, "id")
            .column(user::Column::Username)
            .column(user::Column::AvatarDate)
            .filter(user::Column::UserId.eq(id))
            .into_model::<XfSessionDatabase>()
            .one(db)
            .await
        {
            Ok(res) => match res {
                Some(session) => session,
                None => {
                    println!("No result for user id {:?}", id);
                    XfSessionDatabase::default()
                }
            },
            Err(err) => {
                println!("MySQL Error: {:?}", err);
                XfSessionDatabase::default()
            }
        }
    } else {
        XfSessionDatabase::default()
    };

    // Fetch additional information
    let ignored_users: Vec<u32> = if session.id > 0 {
        match user_ignored::Entity::find()
            .filter(user_ignored::Column::UserId.eq(id))
            .all(db)
            .await
        {
            Ok(res) => res
                .into_iter()
                .map(|m| m.ignored_user_id)
                .collect::<Vec<u32>>(),
            Err(err) => {
                println!("MySQL Error: {:?}", err);
                Default::default()
            }
        }
    } else {
        Default::default()
    };

    XfSession {
        id: session.id,
        username: session.username,
        avatar_date: session.avatar_date,
        ignored_users,
    }
}
