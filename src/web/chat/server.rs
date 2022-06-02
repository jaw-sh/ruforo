use super::message;
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
    /// Random Id -> Recipient Addr
    connections: HashMap<usize, Recipient<message::ServerMessage>>,
    rooms: HashMap<usize, HashSet<usize>>,
    rng: ThreadRng,
}

impl ChatServer {
    pub async fn new_from_xf(db: DatabaseConnection, redis: RedisConnection) -> ChatServer {
        log::info!("New ChatServer from XF Compat");

        // Populate rooms
        let rooms = crate::compat::xf::room::get_room_list(&db).await;

        ChatServer {
            db,
            redis,
            connections: HashMap::new(),
            rooms: HashMap::from_iter(
                rooms
                    .into_iter()
                    .map(|r| (r.room_id as usize, HashSet::<usize>::default())),
            ),
            rng: rand::thread_rng(),
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

        // register session with random id
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
    type Result = ();

    fn handle(&mut self, message: message::ClientMessage, _: &mut Context<Self>) {
        if message.author.can_send_message() {
            if let Ok(json) = &serde_json::to_string(&message) {
                self.send_message(&message.room_id, &json);

                // TODO: XF
                // Spawn thread to insert MySQL row for this message.
                // We don't care, remote app does.
                let thread_db = self.db.clone();
                let mut thread_redis = self.redis.clone();
                actix_web::rt::spawn(async move {
                    crate::compat::xf::message::insert_chat_message(
                        message,
                        &thread_db,
                        &mut thread_redis,
                    )
                    .await;
                });
            } else {
                log::error!("ChatServer has failed to serialize a ClientMessage");
            }
        } else {
            self.send_message_to(message.id, "You cannot send messages.");
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
    type Result = ();

    fn handle(&mut self, message: message::Join, _: &mut Context<Self>) {
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
        for this_room in rooms {
            self.send_message(&this_room, &format!("{} left the room.", &author.username));
        }

        self.rooms
            .entry(room_id.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        self.send_message(&room_id, &format!("{} joined the room.", &author.username));
    }
}
