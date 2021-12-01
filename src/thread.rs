use actix_web::{get, post, web, Error, HttpResponse};
use askama_actix::Template;
use diesel::prelude::*;
use ruforo::models::{NewUgcRevision, Ugc, UgcRevision};
use ruforo::DbPool;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadTemplate {
    pub posts: Vec<UgcRevision>,
}

#[derive(Deserialize)]
pub struct NewPostFormData {
    content: String,
}

#[post("/thread/post-reply")]
pub async fn create_reply(
    pool: web::Data<DbPool>,
    form: web::Form<NewPostFormData>,
) -> Result<HttpResponse, Error> {
    use crate::ugc::create_ugc;

    create_ugc(
        pool,
        NewUgcRevision {
            ip_id: None,
            user_id: None,
            content: Some((&form.content).to_owned()),
        },
    )
    .expect("unable to insert new ugc");

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
