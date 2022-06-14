use super::implement::ChatLayer;
use super::message;
use crate::bbcode::{Constructor, Lexer, Parser};
use actix::prelude::*;
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

    /// Message BbCode Constructor
    pub constructor: Constructor,
}

impl ChatServer {
    /// Prepares a ClientMessage to be sent.
    fn prepare_message(&self, message: &message::ClientMessage) -> String {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(&message.message);

        let mut parser = Parser::new();
        let ast = parser.parse(&tokens);

        serde_json::to_string(&message::ClientMessage {
            id: message.id,
            author: message.author.clone(),
            room_id: message.room_id,
            message_id: message.message_id,
            message_date: message.message_date,
            message: self.constructor.build(ast),
            sanitized: true,
        })
        .expect("ClientMessage stringify failed.")
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

//impl Default for ChatServer {
//    fn default() -> Self {
//        Self::new()
//    }
//}

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
        println!("{} joined chat.", msg.session.username);

        // regifter session with random id
        let id = self.rng.gen::<usize>();
        self.connections.insert(id, msg.addr);

        // auto join session to Main room
        //self.rooms
        //    .entry("Main".to_owned())
        //    .or_insert_with(HashSet::new)
        //    .insert(id);

        id
    }
}

/// Handler for Message message.
impl Handler<message::ClientMessage> for ChatServer {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(
        &mut self,
        message: message::ClientMessage,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        if message.author.can_send_message() {
            let layer = self.layer.clone();

            Box::pin(
                async move { layer.insert_chat_message(&message).await }
                    .into_actor(self)
                    .map(move |message, actor, _ctx| {
                        actor.send_message(&message.room_id, &actor.prepare_message(&message));
                    }),
            )
        } else {
            self.send_message_to(message.id, "You cannot send messages.");
            Box::pin(async {}.into_actor(self))
        }
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

        // send message to other users
        //for room in rooms {
        //    self.send_message(&room, "Someone disconnected");
        //}
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
        if true {
            let message::Join {
                id,
                room_id,
                author,
            } = message;
            let mut rooms = Vec::new();

            // remove session from all rooms
            for (n, connections) in &mut self.rooms {
                if connections.remove(&id) {
                    rooms.push(n.to_owned());
                }
            }

            // send message to other users
            //for this_room in rooms {
            //    self.send_message(&this_room, &format!("{} left the room.", &author.username));
            //}

            let layer = self.layer.clone();

            Box::pin(
                async move { layer.get_room_history(room_id, 20).await }
                    .into_actor(self)
                    .map(move |messages, actor, _ctx| {
                        for message in messages {
                            actor.send_message_to(id, &actor.prepare_message(&message));
                        }

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
