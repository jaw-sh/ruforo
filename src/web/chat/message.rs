use crate::compat::xf::session::XfSession;
use actix::prelude::*;
use serde::Serialize;

// Note: There is ambiguous referencing to 'id'.
// An usize id represents the connection actor addr.
// An u32 id is pulled from the db and is a user id.

/// Send message to specific room
#[derive(Message, Serialize)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Conn Id
    pub id: usize,
    /// Author Session
    pub author: XfSession,
    /// Peer message
    pub message: String,
}

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<ServerMessage>,
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
    /// Room name
    pub name: String,
}

/// List of available rooms
pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

/// Message from server to clients
#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerMessage(pub String);
