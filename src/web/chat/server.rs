use super::implement::ChatLayer;
use super::message;
use crate::bbcode::{tokenize, Constructor, Parser, Smilies};
use actix::prelude::*;
//use actix_broker::BrokerSubscribe;
use rand::{self, rngs::ThreadRng, Rng};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session. implementation is super primitive
pub struct ChatServer {
    pub rng: ThreadRng,
    pub layer: Arc<dyn ChatLayer>,

    /// Random Id -> Recipient Addr
    pub connections: HashMap<usize, Recipient<message::ServerMessage>>,
    pub rooms: HashMap<usize, HashSet<usize>>,
    // Message BbCode Constructor
    pub constructor: Constructor,
}

impl ChatServer {
    pub async fn new(layer: Arc<dyn super::implement::ChatLayer>) -> Self {
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
            rooms: HashMap::from_iter(
                rooms
                    .into_iter()
                    .map(|r| (r.room_id as usize, HashSet::<usize>::default())),
            ),
            constructor,
            layer,
        }
    }

    /// Prepares a ClientMessage to be sent.
    fn prepare_message(&self, message: &message::ClientMessage) -> message::ClientMessage {
        let tokens = match tokenize(&message.message) {
            Ok((_, tokens)) => tokens,
            Err(err) => {
                log::warn!("Tokenizer error: {:?}", err);
                unreachable!();
            }
        };

        let mut parser = Parser::new();
        let ast = parser.parse(&tokens);

        message::ClientMessage {
            id: message.id,
            author: message.author.clone(),
            room_id: message.room_id,
            message_id: message.message_id,
            message_date: message.message_date,
            message: self.constructor.build(ast),
            sanitized: true,
        }
    }

    fn prepare_messages(&self, messages: &Vec<message::ClientMessage>) -> message::ClientMessages {
        let mut data = message::ClientMessages {
            messages: Vec::with_capacity(messages.len()),
        };

        for message in messages {
            data.messages.push(self.prepare_message(&message));
        }

        data
    }

    /// Send message to all users in a room
    fn send_message(&self, room: &usize, message: &str) {
        if let Some(connections) = self.rooms.get(room) {
            for id in connections {
                if let Some(addr) = self.connections.get(id) {
                    let _ = addr.do_send(message::ServerMessage(message.to_owned()));
                }
            }
        }
    }

    /// Send message to specific user
    fn send_message_to(&self, recipient: usize, message: &str) {
        if let Some(addr) = self.connections.get(&recipient) {
            let _ = addr.do_send(message::ServerMessage(message.to_owned()));
        } else {
            println!("Failed to send specific message to client {}", recipient);
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
        self.connections.insert(id, msg.addr);

        id
    }
}

/// Handler for Message message.
impl Handler<message::ClientMessage> for ChatServer {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: message::ClientMessage, _: &mut Context<Self>) -> Self::Result {
        if msg.author.can_send_message() {
            let layer = self.layer.clone();

            Box::pin(
                async move { layer.insert_chat_message(&msg).await }
                    .into_actor(self)
                    .map(move |message, actor, _| {
                        let room_id = message.room_id;
                        let message = vec![message];

                        actor.send_message(
                            &room_id,
                            &serde_json::to_string(&actor.prepare_messages(&message))
                                .expect("ClientMessage serialize failure"),
                        );
                    }),
            )
        } else {
            self.send_message_to(msg.id, "You cannot send messages.");
            Box::pin(async {}.into_actor(self))
        }
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
                let res = layer.get_message(msg.message_id as i32).await;

                // If we got the message, check if we can delete it.
                if let Some(message) = &res {
                    if message.user_id == msg.author.id || msg.author.is_staff {
                        // Delete message.
                        layer
                            .delete_message(message.message_id.to_owned() as i32)
                            .await;
                    } else {
                        log::warn!(
                            "User {} tried to delete message {:?}",
                            msg.author.id,
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
                    actor.send_message(
                        &(message.room_id as usize),
                        &format!("{{\"delete\":[{}]}}", message.message_id),
                    );
                } else {
                    actor.send_message_to(msg.id, "Could not delete message.");
                }
            }),
        )
    }
}
/// Handler for Disconnect message.
impl Handler<message::Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: message::Disconnect, _: &mut Context<Self>) {
        let mut rooms: Vec<usize> = Vec::new();

        // remove address
        if self.connections.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (id, connections) in &mut self.rooms {
                if connections.remove(&msg.id) {
                    rooms.push(id.to_owned());
                }
            }
        }
    }
}

/// Handler for `ListRooms` message.
impl Handler<message::ListRooms> for ChatServer {
    type Result = MessageResult<message::ListRooms>;

    fn handle(&mut self, _: message::ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<message::Join> for ChatServer {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, message: message::Join, _: &mut Context<Self>) -> Self::Result {
        // TODO: Check if room is valid.
        if true {
            let message::Join {
                id,
                room_id,
                author: _,
            } = message;
            let mut rooms = Vec::new();

            // remove session from all rooms
            for (n, connections) in &mut self.rooms {
                if connections.remove(&id) {
                    rooms.push(n.to_owned());
                }
            }

            let layer = self.layer.clone();

            Box::pin(
                async move { layer.get_room_history(room_id, 20).await }
                    .into_actor(self)
                    .map(move |messages, actor, _ctx| {
                        //for message in messages {
                        //    actor.send_message_to(id, &actor.prepare_message(&message));
                        //}

                        actor.send_message_to(
                            id,
                            &serde_json::to_string(&actor.prepare_messages(&messages))
                                .expect("ClientMessages serialize failure"),
                        );

                        // Put user in room now so messages don't load in during history.
                        actor
                            .rooms
                            .entry(room_id.clone())
                            .or_insert_with(HashSet::new)
                            .insert(id);
                    }),
            )
        } else {
            self.send_message_to(message.id, "You cannot join this room.");
            Box::pin(async {}.into_actor(self))
        }
    }
}

//impl SystemService for ChatServer {}
//impl Supervised for ChatServer {}
