use super::orm::smilie;
use sea_orm::entity::prelude::*;
use sea_orm::{query::*, DatabaseConnection};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Smilie {
    pub title: String,
    pub replace: String,
    pub image_url: String,
    pub sprite_params: Option<SpriteParams>,
}

impl Smilie {
    pub fn to_html(&self) -> String {
        format!("<img src=\"{}\" class=\"smilie\" style=\"{}\" alt=\"{}\" title=\"{}   {}\" loading=\"lazy\" />",
            match &self.sprite_params {
                Some(_) => "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7",
                None => &self.image_url,
            },
            match &self.sprite_params {
                Some(sp) => format!("width: {}px; height: {}px; background: url({}) no-repeat 0 0; background-size: contain;", sp.w, sp.h, self.image_url),
                None => String::new(),
            },
            self.replace,
            self.title,
            self.replace
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpriteParams {
    h: usize,
    w: usize,
}

impl From<&serde_json::Value> for SpriteParams {
    fn from(json: &serde_json::Value) -> Self {
        let h = json.get("h");
        let w = json.get("w");

        if let (Some(h), Some(w)) = (h, w) {
            if let (Some(h), Some(w)) = (h.as_str(), w.as_str()) {
                if let (Ok(h), Ok(w)) = (h.parse::<usize>(), w.parse::<usize>()) {
                    return Self { h, w };
                }
            }
        }

        Self { h: 0, w: 0 }
    }
}

pub async fn get_smilie_list(db: &DatabaseConnection) -> Vec<Smilie> {
    let models = smilie::Entity::find()
        .order_by_asc(smilie::Column::DisplayOrder)
        .all(db)
        .await
        .expect("Unable to fetch smilie list");
    let mut result: Vec<Smilie> = Vec::with_capacity(models.len() * 4);

    for model in models {
        for token in model.smilie_text.split_whitespace() {
            result.push(Smilie {
                title: model.title.to_owned(),
                replace: token.to_owned(),
                image_url: model.image_url.to_owned(),
                sprite_params: match model.sprite_mode {
                    1 => Some(SpriteParams::from(&model.sprite_params)),
                    _ => None,
                },
            });
        }
    }

    result
}
