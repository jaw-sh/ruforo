use actix_web::{get, web, Error, HttpResponse};
use askama_actix::Template;
use diesel::prelude::*;
use ruforo::models::{Ugc, UgcRevision};
use ruforo::DbPool;

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadTemplate {
    pub posts: Vec<UgcRevision>,
}

#[get("/thread")]
pub async fn thread(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
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
