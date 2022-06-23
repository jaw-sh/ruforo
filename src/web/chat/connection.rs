use super::implement::{Author, Session};
use super::message;
use super::server::ChatServer;
use super::{CLIENT_TIMEOUT, HEARTBEAT_INTERVAL};
use actix::*;
use actix_web_actors::ws;
use std::time::Instant;

pub struct Connection {
    /// connection id
    pub id: usize,
    /// peer session
    pub session: Session,
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

    fn cmd_delete(&self, ctx: &mut ws::WebsocketContext<Self>, args: Vec<&str>) {
        if args.len() != 2 {
            ctx.text("Invalid command (no message specified?)");
            return;
        }

        match args[1].parse::<u32>() {
            Ok(message_id) => {
                self.addr.do_send(message::Delete {
                    id: self.id,
                    message_id: message_id,
                    author: self.session.clone(),
                });
            }
            Err(_) => ctx.text("Invalid message specified."),
        }
    }

    fn cmd_edit(&self, ctx: &mut ws::WebsocketContext<Self>, args: Vec<&str>) {
        if args.len() != 2 {
            ctx.text("Invalid command (no data supplied)");
            return;
        }

        #[derive(serde::Deserialize)]
        struct EditFragment {
            id: u32,
            message: String,
        }

        match serde_json::from_str::<EditFragment>(args[1]) {
            Ok(v) => {
                let msg = message::Edit {
                    id: self.id,
                    author: self.session.clone(),
                    message_id: v.id,
                    message: v.message.trim().to_string(),
                };

                if msg.message.len() > 0 {
                    self.addr.do_send(msg);
                }
            }
            Err(err) => {
                println!("{:?}", err);
                ctx.text("Unable to understand your input.");
            }
        };
    }

    fn cmd_join(&mut self, ctx: &mut ws::WebsocketContext<Self>, args: Vec<&str>) {
        if args.len() != 2 {
            ctx.text("Invalid command (no room specified)");
            return;
        }

        match args[1].parse::<usize>() {
            Ok(room_id) => {
                self.room = Some(room_id);
                self.addr.do_send(message::Join {
                    id: self.id,
                    room_id: room_id,
                    author: self.session.clone(),
                });
            }
            Err(_) => ctx.text("Invalid room specified."),
        }
    }

    fn cmd_list(&self, ctx: &mut ws::WebsocketContext<Self>) {
        // Send ListRooms message to chat server and wait for
        // response
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
                    _ => unreachable!(),
                }
                fut::ready(())
            })
            // .wait(ctx) pauses all events in context,
            // so actor wont receive any new messages until it get list
            // of rooms back
            .wait(ctx)
    }

    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        // start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsConnectionState, state is shared
        // across all routes within application
        self.addr
            .send(message::Connect {
                addr: ctx.address().recipient(),
                session: self.session.to_owned(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    Err(err) => {
                        // something is wrong with chat server
                        log::warn!("Failed to assign conection id: {:?}", err);
                        ctx.stop();
                    }
                }
                fut::ready(())
            })
            .wait(ctx);
    }
}

impl Actor for Connection {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_heartbeat(ctx);
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
                        "/delete" => self.cmd_delete(ctx, v),
                        "/edit" => self.cmd_edit(ctx, v),
                        "/list" => self.cmd_list(ctx),
                        "/join" => self.cmd_join(ctx, v),
                        _ => ctx.text(format!("Unknown command: {:?}", m)),
                    }
                }
                // Client Chat Messages
                else if let Some(room_id) = self.room {
                    self.addr.do_send(message::ClientMessage {
                        id: self.id,
                        room_id,
                        author: Author::from(&self.session),
                        message: m.to_string(),
                        message_raw: m.to_string(),
                        sanitized: false,
                        message_id: 0,
                        message_date: 0,
                        edited: false,
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
