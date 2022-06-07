use super::message;
use crate::compat::xf::session::XfAuthor;
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use redis::aio::MultiplexedConnection as RedisConnection;
use sea_orm::DatabaseConnection;
use std::collections::{HashMap, HashSet};

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session. implementation is super primitive
pub struct ChatServer {
    db: DatabaseConnection,
    redis: RedisConnection,

    rng: ThreadRng,

    /// Random Id -> Recipient Addr
    connections: HashMap<usize, Recipient<message::ServerMessage>>,
    rooms: HashMap<usize, HashSet<usize>>,
}

impl ChatServer {
    pub async fn new_from_xf(db: DatabaseConnection, redis: RedisConnection) -> ChatServer {
        log::info!("New ChatServer from XF Compat");

        // Populate rooms
        let rooms = crate::compat::xf::room::get_room_list(&db).await;

        ChatServer {
            db,
            redis,
            rng: rand::thread_rng(),
            connections: HashMap::new(),
            rooms: HashMap::from_iter(
                rooms
                    .into_iter()
                    .map(|r| (r.room_id as usize, HashSet::<usize>::default())),
            ),
        }
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
        println!("Someone joined");

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
            let db = self.db.clone();
            let mut redis = self.redis.clone();

            Box::pin(
                async move { crate::compat::xf::message::insert_chat_message(&message, &db).await }
                    .into_actor(self)
                    .map(move |message, actor, _ctx| {
                        actor.send_message(
                            &message.room_id,
                            &serde_json::to_string(&message)
                                .expect("ClientMessage stringify failed."),
                        );
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

            let db = self.db.clone();

            Box::pin(
                async move {
                    use crate::compat::xf::message::get_chat_room_history;
                    get_chat_room_history(&db, &(room_id as u32), 20).await
                }
                .into_actor(self)
                .map(move |messages, actor, _ctx| {
                    for message in messages {
                        let client_msg = message::ClientMessage {
                            id,
                            room_id,
                            author: XfAuthor {
                                id: message.user_id.unwrap_or(0) as u32,
                                username: message.username,
                                avatar_date: 1,
                            },
                            message_id: message.message_id,
                            message: message.message_text.to_owned(),
                        };
                        actor.send_message_to(
                            id,
                            &serde_json::to_string(&client_msg)
                                .expect("ClientMessage stringify failed."),
                        );
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
