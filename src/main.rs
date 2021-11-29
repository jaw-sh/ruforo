use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_session::{CookieSession, Session};
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use env_logger::Env;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// do websocket handshake and start `MyWebSocket` actor
async fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
	println!("{:?}", r);
	let res = ws::start(MyWebSocket::new(), &r, stream);
	println!("{:?}", res);
	res
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
struct MyWebSocket {
	/// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
	/// otherwise we drop connection.
	hb: Instant,
}

impl Actor for MyWebSocket {
	type Context = ws::WebsocketContext<Self>;

	/// Method is called on actor start. We start the heartbeat process here.
	fn started(&mut self, ctx: &mut Self::Context) {
		self.hb(ctx);
	}
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
	fn handle(
		&mut self,
		msg: Result<ws::Message, ws::ProtocolError>,
		ctx: &mut Self::Context,
	) {
		// process websocket messages
		println!("WS: {:?}", msg);
		match msg {
			Ok(ws::Message::Ping(msg)) => {
				self.hb = Instant::now();
				ctx.pong(&msg);
			}
			Ok(ws::Message::Pong(_)) => {
				self.hb = Instant::now();
			}
			Ok(ws::Message::Text(text)) => ctx.text(text),
			Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
			Ok(ws::Message::Close(reason)) => {
				ctx.close(reason);
				ctx.stop();
			}
			_ => ctx.stop(),
		}
	}
}

impl MyWebSocket {
	fn new() -> Self {
		Self { hb: Instant::now() }
	}

	/// helper method that sends ping to client every second.
	///
	/// also this method checks heartbeats from client
	fn hb(&self, ctx: &mut <Self as Actor>::Context) {
		ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
			// check client heartbeats
			if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
				// heartbeat timed out
				println!("Websocket Client heartbeat failed, disconnecting!");

				// stop actor
				ctx.stop();

				// don't try to send a ping
				return;
			}

			ctx.ping(b"");
		});
	}
}

#[get("/")]
async fn hello() -> impl Responder {
	HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
	HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
	HttpResponse::Ok().body("Hey there!")
}

async fn index(session: Session) -> Result<HttpResponse, Error> {
	// access session data
	if let Some(count) = session.get::<i32>("counter")? {
		session.insert("counter", count + 1)?;
	} else {
		session.insert("counter", 1)?;
	}

	Ok(HttpResponse::Ok().body(format!(
		"Count is {:?}!",
		session.get::<i32>("counter")?.unwrap()
	)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
	HttpServer::new(|| {
		App::new()
			.wrap(Logger::default())
			.wrap(Logger::new("%a %{User-Agent}i"))
			.wrap(
				CookieSession::signed(&[0; 32]) // <- create cookie based session middleware
					.secure(false),
			)
			.service(web::resource("/ws/").route(web::get().to(ws_index)))
			.service(web::resource("/").to(index))
			.service(hello)
			.service(echo)
			.route("/hey", web::get().to(manual_hello))
	})
	.bind("127.0.0.1:8080")?
	.run()
	.await
}
