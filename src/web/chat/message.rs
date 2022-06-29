use super::implement;
use actix::prelude::*;
use serde::Serialize;

// Regarding Integers:
// Database keys should be u32.
// Dates are represented with i32.
// WS connections are usize.

/// New chat session is created
pub struct Connect {
    pub addr: Recipient<Reply>,
    pub session: implement::Session,
}

impl Message for Connect {
    type Result = usize;
}

/// Request to delete a chat message.
#[derive(Serialize)]
pub struct Delete {
    pub id: usize,
    pub session: implement::Session,

    pub message_id: u32,
}

impl Message for Delete {
    type Result = ();
}

/// Announce disconnect
pub struct Disconnect {
    pub id: usize,
}

impl Message for Disconnect {
    type Result = ();
}

/// Request to update an existing message.
#[derive(Serialize)]
pub struct Edit {
    pub id: usize,
    pub session: implement::Session,

    pub message: String,
    pub message_id: u32,
}

impl Message for Edit {
    type Result = ();
}

/// Request to join a room.
pub struct Join {
    pub id: usize,
    pub session: implement::Session,

    pub room_id: u32,
}

impl Message for Join {
    type Result = ();
}

#[derive(Serialize)]
pub struct Post {
    /// Conn Id
    pub id: usize,
    /// Author Session
    pub session: implement::Session,

    /// Message as the client entered it
    pub message: String,
    /// Recipient room
    pub room_id: u32,
}

impl Message for Post {
    type Result = ();
}

/// Server response to clientsl
/// Usually a serialized JSON string.
pub struct Reply(pub String);

impl Message for Reply {
    type Result = ();
}

/// A post from the server containing public, sanitized data.
#[derive(serde::Serialize)]
pub struct SanitaryPost {
    /// Public author information
    pub author: implement::Author,

    /// Sanitized message.
    pub message: String,
    /// Message ID from database
    pub message_id: u32,
    /// Timestamp of last message edit
    pub message_edit_date: i32,
    /// Timestamp of message creation
    pub message_date: i32,
    /// Original message as the user entered
    pub message_raw: String,
    /// Recipient room
    pub room_id: u32,
}

impl Message for SanitaryPost {
    type Result = ();
}

#[derive(serde::Serialize)]
pub struct SanitaryPosts {
    pub messages: Vec<SanitaryPost>,
}

impl Message for SanitaryPosts {
    type Result = ();
}
