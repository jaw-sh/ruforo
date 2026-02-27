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
    let mut skipped = 0u32;

    // XF stores relative image paths; prepend the public URL.
    let public_url = std::env::var("XF_PUBLIC_URL").unwrap_or_default();

    for model in models {
        let sprite_params = match model.sprite_mode {
            1 => Some(implement::SpriteParams::from(&model.sprite_params)),
            _ => None,
        };

        // Log first smilie for diagnostics
        if result.is_empty() && skipped == 0 {
            log::info!(
                "Smilie sample: title={:?}, text={:?}, image_url={:?}, sprite_mode={}, sprite_params={:?}",
                model.title, model.smilie_text, model.image_url, model.sprite_mode, model.sprite_params
            );
        }

        // Resolve relative image_url to absolute using XF_PUBLIC_URL
        let image_url = if model.image_url.is_empty() {
            String::new()
        } else if model.image_url.starts_with("http://") || model.image_url.starts_with("https://") {
            model.image_url.to_owned()
        } else {
            format!("{}/{}", public_url, model.image_url.trim_start_matches('/'))
        };

        for token in model.smilie_text.split_whitespace() {
            let smilie = implement::Smilie {
                title: model.title.to_owned(),
                replace: token.to_owned(),
                image_url: image_url.clone(),
                sprite_params: sprite_params.clone(),
            };

            if smilie.is_valid() {
                result.push(smilie);
            } else {
                skipped += 1;
            }
        }
    }

    if skipped > 0 {
        log::warn!("Skipped {} smilies with empty image_url and no sprite params.", skipped);
    }
    log::info!("Loaded {} smilies.", result.len());

    result
}
