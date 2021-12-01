use actix_web::Error;
use chrono::prelude::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use ruforo::models::{NewUgcRevision, NewUgcRevisionWithContext, Post, Ugc, UgcRevision};

pub fn create_ugc(db: &PgConnection, revision: NewUgcRevision) -> Result<UgcRevision, Error> {
    use diesel::insert_into;
    use ruforo::schema::ugc::dsl::*;
    use ruforo::schema::ugc_revisions::dsl::*;

    // Insert new UGC reference with only default values.
    let new_ugc = insert_into(ugc)
        .default_values()
        .get_result::<Ugc>(db)
        .expect("couldn't insert ugc");

    // Use supplied _revision to build a UGC Revision with references we just created.
    let new_ugc_revision = insert_into(ugc_revisions)
        .values(NewUgcRevisionWithContext {
            ugc_id: new_ugc.id,
            ip_id: revision.ip_id,
            user_id: revision.user_id,
            created_at: Utc::now().naive_utc(),
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
