// This is dark magic which interprets the XF2 PHP-serialized session keys.

use super::orm::user;
use super::orm::user_ignored;
use redis::Commands;
use ruforo::web::chat::implement;
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, FromQueryResult, QuerySelect};
use serde::Deserialize;

#[derive(FromQueryResult)]
struct XfSession {
    pub id: u32,
    pub username: String,
    pub avatar_date: u32,
    pub is_staff: bool,
}

pub fn avatar_uri(id: u32, date: u32) -> String {
    if date > 0 {
        format!(
            "{}/data/avatars/m/{}/{}.jpg?{}",
            std::env::var("XF_PUBLIC_URL").expect("XF_PUBLIC_URL must be set in .env"),
            id / 1000,
            id,
            date
        )
    } else {
        String::new()
    }
}

impl Default for XfSession {
    fn default() -> Self {
        Self {
            id: 0,
            username: "Guest".to_owned(),
            avatar_date: 0,
            is_staff: false,
        }
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Eq, PartialEq)]
struct XfSessionSerialized {
    userId: u32,
}

pub fn get_user_id_from_cookie(
    redis: &mut redis::Connection,
    cookie: &actix_web::cookie::Cookie<'_>,
) -> u32 {
    let session_value: redis::RedisResult<String> =
        redis.get(format!("xf[session_{}][1]", cookie.value()));

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

pub async fn get_session_with_user_id(db: &DatabaseConnection, id: u32) -> implement::Session {
    // Fetch basic user info
    let session = if id > 0 {
        match user::Entity::find_by_id(id)
            .select_only()
            .column_as(user::Column::UserId, "id")
            .column(user::Column::Username)
            .column(user::Column::AvatarDate)
            .column(user::Column::IsStaff)
            .filter(user::Column::UserId.eq(id))
            .filter(user::Column::IsBanned.eq(false))
            .into_model::<XfSession>()
            .one(db)
            .await
        {
            Ok(res) => match res {
                Some(session) => session,
                None => {
                    log::warn!("No result for user id {:?}", id);
                    XfSession::default()
                }
            },
            Err(err) => {
                log::warn!("MySQL Error: {:?}", err);
                XfSession::default()
            }
        }
    } else {
        XfSession::default()
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
                log::warn!("MySQL Error: {:?}", err);
                Default::default()
            }
        }
    } else {
        Default::default()
    };

    implement::Session {
        id: session.id,
        username: session.username,
        avatar_url: avatar_uri(session.id, session.avatar_date),
        ignored_users,
        is_staff: session.is_staff,
    }
}
