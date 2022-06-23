use super::message;
use serde::{Deserialize, Serialize};

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

impl Author {
    pub fn can_send_message(&self) -> bool {
        self.id > 0
    }
}

pub struct Message {
    pub user_id: u32,
    pub room_id: u32,
    pub message_id: u32,
    pub message_date: i32,
    pub message: String,
    pub edited: bool,
}

pub struct Room {
    pub room_id: u32,
    pub title: String,
    pub description: String,
    pub motd: Option<String>,
    pub display_order: u32,
}

/// Private session data for chat.
#[derive(Clone, Debug, Serialize)]
pub struct Session {
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
    async fn delete_message(&self, id: i32);
    async fn edit_message(&self, id: i32, author: Author, message: String) -> Option<Message>;
    async fn get_message(&self, message_id: i32) -> Option<Message>;
    async fn get_room_list(&self) -> Vec<Room>;
    async fn get_room_history(&self, room_id: usize, limit: usize) -> Vec<message::ClientMessage>;
    async fn get_smilie_list(&self) -> Vec<Smilie>;
    async fn get_session_from_user_id(&self, id: u32) -> Session;
    fn get_user_id_from_request(&self, req: &actix_web::HttpRequest) -> u32;
    async fn insert_chat_message(&self, message: &message::ClientMessage)
        -> message::ClientMessage;
}

// When we diverge from the XF compat, this can probably be compressed out of a trait.
pub mod default {
    use crate::middleware::ClientCtx;
    use rand::Rng;
    use sea_orm::DatabaseConnection;
    use std::time::SystemTime;

    pub struct Layer {
        pub db: DatabaseConnection,
    }

    #[async_trait::async_trait]
    impl super::ChatLayer for Layer {
        async fn delete_message(&self, _: i32) {
            // TODO
        }

        async fn edit_message(
            &self,
            _: i32,
            _: super::Author,
            _: String,
        ) -> Option<super::Message> {
            // TODO
            None
        }

        async fn get_message(&self, _: i32) -> Option<super::Message> {
            // TODO
            None
        }

        async fn get_room_list(&self) -> Vec<super::Room> {
            vec![super::Room {
                room_id: 1,
                title: "Test".to_owned(),
                description: "Dummy room for testing".to_owned(),
                motd: None,
                display_order: 1,
            }]
        }

        async fn get_room_history(&self, _: usize, _: usize) -> Vec<super::message::ClientMessage> {
            Vec::new()
        }

        async fn get_smilie_list(&self) -> Vec<super::Smilie> {
            Vec::new()
        }

        async fn get_session_from_user_id(&self, id: u32) -> super::Session {
            match crate::user::ClientUser::fetch_by_user_id(&self.db, id as i32).await {
                Some(user) => super::Session {
                    id,
                    username: user.name,
                    avatar_url: "".to_owned(),
                    ignored_users: Vec::new(),
                    is_staff: false,
                },
                None => super::Session::default(),
            }
        }

        fn get_user_id_from_request(&self, req: &actix_web::HttpRequest) -> u32 {
            match req.app_data::<ClientCtx>() {
                Some(client) => client.get_id().unwrap_or(0) as u32,
                None => 0,
            }
        }

        async fn insert_chat_message(
            &self,
            message: &super::message::ClientMessage,
        ) -> super::message::ClientMessage {
            let mut rng = rand::thread_rng();
            let now = SystemTime::UNIX_EPOCH;

            super::message::ClientMessage {
                id: rng.gen(),
                message_id: rng.gen(),
                author: message.author.to_owned(),
                room_id: message.room_id,
                message_date: now.elapsed().unwrap().as_secs() as i32,
                message: message.message.to_owned(),
                message_raw: message.message.to_owned(),
                sanitized: false,
                edited: message.edited,
            }
        }
    }
}
