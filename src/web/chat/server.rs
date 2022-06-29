use super::implement::{self, UserActivity};
use super::implement::{ChatLayer, Connection};
use super::message::{self, SanitaryPost, SanitaryPosts};
use crate::bbcode::{tokenize, Constructor, Parser, Smilies};
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::SystemTime;

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session. implementation is super primitive
pub struct ChatServer {
    pub rng: ThreadRng,
    pub layer: Arc<dyn ChatLayer>,

    /// Random Id -> Recipient Addr
    pub connections: HashMap<usize, Connection>,
    /// Room Id -> Vec<Conn Ids>
    pub rooms: HashMap<u32, HashSet<usize>>,
    // Message BbCode Constructor
    pub constructor: Constructor,
}

impl ChatServer {
    pub async fn new(layer: Arc<dyn implement::ChatLayer>) -> Self {
        log::info!("Chat actor starting up.");

        // Populate rooms
        let rooms = layer.get_room_list().await;

        // Constructor
        let constructor = Constructor {
            smilies: Smilies::new_from_tuples(
                layer
                    .get_smilie_list()
                    .await
                    .into_iter()
                    .map(|smilie| (smilie.replace.to_string(), smilie.to_html()))
                    .collect(),
            ),
        };

        Self {
            rng: rand::thread_rng(),
            connections: HashMap::new(),
            rooms: HashMap::from_iter(rooms.into_iter().map(|r| (r.id, Default::default()))),
            constructor,
            layer,
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
        let tokens = match tokenize(&message.message) {
            Ok((_, tokens)) => tokens,
            Err(err) => {
                log::warn!("Tokenizer error: {:?}", err);
                unreachable!();
            }
        };

        let mut parser = Parser::new();
        let ast = parser.parse(&tokens);

        message::SanitaryPost {
            author,
            room_id: message.room_id,
            message_id: message.message_id,
            message_date: message.message_date,
            message_edit_date: message.message_edit_date,
            message: self.constructor.build(ast),
            message_raw: Constructor::sanitize(&message.message),
        }
    }

    /// Send message to specific user
    fn send_message_to_conn(&self, recipient: usize, message: String) {
        if let Some(conn) = self.connections.get(&recipient) {
            conn.recipient.do_send(message::Reply(message));
        }
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

        Box::pin(
            async move {
                // Get the message.
                let res = layer.get_message(msg.message_id).await;

                // If we got the message, check if we can delete it.
                if let Some(message) = &res {
                    if message.user_id == msg.session.id || msg.session.is_staff {
                        // Delete message.
                        layer.delete_message(message.message_id).await;
                    } else {
                        log::warn!(
                            "User {} tried to delete message {:?}",
                            msg.session.id,
                            msg.message_id
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
                        format!("{{\"delete\":[{}]}}", message.message_id),
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

        Box::pin(
            async move {
                // Get the message.
                let res = layer.get_message(msg.message_id).await;

                // If we got the message, check if we can delete it.
                if let Some(message) = &res {
                    if message.user_id == session.id {
                        // Delete message.
                        return layer
                            .edit_message(message.message_id, author, msg.message)
                            .await;
                    } else {
                        log::warn!(
                            "User {} tried to edit message {:?}",
                            msg.session.id,
                            msg.message_id
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
                if layer.can_view(session.id, room_id).await {
                    (true, layer.get_room_history(room_id, 40).await)
                } else {
                    (false, Vec::default())
                }
            }
            .into_actor(self)
            .map(move |(can_view, unsanitized), actor, _ctx| {
                if can_view {
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
impl Handler<message::Post> for ChatServer {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: message::Post, _: &mut Context<Self>) -> Self::Result {
        if msg.session.can_send_message() {
            let layer = self.layer.to_owned();
            let session = msg.session.to_owned();

            Box::pin(
                async move { layer.insert_chat_message(&msg).await }
                    .into_actor(self)
                    .map(move |message, actor, _| {
                        let room_id = message.room_id;

                        actor.send_message_to_room(
                            room_id,
                            serde_json::to_string(&message::SanitaryPosts {
                                messages: vec![actor
                                    .prepare_message(implement::Author::from(&session), message)],
                            })
                            .expect("message::Post serialize failure"),
                        );
                    }),
            )
        } else {
            self.send_message_to_conn(msg.id, "You cannot send messages.".to_string());
            Box::pin(async {}.into_actor(self))
        }
    }
}

//impl SystemService for ChatServer {}
//impl Supervised for ChatServer {}
