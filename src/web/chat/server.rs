use super::message;
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use std::collections::{HashMap, HashSet};

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session. implementation is super primitive
pub struct ChatServer {
    /// Random Id -> Recipient Addr
    connections: HashMap<usize, Recipient<message::ServerMessage>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
}

impl ChatServer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ChatServer {
        log::info!("New ChatServer");
        // pub fn new(visitor_count: Arc<AtomicUsize>) -> ChatServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("Main".to_owned(), HashSet::new());

        ChatServer {
            connections: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
            // visitor_count,
        }
    }

    /// Send message to all users in the room
    fn send_message(&self, room: &str, message: &str) {
        if let Some(connections) = self.rooms.get(room) {
            println!("In room {:?}", room);
            for id in connections {
                println!("Looking at id {:?}", id);
                //if let Some(skip_id)*id != skip_id {
                if let Some(addr) = self.connections.get(id) {
                    println!("Sending to {:?}", id);
                    let _ = addr.do_send(message::ServerMessage(message.to_owned()));
                } else {
                    println!("NOT sending to {:?}", id);
                }
            }
        }
    }
}

impl Default for ChatServer {
    fn default() -> Self {
        Self::new()
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
        println!("Someone joined");

        // notify all users in same room
        self.send_message(&"Main".to_owned(), "Someone joined");

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.connections.insert(id, msg.addr);

        // auto join session to Main room
        self.rooms
            .entry("Main".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        println!(" - Room cnt {:?}", self.rooms.len());
        println!(
            " - Client cnt {:?}",
            self.rooms.entry("Main".to_owned()).or_default().len()
        );

        // let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        // self.send_message("Main", &format!("Total visitors {}", count), 0);

        // send id back
        id
    }
}

/// Handler for Message message.
impl Handler<message::ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: message::ClientMessage, _: &mut Context<Self>) {
        println!("Received client message.");
        self.send_message("Main", msg.msg.as_str());
    }
}

/// Handler for Disconnect message.
impl Handler<message::Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: message::Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.connections.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, connections) in &mut self.rooms {
                if connections.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message(&room, "Someone disconnected");
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
    type Result = ();

    fn handle(&mut self, msg: message::Join, _: &mut Context<Self>) {
        let message::Join { id, name } = msg;
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (n, connections) in &mut self.rooms {
            if connections.remove(&id) {
                rooms.push(n.to_owned());
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message(&room, "Someone disconnected");
        }

        self.rooms
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        self.send_message(&name, "Someone connected");
    }
}
