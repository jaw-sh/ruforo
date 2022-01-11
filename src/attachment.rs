use crate::filesystem::get_file_url_by_hash;
use crate::init::get_db_pool;
use crate::orm::{attachments, posts, ugc_attachments};
use sea_orm::{entity::*, query::*, FromQueryResult};
use std::collections::HashMap;

/// Represents an attachments on UGC.
#[derive(Debug, FromQueryResult)]
pub struct AttachmentForTemplate {
    // ugc_attachments
    pub id: i32,
    pub ugc_id: i32,
    pub ugc_filename: String,
    // attachments
    pub attachment_id: i32,
    pub local_filename: String,
    pub hash: String,
    pub filesize: i64,
    pub file_height: Option<i32>,
    pub file_width: Option<i32>,
    pub mime: String,
}

impl AttachmentForTemplate {
    pub fn get_download_url(&self) -> String {
        get_file_url_by_hash(&self.hash, &self.ugc_filename)
    }
}

pub async fn get_attachments_for_ugc_by_id(
    ugc: Vec<i32>,
) -> HashMap<i32, Vec<AttachmentForTemplate>> {
    let db = get_db_pool();
    let attachments: Vec<AttachmentForTemplate> = ugc_attachments::Entity::find()
        .select_only()
        .column(ugc_attachments::Column::Id)
        .column(ugc_attachments::Column::UgcId)
        .column_as(ugc_attachments::Column::Filename, "ugc_filename")
        .left_join(attachments::Entity)
        .column_as(attachments::Column::Id, "attachment_id")
        .column_as(attachments::Column::Filename, "local_filename")
        .column(attachments::Column::Hash)
        .column(attachments::Column::Filesize)
        .column(attachments::Column::FileHeight)
        .column(attachments::Column::FileWidth)
        .column(attachments::Column::Mime)
        .filter(ugc_attachments::Column::UgcId.is_in(ugc))
        .order_by_asc(ugc_attachments::Column::CreatedAt)
        .into_model::<AttachmentForTemplate>()
        .all(db)
        .await
        .map_err(|e| {
            log::error!(
                "get_attachments_for_ugc_by_id: ugc_attachments::find(): {}",
                e
            );
        })
        .unwrap_or_default();

    let mut result: HashMap<i32, Vec<AttachmentForTemplate>> = HashMap::new();
    for attachment in attachments {
        let v: &mut Vec<_> = result.entry(attachment.ugc_id).or_default();
        v.push(attachment);
    }
    result
}
