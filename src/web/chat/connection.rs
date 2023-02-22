use super::implement::Session;
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
                act.send_or_reply(ctx, message::Disconnect { id: act.id });

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
                self.send_or_reply(
                    ctx,
                    message::Delete {
                        id: self.id,
                        session: self.session.to_owned(),
                        message_id,
                    },
                );
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
                    session: self.session.to_owned(),
                    message: v.message.trim().to_string(),
                    message_id: v.id,
                };

                if !msg.message.is_empty() {
                    self.send_or_reply(ctx, msg);
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
                self.send_or_reply(
                    ctx,
                    message::Join {
                        id: self.id,
                        session: self.session.to_owned(),
                        room_id: room_id as u32,
                    },
                );
            }
            Err(_) => ctx.text("Invalid room specified."),
        }
    }

    fn cmd_restart(&mut self, ctx: &mut ws::WebsocketContext<Self>, _: Vec<&str>) {
        self.send_or_reply(
            ctx,
            message::Restart {
                id: self.id,
                session: self.session.to_owned(),
            },
        );
    }

    /// Try to send message
    ///
    /// This method fails if actor's mailbox is full or closed. This method
    /// register current task in receivers queue.
    fn send_or_reply<M>(&self, ctx: &mut ws::WebsocketContext<Self>, msg: M)
    where
        M: Message + std::marker::Send + 'static,
        M::Result: Send,
        ChatServer: Handler<M>,
    {
        if let Err(err) = self.addr.try_send(msg) {
            ctx.text("Chat server is down. Waiting for OK.");
        }
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

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        // notify chat server
        self.send_or_reply(ctx, message::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<message::Reply> for Connection {
    type Result = ();

    fn handle(&mut self, msg: message::Reply, ctx: &mut Self::Context) {
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

                if m.is_empty() || m.len() >= 1024 {
                    return;
                }

                // Forward-slash commands
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/delete" => self.cmd_delete(ctx, v),
                        "/edit" => self.cmd_edit(ctx, v),
                        "/join" => self.cmd_join(ctx, v),
                        "/reset" => self.cmd_restart(ctx, v),
                        _ => ctx.text(format!("Unknown command: {:?}", m)),
                    }
                }
                // Client Chat Messages
                else if let Some(room_id) = self.room {
                    self.send_or_reply(
                        ctx,
                        message::Post {
                            id: self.id,
                            session: self.session.to_owned(),
                            message: m.to_string(),
                            room_id: room_id as u32,
                        },
                    )
                }
                // Client message to nowhere
                else {
                    ctx.text("You say something to yourself. Nobody replies.")
                }
            }
            ws::Message::Binary(_) => log::warn!("Unexpected binary"),
            ws::Message::Close(reason) => {
                log::debug!(
                    "Client {} disconnecting with reason: {:?}",
                    self.session.id,
                    reason
                );
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
