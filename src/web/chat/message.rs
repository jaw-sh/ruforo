use crate::compat::xf::orm::{chat_message::Model as XfMsgModel, user::Model as XfUserModel};
use crate::compat::xf::session::{XfAuthor, XfSession};
use actix::prelude::*;
use serde::Serialize;

// Note: There is ambiguous referencing to 'id'.
// An usize id represents the connection actor addr.
// An u32 id is pulled from the db and is a user id.

/// Send message to specific room
#[derive(Serialize)]
pub struct ClientMessage {
    /// Conn Id
    pub id: usize,
    /// Author Session
    pub author: XfAuthor,
    /// Recipient room
    pub room_id: usize,
    /// Message ID from database
    pub message_id: u32,
    /// Message Data
    pub message_date: i32,
    /// Peer message
    pub message: String,
    /// If message has passed through sanitizer.
    pub sanitized: bool,
}

impl ClientMessage {
    pub fn from_xf(message: &XfMsgModel, user: Option<&XfUserModel>) -> Self {
        Self {
            id: 0,
            message_id: message.message_id,
            message_date: message.message_date.try_into().unwrap(),
            message: message.message_text.to_owned(),
            sanitized: false,
            room_id: message.room_id as usize,
            author: match user {
                Some(user) => XfAuthor {
                    id: user.user_id as u32,
                    username: user.username.to_owned(),
                    avatar_date: user.avatar_date as u32,
                },
                None => XfAuthor {
                    id: 0,
                    username: "Guest".to_owned(),
                    avatar_date: 0,
                },
            },
        }
    }
}

impl Message for ClientMessage {
    type Result = ();
}

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<ServerMessage>,
    pub session: XfSession,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    /// Conn Id
    pub id: usize,
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Conn Id
    pub id: usize,
    /// Room Id
    pub room_id: usize,
    /// Author Session
    pub author: XfSession,
}

/// List of available rooms
pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<usize>;
}

/// Message from server to clients
#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerMessage(pub String);

/// Message from server to clients
#[derive(Message)]
#[rtype(result = "()")]
pub struct RoomMessage {
    pub room_id: usize,
    pub message: String,
}
