pub mod message;
pub mod orm;
pub mod permission;
pub mod room;
pub mod session;
pub mod smilie;

use actix_web::web::Data;
use ruforo::web::chat::{implement, message::Post};
use std::time::Duration;

pub struct XfLayer {
    pub db: sea_orm::DatabaseConnection,
}

#[async_trait::async_trait]
impl implement::ChatLayer for XfLayer {
    async fn can_send_message(&self, session: &implement::Session) -> bool {
        session::can_send_message(&self.db, session.id).await
    }

    async fn can_view(&self, session_id: u32, room_id: u32) -> bool {
        room::can_read_room(&self.db, session_id, room_id).await
    }

    async fn delete_message(&self, id: u32) {
        message::delete_message(&self.db, id).await
    }

    async fn edit_message(
        &self,
        id: u32,
        author: implement::Author,
        message: String,
    ) -> Option<implement::Message> {
        message::edit_message(&self.db, id, author, message).await
    }

    async fn get_message(&self, id: u32) -> Option<implement::Message> {
        message::get_message(&self.db, id).await
    }

    async fn get_room_list(&self) -> Vec<implement::Room> {
        room::get_room_list(&self.db)
            .await
            .into_iter()
            .map(|room| implement::Room {
                id: room.room_id,
                title: room.title,
                description: room.description,
                motd: None,
                display_order: room.display_order,
            })
            .collect()
    }

    async fn get_room_history(
        &self,
        room_id: u32,
        limit: usize,
    ) -> Vec<(implement::Author, implement::Message)> {
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

    async fn insert_chat_message(&self, message: &Post) -> Option<implement::Message> {
        if self.can_send_message(&message.session).await {
            Some(message::insert_chat_message(&self.db, message).await)
        } else {
            None
        }
    }
}

impl From<orm::chat_message::Model> for implement::Message {
    fn from(model: orm::chat_message::Model) -> Self {
        implement::Message {
            user_id: model.user_id.unwrap_or(0),
            room_id: model.room_id,
            message: model.message_text,
            message_id: model.message_id,
            message_date: model.message_date.try_into().unwrap(),
            message_edit_date: match model.last_edit_date {
                Some(date) => date.try_into().unwrap(),
                None => 0,
            },
        }
    }
}
