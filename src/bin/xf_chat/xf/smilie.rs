use super::orm::smilie;
use ruforo::web::chat::implement;
use sea_orm::entity::prelude::*;
use sea_orm::{query::*, DatabaseConnection};

pub async fn get_smilie_list(db: &DatabaseConnection) -> Vec<implement::Smilie> {
    let models = smilie::Entity::find()
        .order_by_asc(smilie::Column::DisplayOrder)
        .all(db)
        .await
        .expect("Unable to fetch smilie list");
    let mut result: Vec<implement::Smilie> = Vec::with_capacity(models.len() * 4);

    for model in models {
        for token in model.smilie_text.split_whitespace() {
            result.push(implement::Smilie {
                title: model.title.to_owned(),
                replace: token.to_owned(),
                image_url: model.image_url.to_owned(),
                sprite_params: match model.sprite_mode {
                    1 => Some(implement::SpriteParams::from(&model.sprite_params)),
                    _ => None,
                },
            });
        }
    }

    result
}
