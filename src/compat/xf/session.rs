// This is dark magic which interprets the XF2 PHP-serialized session keys.

use actix_web::{web::Data, HttpRequest};
use redis::Commands;
use serde::Deserialize;
use serde_php::from_bytes;
use std::time::Duration;

pub struct XfSession {
    pub id: usize,
    pub username: String,
    pub avatar_date: i32,
}

impl XfSession {
    fn get_avatar_uri(&self) -> String {
        format!(
            "http://xf.localhost/data/avatars/m/{}/{}.jpg?{}",
            self.id / 1000,
            self.id,
            self.avatar_date
        )
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Eq, PartialEq)]
struct XfSessionData {
    userId: usize,
}

pub fn get_user_from_request(req: &HttpRequest) -> XfSession {
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

    if id > 0 {}

    XfSession {
        id: 0,
        username: "Guest".to_owned(),
        avatar_date: 0,
    }
}
