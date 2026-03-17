use super::implement::{self, UserActivity};
use super::implement::{ChatLayer, Connection};
use super::message::{self, SanitaryPost, SanitaryPosts};
use crate::bbcode::ChatBBCode;
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use actix_web::rt::time;
use std::time::{Duration, SystemTime};

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session. implementation is super primitive
pub struct ChatServer {
    pub rng: ThreadRng,
    pub layer: Arc<dyn ChatLayer>,

    /// Random Id -> Recipient Addr
    pub connections: HashMap<usize, Connection>,
    /// Room Id -> Vec<Conn Ids>
    pub rooms: HashMap<u32, HashSet<usize>>,
    // Message BbCode renderer
    pub bbcode: ChatBBCode,
    /// Room Id -> pinned MOTD message
    pub motd: HashMap<u32, SanitaryPost>,
}

impl ChatServer {
    pub async fn new(layer: Arc<dyn implement::ChatLayer>) -> Self {
        log::info!("Chat actor starting up.");

        // Populate rooms
        let rooms = layer.get_room_list().await;

        // BBCode renderer with smilies
        let smilies = layer
            .get_smilie_list()
            .await
            .into_iter()
            .map(|smilie| (smilie.replace.to_string(), smilie.to_html()))
            .collect();
        let bbcode = ChatBBCode::new(smilies);

        Self {
            rng: rand::thread_rng(),
            connections: HashMap::new(),
            rooms: HashMap::from_iter(rooms.into_iter().map(|r| (r.id, Default::default()))),
            bbcode,
            layer,
            motd: HashMap::new(),
        }
    }

    fn connect_message(&mut self, room: u32, id: usize) {
        if let Some(conn) = self.connections.get(&id) {
            if conn.session.id > 0 {
                self.send_message_to_room(
                    room,
                    format!(
                        "{{\"users\":{{\"{}\":{}}}}}",
                        conn.session.id,
                        serde_json::to_string(&implement::UserActivity::from(conn))
                            .expect("Failed to serialize Author for connection message.")
                    ),
                );
            }

            if let Some(room_conns) = self.rooms.get(&room) {
                let mut users: HashMap<u32, UserActivity> =
                    HashMap::with_capacity(room_conns.len());

                for room_conn in room_conns {
                    if let Some(tconn) = self.connections.get(room_conn) {
                        users.insert(tconn.session.id, implement::UserActivity::from(tconn));
                    }
                }

                self.send_message_to_conn(
                    id,
                    serde_json::to_string(&implement::UserActivities { users })
                        .expect("Failed to serialize UserActivities for connection message."),
                );
            }
        }
    }

    fn disconnect_message(&mut self, id: usize) {
        let mut left_rooms: Vec<u32> = Vec::with_capacity(self.rooms.len());

        // remove session from all rooms
        for (room_id, roomconns) in &mut self.rooms {
            if roomconns.remove(&id) {
                left_rooms.push(*room_id);
            }
        }

        for room_id in left_rooms {
            if let Some(conn) = self.connections.get(&id) {
                if conn.session.id > 0 {
                    self.send_message_to_room(
                        room_id,
                        format!("{{\"user\":{{\"{}\":false}}}}", conn.session.id),
                    );
                }
            }
        }
    }

    /// Receives session+message database data to create a SanitaryPost.
    fn prepare_message(
        &self,
        author: implement::Author,
        message: implement::Message,
    ) -> message::SanitaryPost {
        message::SanitaryPost {
            author,
            room_id: message.room_id,
            message_uuid: message.message_uuid,
            message_date: message.message_date,
            message_edit_date: message.message_edit_date,
            message: self.bbcode.render(&message.message),
            message_raw: ChatBBCode::sanitize(&message.message),
        }
    }

    /// Send message to specific user
    fn send_message_to_conn(&self, recipient: usize, message: String) {
        if let Some(conn) = self.connections.get(&recipient) {
            conn.recipient.do_send(message::Reply(message));
        }
    }

    /// Find all connection IDs for a user by user_id or username (case-insensitive).
    fn find_connections_by_user(&self, user_id: u32, username: &str) -> Vec<usize> {
        self.connections
            .iter()
            .filter(|(_, conn)| {
                if user_id > 0 {
                    conn.session.id == user_id
                } else {
                    conn.session.username.eq_ignore_ascii_case(username)
                }
            })
            .map(|(&id, _)| id)
            .collect()
    }

    /// Send message to all users in a room
    fn send_message_to_room(&self, room: u32, message: String) {
        if let Some(connections) = self.rooms.get(&room) {
            for id in connections {
                if let Some(conn) = self.connections.get(id) {
                    conn.recipient.do_send(message::Reply(message.to_owned()));
                }
            }
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    /// We are going to use simple Context, we just need ability to communicate with other actors.
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(32);
    }
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<message::Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: message::Connect, _: &mut Context<Self>) -> Self::Result {
        // register session with random id
        let id = self.rng.gen::<usize>();
        self.connections.insert(
            id,
            Connection {
                last_activity: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                recipient: msg.addr,
                session: msg.session,
                room_perms: implement::RoomPermissions::default(),
            },
        );
        id
    }
}

/// Handler for Delete message.
impl Handler<message::Delete> for ChatServer {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: message::Delete, _: &mut Context<Self>) -> Self::Result {
        let layer = self.layer.clone();
        let perms = self
            .connections
            .get(&msg.id)
            .map(|conn| conn.room_perms.clone())
            .unwrap_or_default();

        Box::pin(
            async move {
                // Get the message.
                let res = layer.get_message(msg.message_uuid).await;

                // If we got the message, check if we can delete it.
                if let Some(message) = &res {
                    let is_own = message.user_id == msg.session.id;
                    if (is_own && perms.can_delete_own) || (!is_own && perms.can_delete_other) {
                        log::info!("[delete] {} deleted message {}", msg.session.username, msg.message_uuid);
                        // Delete message.
                        layer.delete_message(message.message_uuid).await;
                    } else {
                        log::warn!(
                            "User {} tried to delete message {:?}",
                            msg.session.id,
                            msg.message_uuid
                        );
                        return None;
                    }
                }

                res
            }
            .into_actor(self)
            .map(move |message, actor, _ctx| {
                if let Some(message) = message {
                    actor.send_message_to_room(
                        message.room_id,
                        format!("{{\"delete\":[\"{}\"]}}", message.message_uuid),
                    );
                } else {
                    actor.send_message_to_conn(msg.id, "Could not delete message.".to_string());
                }
            }),
        )
    }
}

/// Handler for Disconnect message.
impl Handler<message::Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: message::Disconnect, _: &mut Context<Self>) {
        // Send disconnection alert to users in room.
        self.disconnect_message(msg.id);

        // remove address
        self.connections.remove(&msg.id);
    }
}

/// Handler for Edit message.
impl Handler<message::Edit> for ChatServer {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: message::Edit, _: &mut Context<Self>) -> Self::Result {
        let layer = self.layer.to_owned();
        let session = msg.session.to_owned();
        let author = implement::Author::from(&session);
        let perms = self
            .connections
            .get(&msg.id)
            .map(|conn| conn.room_perms.clone())
            .unwrap_or_default();
        log::info!("[edit] {} edited message {}: {}", session.username, msg.message_uuid, msg.message);

        Box::pin(
            async move {
                // Get the message.
                let res = layer.get_message(msg.message_uuid).await;

                // If we got the message, check if we can edit it.
                if let Some(message) = &res {
                    let is_own = message.user_id == session.id;
                    if (is_own && perms.can_edit_own) || (!is_own && perms.can_edit_other) {
                        // Edit message.
                        return layer
                            .edit_message(message.message_uuid, author, msg.message)
                            .await;
                    } else {
                        log::warn!(
                            "User {} tried to edit message {:?}",
                            msg.session.id,
                            msg.message_uuid
                        );
                        return None;
                    }
                }

                res
            }
            .into_actor(self)
            .map(move |message, actor, _ctx| {
                if let Some(message) = message {
                    actor.send_message_to_room(
                        message.room_id,
                        serde_json::to_string(&message::SanitaryPosts {
                            messages: vec![
                                actor.prepare_message(implement::Author::from(&session), message)
                            ],
                        })
                        .expect("ClientMessages serialize failure"),
                    );
                } else {
                    actor.send_message_to_conn(msg.id, "Could not edit message.".to_string());
                }
            }),
        )
    }
}

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<message::Join> for ChatServer {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: message::Join, _: &mut Context<Self>) -> Self::Result {
        let message::Join {
            id,
            session,
            room_id,
        } = msg;

        // Send disconnection alert to users in room.
        self.disconnect_message(msg.id);

        let layer = self.layer.clone();
        Box::pin(
            async move {
                let mut perms = layer.get_room_permissions(session.id, room_id).await;
                // Guests (id 0) can never send
                perms.can_send = perms.can_send && session.id > 0;

                if perms.can_view {
                    match time::timeout(
                        Duration::from_secs(5),
                        layer.get_room_history(room_id, 40),
                    )
                    .await
                    {
                        Ok(history) => (perms, history),
                        Err(_) => {
                            log::warn!("Room history fetch timed out for room {}", room_id);
                            (perms, Vec::default())
                        }
                    }
                } else {
                    (perms, Vec::default())
                }
            }
            .into_actor(self)
            .map(move |(perms, unsanitized), actor, _ctx| {
                if perms.can_view {
                    // Send permissions BEFORE history so the client
                    // has can_report/can_edit/etc. when rendering messages.
                    actor.send_message_to_conn(
                        id,
                        format!(
                            "{{\"permissions\":{}}}",
                            serde_json::to_string(&perms)
                                .expect("RoomPermissions serialize failure")
                        ),
                    );

                    // Store permissions on the connection.
                    if let Some(conn) = actor.connections.get_mut(&id) {
                        conn.room_perms = perms;
                    }

                    // Send MOTD if one is set for this room
                    let motd_payload = message::MotdPayload {
                        motd: actor.motd.get(&room_id).cloned(),
                    };
                    actor.send_message_to_conn(
                        id,
                        serde_json::to_string(&motd_payload)
                            .expect("MotdPayload serialize failure"),
                    );

                    let mut messages: Vec<SanitaryPost> = Vec::with_capacity(unsanitized.len());

                    for (author, message) in unsanitized {
                        messages.push(actor.prepare_message(author, message));
                    }

                    actor.send_message_to_conn(
                        id,
                        serde_json::to_string(&SanitaryPosts { messages })
                            .expect("SanitaryPosts serialize failure"),
                    );

                    // Put user in room now so messages don't load in during history.
                    actor
                        .rooms
                        .entry(room_id)
                        .or_insert_with(HashSet::new)
                        .insert(id);

                    // Announce connection and provide activity to new user.
                    actor.connect_message(room_id, msg.id);

                } else {
                    actor.send_message_to_conn(
                        msg.id,
                "You cannot join this room. Try refreshing. If you still have issues, post in the Sneedchat Discussion thread."
                    .to_string(),
            );
                }
            }),
        )
    }
}

/// Handler for Message message.
/// Uses optimistic broadcast: message is sent to the room immediately with
/// the real UUID, then the DB write happens asynchronously.
impl Handler<message::Post> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: message::Post, ctx: &mut Context<Self>) {
        let can_send = self
            .connections
            .get(&msg.id)
            .map(|conn| conn.room_perms.can_send)
            .unwrap_or(false);

        if !can_send {
            self.send_message_to_conn(msg.id, "You cannot send messages.".to_string());
            return;
        }

        let id = msg.id;
        let room_id = msg.room_id;
        let session = msg.session.to_owned();

        // Reject messages with no visible content (e.g. "[b][/b]")
        let rendered = self.bbcode.render(&msg.message);
        if !ChatBBCode::has_visible_content(&rendered) {
            return;
        }

        log::info!("[room:{}] <{}> {}", msg.room_id, msg.session.username, msg.message);

        // Create an optimistic message with the real UUID and broadcast immediately.
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Reuse the rendered HTML from the visible content check above.
        let sanitary = message::SanitaryPost {
            author: implement::Author::from(&session),
            room_id,
            message_uuid: msg.message_uuid,
            message_date: now,
            message_edit_date: 0,
            message: rendered,
            message_raw: ChatBBCode::sanitize(&msg.message),
        };
        self.send_message_to_room(
            room_id,
            serde_json::to_string(&message::SanitaryPosts {
                messages: vec![sanitary],
            })
            .expect("message::Post optimistic serialize failure"),
        );

        // Spawn background future for DB write with retry logic.
        let layer = self.layer.clone();
        ctx.spawn(
            async move {
                // Attempt 1
                let result = layer.insert_chat_message(&msg).await;
                if result.is_some() {
                    return Ok(());
                }

                // Retry 1 after 1 second
                log::warn!("DB write failed for room {}, retrying in 1s...", room_id);
                time::sleep(Duration::from_secs(1)).await;
                let result = layer.insert_chat_message(&msg).await;
                if result.is_some() {
                    return Ok(());
                }

                // Retry 2 after 2 seconds
                log::warn!("DB write failed for room {}, retrying in 2s...", room_id);
                time::sleep(Duration::from_secs(2)).await;
                let result = layer.insert_chat_message(&msg).await;
                if result.is_some() {
                    return Ok(());
                }

                Err(())
            }
            .into_actor(self)
            .map(move |result, actor, _ctx| {
                if let Err(()) = result {
                    log::error!(
                        "All DB write retries failed for room {} by user {}",
                        room_id,
                        session.username
                    );
                    actor.send_message_to_conn(
                        id,
                        "Your message was displayed but could not be saved. Please try again."
                            .to_string(),
                    );
                }
            }),
        );
    }
}
/// Handler for Whisper message (ephemeral, no DB persistence).
impl Handler<message::Whisper> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: message::Whisper, _: &mut Context<Self>) {
        let can_send = self
            .connections
            .get(&msg.id)
            .map(|conn| conn.room_perms.can_send)
            .unwrap_or(false);

        if !can_send {
            self.send_message_to_conn(msg.id, "You cannot send messages.".to_string());
            return;
        }

        let recipient_conns =
            self.find_connections_by_user(msg.recipient_id, &msg.recipient_username);

        if recipient_conns.is_empty() {
            self.send_message_to_conn(msg.id, "User is not online.".to_string());
            return;
        }

        // Get recipient Author from first matching connection
        let recipient_author = match recipient_conns
            .first()
            .and_then(|id| self.connections.get(id))
        {
            Some(conn) => implement::Author::from(&conn.session),
            None => {
                self.send_message_to_conn(msg.id, "User is not online.".to_string());
                return;
            }
        };

        let rendered = self.bbcode.render(&msg.message);
        if !ChatBBCode::has_visible_content(&rendered) {
            return;
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let sender_author = implement::Author::from(&msg.session);

        let payload = message::WhisperPayload {
            whisper: message::WhisperPost {
                author: sender_author.clone(),
                recipient: recipient_author.clone(),
                message: rendered,
                message_raw: ChatBBCode::sanitize(&msg.message),
                message_date: now,
            },
        };

        let json =
            serde_json::to_string(&payload).expect("WhisperPayload serialize failure");

        // Send to all recipient connections (cross-room)
        for &conn_id in &recipient_conns {
            self.send_message_to_conn(conn_id, json.clone());
        }

        // Send to sender too (if sender is different from recipient)
        let sender_id = msg.session.id;
        if recipient_author.id != sender_id {
            self.send_message_to_conn(msg.id, json);
        }
    }
}

/// Handler for Motd message (in-memory, per-room).
impl Handler<message::Motd> for ChatServer {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: message::Motd, _: &mut Context<Self>) -> Self::Result {
        let can_motd = self
            .connections
            .get(&msg.id)
            .map(|conn| conn.room_perms.can_motd)
            .unwrap_or(false);

        if !can_motd {
            self.send_message_to_conn(
                msg.id,
                "You do not have permission to set the MOTD.".to_string(),
            );
            return Box::pin(async {}.into_actor(self).map(|_, _, _| ()));
        }

        let room_id = msg.room_id;

        match msg.message_uuid {
            None => {
                // Clear MOTD
                self.motd.remove(&room_id);
                let payload = message::MotdPayload { motd: None };
                let json = serde_json::to_string(&payload)
                    .expect("MotdPayload serialize failure");
                self.send_message_to_room(room_id, json);
                Box::pin(async {}.into_actor(self).map(|_, _, _| ()))
            }
            Some(uuid) => {
                let layer = self.layer.clone();
                Box::pin(
                    async move { layer.get_message_with_author(uuid).await }
                        .into_actor(self)
                        .map(move |result, actor, _ctx| {
                            match result {
                                Some((author, message)) => {
                                    if message.room_id != room_id {
                                        actor.send_message_to_conn(
                                            msg.id,
                                            "That message is not in this room.".to_string(),
                                        );
                                        return;
                                    }
                                    let sanitary = actor.prepare_message(author, message);
                                    actor.motd.insert(room_id, sanitary.clone());
                                    let payload = message::MotdPayload {
                                        motd: Some(sanitary),
                                    };
                                    let json = serde_json::to_string(&payload)
                                        .expect("MotdPayload serialize failure");
                                    actor.send_message_to_room(room_id, json);
                                }
                                None => {
                                    actor.send_message_to_conn(
                                        msg.id,
                                        "Message not found.".to_string(),
                                    );
                                }
                            }
                        }),
                )
            }
        }
    }
}

impl Handler<message::Restart> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: message::Restart, ctx: &mut Context<ChatServer>) {
        if msg.session.is_staff {
            log::warn!("ChatServer is being restarted by command, initiated by {:?}", msg.session.username);
            ctx.stop();
        }
    }
}

/// Handler for AssetChanged - notify all clients to refresh.
impl Handler<message::AssetChanged> for ChatServer {
    type Result = ();

    fn handle(&mut self, _: message::AssetChanged, _: &mut Context<Self>) {
        log::info!("Broadcasting asset change notification to all rooms.");
        for room_id in self.rooms.keys().cloned().collect::<Vec<_>>() {
            self.send_message_to_room(
                room_id,
                "{\"system\":\"The chat has been updated. Refresh required.\"}".to_string(),
            );
        }
    }
}

impl Supervised for ChatServer {
    fn restarting(&mut self, _: &mut Context<ChatServer>) {
        log::warn!("Restarting the ChatServer.");
    }
}
