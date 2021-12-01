use actix_web::{get, post, web, Error, HttpResponse};
use askama_actix::Template;
use diesel::prelude::*;
use ruforo::models::Thread;
use ruforo::DbPool;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "forum.html")]
pub struct ForumTemplate {
    pub threads: Vec<Thread>,
}

#[derive(Deserialize)]
pub struct NewPostFormData {
    content: String,
}

#[get("/forum")]
pub async fn read_forum(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
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
