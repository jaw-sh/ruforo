use crate::orm::{ugc, ugc_revisions};
use actix_web::{error, Error};
use chrono::prelude::Utc;
use sea_orm::DatabaseConnection;
use sea_orm::{entity::*, Set};

// Contains only the UGC we can get from a form submission.
pub struct NewUgcPartial {
    pub ip_id: Option<i32>,
    pub user_id: Option<i32>,
    pub content: String,
}

pub async fn create_ugc(
    pool: &DatabaseConnection,
    revision: NewUgcPartial,
) -> Result<ugc_revisions::ActiveModel, Error> {
    let timestamp = Utc::now().naive_utc();

    // Run model through validator.
    let revision = validate_ugc(revision).map_err(|err| err)?;

    // Insert new UGC reference with only default values.
    let mut new_ugc = dbg!(ugc::ActiveModel {
        ugc_revision_id: Set(None),
        ..Default::default()
    }
    .insert(pool)
    .await
    .map_err(|_| error::ErrorInternalServerError("Failed to insert new UGC."))?);

    // Use supplied _revision to build a UGC Revision with referebasences we just created.
    let new_revision: ugc_revisions::ActiveModel = ugc_revisions::ActiveModel {
        created_at: Set(timestamp),
        ugc_id: new_ugc.id.to_owned(),
        ip_id: Set(revision.ip_id),
        user_id: Set(revision.user_id),
        content: Set(revision.content),
        ..Default::default()
    }
    .insert(pool)
    .await
    .map_err(|_| error::ErrorInternalServerError("Failed to insert new UGC revision."))?;

    // Update the new UGC to point at the living revision we just inserted.
    new_ugc.ugc_revision_id = Set(Some(new_revision.id.to_owned().unwrap()));
    new_ugc.update(pool).await.map_err(|_| {
        error::ErrorInternalServerError("Could not update ugc with living revision id.")
    })?;

    Ok(new_revision)
}

fn validate_ugc(revision: NewUgcPartial) -> Result<NewUgcPartial, Error> {
    let content = revision.content;
    let clean_content = content.trim();

    if clean_content.len() == 0 {
        return Err(error::ErrorUnprocessableEntity(
            "Input must contain content or attachments.",
        ));
    }

    Ok(NewUgcPartial {
        ip_id: revision.ip_id,
        user_id: revision.user_id,
        content: clean_content.to_owned(),
    })
}
