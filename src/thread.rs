use actix_web::{get, post, web, Error, HttpResponse};
use askama_actix::Template;
use chrono::prelude::Utc;
use diesel::prelude::*;
use ruforo::models::{NewUgcRevision, Ugc, UgcRevision};
use ruforo::DbPool;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewPostFormData {
    content: String,
}

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadTemplate {
    pub posts: Vec<UgcRevision>,
}

#[post("/thread/post-reply")]
pub async fn create_reply(
    pool: web::Data<DbPool>,
    form: web::Form<NewPostFormData>,
) -> Result<HttpResponse, Error> {
    use diesel::insert_into;
    use ruforo::schema::ugc::dsl::*;
    use ruforo::schema::ugc_revisions::dsl::*;

    let conn = pool.get().expect("couldn't get db connection from pool");
    let new_ugc = insert_into(ugc)
        .default_values()
        .get_result::<Ugc>(&conn)
        .expect("couldn't insert ugc");

    let new_ugc_model = NewUgcRevision {
        ugc_id: new_ugc.id,
        ip_id: None,
        user_id: None,
        created_at: Utc::now().naive_utc(),
        content: Some((&form.content).to_owned()),
    };
    let new_ugc_revision = insert_into(ugc_revisions)
        .values(new_ugc_model)
        .get_result::<UgcRevision>(&conn)
        .expect("couldn't insert ugc revision");

    // Both entities are being created at the same time,
    // so we need to update the ugc to point at the new living revision.
    diesel::update(&new_ugc)
        .set(ugc_revision_id.eq(Some(new_ugc_revision.id)))
        .execute(&conn)
        .expect("couldn't update ugc with living revision id");

    Ok(HttpResponse::Found()
        .append_header(("Location", "/thread"))
        .finish())
}

#[get("/thread")]
pub async fn read_thread(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    use ruforo::schema::ugc::dsl::*;

    let conn = pool.get().expect("couldn't get db connection from pool");
    let posts: Vec<Ugc> = ugc.get_results::<Ugc>(&conn).expect("error fetching ugc");
    let post_content: Vec<UgcRevision> = UgcRevision::belonging_to(&posts)
        .load::<UgcRevision>(&conn)
        .expect("error fetching ugc revisions");

    Ok(HttpResponse::Ok().body(
        ThreadTemplate {
            posts: post_content,
        }
        .render()
        .unwrap(),
    ))
}
