use crate::orm::{ugc, ugc_revisions};
use actix_web::{error, Error};
use chrono::prelude::Utc;
use sea_orm::sea_query::Expr;
use sea_orm::DatabaseConnection;
use sea_orm::{entity::*, query::*, Set};

// Contains only the UGC we can get from a form submission.
pub struct NewUgcPartial<'a> {
    pub ip_id: Option<i32>,
    pub user_id: Option<i32>,
    pub content: &'a str,
}

// Crates a new UGC and an accompanying first revision.
pub async fn create_ugc<'a>(
    pool: &DatabaseConnection,
    revision: NewUgcPartial<'a>,
) -> Result<ugc_revisions::ActiveModel, Error> {
    // Insert new UGC reference with only default values.
    let new_ugc = ugc::ActiveModel {
        ugc_revision_id: Set(None),
        ..Default::default()
    }
    .insert(pool)
    .await
    .map_err(|_| error::ErrorInternalServerError("Failed to insert new UGC."))?;

    let ugc_id = new_ugc.id.clone().unwrap(); // TODO: Change once SeaQL 0.5.0 is out

    Ok(create_ugc_revision(pool, ugc_id, revision).await?)
}

// Creates a new UGC revision and sets it as the living revision for the UGC it belongs to.
pub async fn create_ugc_revision<'a>(
    pool: &DatabaseConnection,
    ugc_id: i32,
    revision: NewUgcPartial<'a>,
) -> Result<ugc_revisions::ActiveModel, Error> {
    // Run model through validator.
    let revision = validate_ugc(revision).map_err(|err| err)?;

    // Use supplied _revision to build a UGC Revision with referebasences we just created.
    let new_revision: ugc_revisions::ActiveModel = ugc_revisions::ActiveModel {
        created_at: Set(Utc::now().naive_utc()),
        ugc_id: Set(ugc_id),
        ip_id: Set(revision.ip_id),
        user_id: Set(revision.user_id),
        content: Set(revision.content.to_owned()),
        ..Default::default()
    }
    .insert(pool)
    .await
    .map_err(|_| error::ErrorInternalServerError("Failed to insert new UGC revision."))?;

    let ugc_revision_id = new_revision.id.clone().unwrap(); // TODO: Change once SeaQL 0.5.0 is out
    ugc::Entity::update_many()
        .col_expr(ugc::Column::UgcRevisionId, Expr::value(ugc_revision_id))
        .filter(ugc::Column::Id.eq(ugc_id))
        .exec(pool)
        .await
        .map_err(|_| error::ErrorInternalServerError("Failed to update UGC to living revision."))?;

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
        content: clean_content,
    })
}
