pub mod message;
pub mod orm;
pub mod room;
pub mod session;
pub mod smilie;

use actix_web::web::Data;
use ruforo::bbcode::Constructor;
use ruforo::web::chat::{implement, message::ClientMessage, server::ChatServer};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

pub struct XfLayer {
    pub db: sea_orm::DatabaseConnection,
}

#[async_trait::async_trait]
impl implement::ChatLayer for XfLayer {
    async fn get_room_list(&self) -> Vec<implement::Room> {
        room::get_room_list(&self.db)
            .await
            .into_iter()
            .map(|room| implement::Room {
                room_id: room.room_id,
                title: room.title,
                description: room.description,
                motd: None,
                display_order: room.display_order,
            })
            .collect()
    }

    async fn get_room_history(&self, room_id: usize, limit: usize) -> Vec<ClientMessage> {
        room::get_room_history(&self.db, room_id, limit).await
    }

    async fn get_smilie_list(&self) -> Vec<implement::Smilie> {
        smilie::get_smilie_list(&self.db).await
    }

    fn get_user_id_from_request(&self, req: &actix_web::HttpRequest) -> u32 {
        let mut redis = req
            .app_data::<Data<redis::Client>>()
            .expect("No Redis client!")
            .get_connection_with_timeout(Duration::new(1, 0))
            .expect("No Redis connection!");

        match req.cookie("xf_session") {
            Some(cookie) => session::get_user_id_from_cookie(&mut redis, &cookie),
            None => 0,
        }
    }

    async fn get_session_from_user_id(&self, id: u32) -> implement::Session {
        session::get_session_with_user_id(&self.db, id).await
    }

    async fn insert_chat_message(&self, message: &ClientMessage) -> ClientMessage {
        message::insert_chat_message(&self.db, message).await
    }
}

pub async fn start_chat_server(layer: Arc<dyn implement::ChatLayer>) -> ChatServer {
    log::info!("New ChatServer from XF Compat");

    // Populate rooms
    let rooms = layer.get_room_list().await;

    // Constructor
    let constructor = Constructor {
        emojis: Some(
            layer
                .get_smilie_list()
                .await
                .into_iter()
                .map(|smilie| (smilie.replace.to_string(), smilie.to_html()))
                .collect(),
        ),
    };

    ChatServer {
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
