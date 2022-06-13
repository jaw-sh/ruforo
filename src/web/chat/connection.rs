use super::message;
use super::server::ChatServer;
use super::{CLIENT_TIMEOUT, HEARTBEAT_INTERVAL};
use crate::compat::xf::session::{XfAuthor, XfSession};
use actix::*;
use actix_web_actors::ws;
use std::time::Instant;

pub struct Connection {
    /// connection id
    pub id: usize,
    /// peer session
    pub session: XfSession,
    /// Last Heartbeat
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT), otherwise we drop connection.
    pub hb: Instant,
    /// Active room
    pub room: Option<usize>,
    /// Chat server
    pub addr: Addr<ChatServer>,
    /// Last command (any) sent
    pub last_command: Instant,
}

impl Connection {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.addr.do_send(message::Disconnect { id: act.id });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for Connection {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsConnectionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(message::Connect {
                addr: addr.recipient(),
                session: self.session.to_owned(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    _ => {
                        // something is wrong with chat server
                        println!("Failed to assign conection id");
                        ctx.stop();
                    }
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.addr.do_send(message::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<message::ServerMessage> for Connection {
    type Result = ();

    fn handle(&mut self, msg: message::ServerMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Connection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let m = text.trim();

                if m.len() <= 0 || m.len() >= 1024 {
                    return;
                }

                // Forward-slash commands
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/list" => {
                            // Send ListRooms message to chat server and wait for
                            // response
                            println!("List rooms");
                            self.addr
                                .send(message::ListRooms)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            for room in rooms {
                                                ctx.text(format!("{}", room));
                                            }
                                        }
                                        _ => println!("Something is wrong"),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx)
                            // .wait(ctx) pauses all events in context,
                            // so actor wont receive any new messages until it get list
                            // of rooms back
                        }
                        "/join" => {
                            if v.len() == 2 {
                                match v[1].parse::<usize>() {
                                    Ok(room_id) => {
                                        self.room = Some(room_id);
                                        self.addr.do_send(message::Join {
                                            id: self.id,
                                            room_id: room_id,
                                            author: self.session.clone(),
                                        });
                                    }
                                    Err(_) => ctx.text("!!! invalid room"),
                                }
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        _ => ctx.text(format!("!!! unknown command: {:?}", m)),
                    }
                }
                // Client Chat Messages
                else if let Some(room_id) = self.room {
                    self.addr.do_send(message::ClientMessage {
                        id: self.id,
                        room_id: room_id,
                        author: XfAuthor::from(&self.session),
                        message: crate::bbcode::parse(m),
                        message_id: 0,
                        message_date: 0,
                    })
                }
                // Client message to nowhere
                else {
                    ctx.text("You say something to yourself. Nobody replies.")
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
