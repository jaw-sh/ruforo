use actix_web::{error, Error};
use chrono::prelude::Utc;

pub fn create_ugc(db: &PgConnection, revision_raw: NewUgcRevision) -> Result<UgcRevision, Error> {
    use diesel::insert_into;

    // Input validation.
    let revision = match validate_ugc(revision_raw) {
        Some(revision_result) => match revision_result {
            Ok(revision) => revision,
            Err(err) => return Err(err),
        },
        None => {
            return Err(error::ErrorUnprocessableEntity(
                "Input must contain content or attachments.", // We don't have attachments yet.
            ));
        }
    };

    let timestamp = Utc::now().naive_utc();

    // Insert new UGC reference with only default values.
    let new_ugc = insert_into(ugc)
        .values(NewUgc {
            first_revision_at: timestamp,
            last_revision_at: timestamp,
        })
        .get_result::<Ugc>(db)
        .expect("couldn't insert ugc");

    // Use supplied _revision to build a UGC Revision with referebasences we just created.
    let new_ugc_revision = insert_into(ugc_revisions)
        .values(NewUgcRevisionWithContext {
            ugc_id: new_ugc.id,
            ip_id: revision.ip_id,
            user_id: revision.user_id,
            created_at: timestamp,
            content: revision.content,
        })
        .get_result::<UgcRevision>(db)
        .expect("couldn't insert ugc revision");

    // Update the new UGC to point at the living revision we just inserted.
    diesel::update(&new_ugc)
        .set(ugc_revision_id.eq(Some(new_ugc_revision.id)))
        .execute(db)
        .expect("couldn't update ugc with living revision id");

    Ok(new_ugc_revision)
}

fn validate_ugc(revision: NewUgcRevision) -> Option<Result<NewUgcRevision, Error>> {
    if revision.content.is_none() {
        return None;
    }

    let content = revision.content.unwrap();
    let clean_content = content.trim();

    if clean_content.len() == 0 {
        return None;
    }

    Some(Ok(NewUgcRevision {
        ip_id: revision.ip_id,
        user_id: revision.user_id,
        content: Some(clean_content.to_owned()),
    }))
}
