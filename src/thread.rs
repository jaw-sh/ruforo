use actix_web::{get, post, web, Error, HttpResponse};
use askama_actix::Template;
use diesel::prelude::*;
use ruforo::models::{NewUgcRevision, Thread, Ugc, UgcRevision};
use ruforo::DbPool;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadTemplate {
    pub thread: Thread,
    pub posts: Vec<UgcRevision>,
}

#[derive(Deserialize)]
pub struct NewPostFormData {
    content: String,
}

#[post("/threads/{thread_id}/post-reply")]
pub async fn create_reply(
    __path: web::Path<(i32,)>,
    __pool: web::Data<DbPool>,
    __form: web::Form<NewPostFormData>,
) -> Result<HttpResponse, Error> {
    use crate::ugc::create_ugc;

    let _conn = __pool.get().expect("couldn't get db connection from pool");

    create_ugc(
        __pool,
        NewUgcRevision {
            ip_id: None,
            user_id: None,
            content: Some((&__form.content).to_owned()),
        },
    )
    .expect("unable to insert new ugc");

    Ok(HttpResponse::Found()
        .append_header(("Location", format!("/threads/{}/", __path.into_inner().0)))
        .finish())
}

#[get("/threads/{thread_id}/")]
pub async fn read_thread(
    __path: web::Path<(i32,)>,
    __pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    use ruforo::schema::threads::dsl::*;
    use ruforo::schema::ugc::dsl::*;

    let _conn = __pool.get().expect("couldn't get db connection from pool");
    let _thread: Thread = threads
        .find(__path.into_inner().0)
        .get_result::<Thread>(&_conn)
        .expect("error fetching thread");
    let _ugc: Vec<Ugc> = ugc.get_results::<Ugc>(&_conn).expect("error fetching ugc");
    let _ugc_revision: Vec<UgcRevision> = UgcRevision::belonging_to(&_ugc)
        .load::<UgcRevision>(&_conn)
        .expect("error fetching ugc revisions");

    Ok(HttpResponse::Ok().body(
        ThreadTemplate {
            thread: _thread,
            posts: _ugc_revision,
        }
        .render()
        .unwrap(),
    ))
}
