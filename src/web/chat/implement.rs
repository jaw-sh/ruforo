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
}

impl Default for Session {
    fn default() -> Self {
        Self {
            id: 0,
            username: "Guest".to_owned(),
            avatar_url: String::new(),
            ignored_users: Default::default(),
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
    async fn get_room_list(&self) -> Vec<Room>;
    async fn get_room_history(&self, room_id: usize, limit: usize) -> Vec<message::ClientMessage>;
    async fn get_smilie_list(&self) -> Vec<Smilie>;
    async fn get_session_from_user_id(&self, id: u32) -> Session;
    fn get_user_id_from_request(&self, req: &actix_web::HttpRequest) -> u32;
    async fn insert_chat_message(&self, message: &message::ClientMessage)
        -> message::ClientMessage;
}
