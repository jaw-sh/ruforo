use super::implement;
use actix::prelude::*;
use serde::Serialize;
use uuid::Uuid;

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

    pub message_uuid: Uuid,
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
    pub message_uuid: Uuid,
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
    /// Recipient room. Zero for direct messages.
    pub room_id: u32,
    /// Unique message identifier, generated before broadcast
    pub message_uuid: Uuid,
    /// Set for direct messages only.
    pub recipient_id: Option<u32>,
    /// Set for direct messages only.
    pub recipient_username: Option<String>,
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

pub struct Restart {
    /// Conn Id
    pub id: usize,
    /// Author Session
    pub session: implement::Session,
}

impl Message for Restart {
    type Result = ();
}

/// A post from the server containing public, sanitized data.
#[derive(Clone, serde::Serialize)]
pub struct SanitaryPost {
    /// Public author information
    pub author: implement::Author,

    /// Sanitized message.
    pub message: String,
    /// Unique message identifier
    pub message_uuid: Uuid,
    /// Timestamp of last message edit
    pub message_edit_date: i64,
    /// Timestamp of message creation
    pub message_date: i64,
    /// Original message as the user entered
    pub message_raw: String,
    /// Recipient room. Zero for direct messages.
    pub room_id: u32,
    /// Present only on direct messages; identifies the recipient.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<implement::Author>,
}

impl Message for SanitaryPost {
    type Result = ();
}

#[derive(serde::Serialize)]
pub struct SanitaryPosts {
    pub messages: Vec<SanitaryPost>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub history: bool,
}

impl Message for SanitaryPosts {
    type Result = ();
}

#[cfg(test)]
mod tests {
    use super::{SanitaryPost, SanitaryPosts};
    use crate::web::chat::implement::Author;

    fn post(recipient: Option<Author>) -> SanitaryPost {
        SanitaryPost {
            author: Author {
                id: 1,
                username: "Sender".to_owned(),
                avatar_url: String::new(),
            },
            message: "hi".to_owned(),
            message_uuid: uuid::Uuid::nil(),
            message_edit_date: 0,
            message_date: 0,
            message_raw: "hi".to_owned(),
            room_id: 0,
            recipient,
        }
    }

    #[test]
    fn serializes_recipient_only_for_direct_messages() {
        let room_post = serde_json::to_value(post(None)).unwrap();
        assert!(
            room_post.get("recipient").is_none(),
            "room messages must not carry a recipient key"
        );

        let dm = serde_json::to_value(post(Some(Author {
            id: 2,
            username: "Recipient".to_owned(),
            avatar_url: "/avatar.jpg".to_owned(),
        })))
        .unwrap();
        let recipient = dm
            .get("recipient")
            .expect("direct messages carry recipient");
        assert_eq!(recipient.get("id").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(
            recipient.get("username").and_then(|v| v.as_str()),
            Some("Recipient")
        );
        assert_eq!(
            recipient.get("avatar_url").and_then(|v| v.as_str()),
            Some("/avatar.jpg")
        );
    }

    #[test]
    fn serializes_history_marker_only_for_history_batches() {
        let history = serde_json::to_value(SanitaryPosts {
            messages: Vec::new(),
            history: true,
        })
        .unwrap();
        assert_eq!(
            history.get("history"),
            Some(&serde_json::Value::Bool(true))
        );

        let live = serde_json::to_value(SanitaryPosts {
            messages: Vec::new(),
            history: false,
        })
        .unwrap();
        assert!(live.get("history").is_none());
    }
}

/// Direct message from one user to another.
///
/// Recipient is identified by id when `recipient_id` is nonzero, otherwise by
/// `recipient_username`; the chat layer resolves it against the user table so
/// offline users can be addressed.
pub struct Whisper {
    pub id: usize,
    pub session: implement::Session,
    pub message: String,
    pub recipient_id: u32,
    pub recipient_username: String,
    /// Unique message identifier, generated before broadcast
    pub message_uuid: Uuid,
}

impl Message for Whisper {
    type Result = ();
}

/// Request to set or clear the MOTD for a room.
pub struct Motd {
    pub id: usize,
    pub session: implement::Session,
    pub room_id: u32,
    pub message_uuid: Option<Uuid>,
}

impl Message for Motd {
    type Result = ();
}

#[derive(serde::Serialize)]
pub struct MotdPayload {
    pub motd: Option<SanitaryPost>,
}

/// Notification that frontend assets have changed.
pub struct AssetChanged;

impl Message for AssetChanged {
    type Result = ();
}
