pub mod message;
pub mod orm;
pub mod room;
pub mod session;
pub mod smilie;

use actix_web::web::Data;
use ruforo::web::chat::{implement, message::ClientMessage};
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
