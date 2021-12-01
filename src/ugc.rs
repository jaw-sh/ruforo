use actix_web::{web, Error};
use chrono::prelude::Utc;
use diesel::prelude::*;
use ruforo::models::{NewUgcRevision, NewUgcRevisionWithContext, Ugc, UgcRevision};
use ruforo::DbPool;

pub fn create_ugc(
    _pool: web::Data<DbPool>,
    _revision: NewUgcRevision,
) -> Result<UgcRevision, Error> {
    use diesel::insert_into;
    use ruforo::schema::ugc::dsl::*;
    use ruforo::schema::ugc_revisions::dsl::*;

    let conn = _pool.get().expect("couldn't get db connection from pool");

    // Insert new UGC reference with only default values.
    let new_ugc = insert_into(ugc)
        .default_values()
        .get_result::<Ugc>(&conn)
        .expect("couldn't insert ugc");

    // Use supplied _revision to build a UGC Revision with references we just created.
    let new_ugc_revision = insert_into(ugc_revisions)
        .values(NewUgcRevisionWithContext {
            ugc_id: new_ugc.id,
            ip_id: _revision.ip_id,
            user_id: _revision.user_id,
            created_at: Utc::now().naive_utc(),
            content: _revision.content,
        })
        .get_result::<UgcRevision>(&conn)
        .expect("couldn't insert ugc revision");

    // Update the new UGC to point at the living revision we just inserted.
    diesel::update(&new_ugc)
        .set(ugc_revision_id.eq(Some(new_ugc_revision.id)))
        .execute(&conn)
        .expect("couldn't update ugc with living revision id");

    Ok(new_ugc_revision)
}
