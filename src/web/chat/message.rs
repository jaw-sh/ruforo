use super::implement::{Author, Session};
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
    pub author: Author,
    /// Recipient room
    pub room_id: usize,
    /// Message ID from database
    pub message_id: u32,
    /// Message Data
    pub message_date: i32,
    /// Peer message
    pub message: String,
    /// Original message
    pub message_raw: String,
    /// If message has passed through sanitizer.
    pub sanitized: bool,
    /// If the message text has been changed since it was published.
    pub edited: bool,
}

impl Message for ClientMessage {
    type Result = ();
}

/// Send multiple messages
#[derive(Serialize)]
pub struct ClientMessages {
    pub messages: Vec<ClientMessage>,
}

impl Message for ClientMessages {
    type Result = ();
}

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<ServerMessage>,
    pub session: Session,
}

/// Instruction to delete a chat message.
#[derive(Serialize)]
pub struct Delete {
    pub id: usize,
    pub message_id: u32,
    pub author: Session,
}

impl Message for Delete {
    type Result = ();
}

/// Instruction to edit a chat message.
#[derive(Serialize)]
pub struct Edit {
    pub id: usize,
    pub author: Session,
    pub message: String,
    pub message_id: u32,
}

impl Message for Edit {
    type Result = ();
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
    pub author: Session,
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
