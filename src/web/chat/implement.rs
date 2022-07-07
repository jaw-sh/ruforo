use super::message;
use crate::user::Profile;
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Regarding Integers:
// Database keys should be u32.
// Dates are represented with i32.
// WS connections are usize.

/// Author data exposed to the client through chat.
#[derive(Clone, Debug, Serialize)]
pub struct Author {
    pub id: u32,
    pub username: String,
    pub avatar_url: String,
}

impl From<&Session> for Author {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id,
            username: session.username.to_owned(),
            avatar_url: session.avatar_url.to_owned(),
        }
    }
}

/// Author data exposed to the client through chat.
#[derive(Serialize)]
pub struct UserActivity {
    pub id: u32,
    pub username: String,
    pub avatar_url: String,
    pub last_activity: u64,
}

impl From<&Connection> for UserActivity {
    fn from(conn: &Connection) -> Self {
        Self {
            id: conn.session.id,
            username: conn.session.username.to_owned(),
            avatar_url: conn.session.avatar_url.to_owned(),
            last_activity: conn.last_activity,
        }
    }
}

#[derive(Serialize)]
pub struct UserActivities {
    pub users: HashMap<u32, UserActivity>,
}

pub struct Connection {
    pub last_activity: u64,
    pub recipient: Recipient<message::Reply>,
    pub session: Session,
}

pub struct Message {
    pub user_id: u32,
    pub room_id: u32,
    pub message_id: u32,
    pub message_date: i32,
    pub message_edit_date: i32,
    pub message: String,
}

pub struct Room {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub motd: Option<String>,
    pub display_order: u32,
}

/// Private session data for chat.
#[derive(Clone, Debug, Serialize)]
pub struct Session {
    /// User ID, not Conn ID
    pub id: u32,
    pub username: String,
    pub avatar_url: String,
    pub ignored_users: Vec<u32>,
    pub is_staff: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            id: 0,
            username: "Guest".to_owned(),
            avatar_url: String::new(),
            ignored_users: Default::default(),
            is_staff: false,
        }
    }
}

impl Session {
    pub fn can_send_message(&self) -> bool {
        self.id > 0
    }
}

#[derive(Debug)]
pub struct Smilie {
    pub title: String,
    pub replace: String,
    pub image_url: String,
    pub sprite_params: Option<SpriteParams>,
}

impl Smilie {
    pub fn to_html(&self) -> String {
        format!("<img src=\"{}\" class=\"smilie\" style=\"{}\" alt=\"{}\" title=\"{}   {}\" loading=\"lazy\" />",
            match &self.sprite_params {
                Some(_) => "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7",
                None => &self.image_url,
            },
            match &self.sprite_params {
                Some(sp) => format!("width: {}px; height: {}px; background: url({}) no-repeat 0 0; background-size: contain;", sp.w, sp.h, self.image_url),
                None => String::new(),
            },
            self.replace,
            self.title,
            self.replace
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpriteParams {
    h: usize,
    w: usize,
}

impl From<&serde_json::Value> for SpriteParams {
    fn from(json: &serde_json::Value) -> Self {
        let h = json.get("h");
        let w = json.get("w");

        if let (Some(h), Some(w)) = (h, w) {
            if let (Some(h), Some(w)) = (h.as_str(), w.as_str()) {
                if let (Ok(h), Ok(w)) = (h.parse::<usize>(), w.parse::<usize>()) {
                    return Self { h, w };
                }
            }
        }

        Self { h: 0, w: 0 }
    }
}

#[async_trait::async_trait]
pub trait ChatLayer {
    async fn can_view(&self, session_id: u32, room_id: u32) -> bool;
    async fn delete_message(&self, id: u32);
    async fn edit_message(&self, id: u32, author: Author, message: String) -> Option<Message>;
    async fn get_message(&self, message_id: u32) -> Option<Message>;
    async fn get_room_history(&self, room_id: u32, limit: usize) -> Vec<(Author, Message)>;
    async fn get_room_list(&self) -> Vec<Room>;
    async fn get_session_from_user_id(&self, id: u32) -> Session;
    async fn get_smilie_list(&self) -> Vec<Smilie>;
    fn get_user_id_from_request(&self, req: &actix_web::HttpRequest) -> u32;
    async fn insert_chat_message(&self, message: &message::Post) -> Message;
}

// When we diverge from the XF compat, this can probably be compressed out of a trait.
pub mod default {
    use super::super::message;
    use super::*;
    use crate::middleware::ClientCtx;
    use rand::Rng;
    use sea_orm::DatabaseConnection;
    use std::time::SystemTime;

    pub struct Layer {
        pub db: DatabaseConnection,
    }

    #[async_trait::async_trait]
    impl super::ChatLayer for Layer {
        async fn can_view(&self, _: u32, _: u32) -> bool {
            true
        }

        async fn delete_message(&self, _: u32) {
            // TODO
        }

        async fn edit_message(
            &self,
            _: u32,
            _: super::Author,
            _: String,
        ) -> Option<super::Message> {
            // TODO
            None
        }

        async fn get_message(&self, _: u32) -> Option<Message> {
            // TODO
            None
        }

        async fn get_room_list(&self) -> Vec<Room> {
            vec![super::Room {
                id: 1,
                title: "Test".to_owned(),
                description: "Dummy room for testing".to_owned(),
                motd: None,
                display_order: 1,
            }]
        }

        async fn get_room_history(&self, _: u32, _: usize) -> Vec<(Author, Message)> {
            Vec::new()
        }

        async fn get_smilie_list(&self) -> Vec<Smilie> {
            Vec::new()
        }

        async fn get_session_from_user_id(&self, id: u32) -> Session {
            if let Ok(Some(user)) = Profile::get_by_id(&self.db, id as i32).await {
                Session {
                    id,
                    username: user.name,
                    avatar_url: "".to_owned(),
                    ignored_users: Vec::new(),
                    is_staff: false,
                }
            } else {
                Session::default()
            }
        }

        fn get_user_id_from_request(&self, req: &actix_web::HttpRequest) -> u32 {
            match req.app_data::<ClientCtx>() {
                Some(client) => client.get_id().unwrap_or(0) as u32,
                None => 0,
            }
        }

        async fn insert_chat_message(&self, message: &message::Post) -> Message {
            let mut rng = rand::thread_rng();
            let now = SystemTime::UNIX_EPOCH;

            Message {
                user_id: message.session.id,
                room_id: message.room_id,
                message: message.message.to_owned(),
                message_date: now.elapsed().unwrap().as_secs() as i32,
                message_edit_date: 0,
                message_id: rng.gen(),
            }
        }
    }
}
