use actix_web::{get, post, web, Error, HttpResponse};
use askama_actix::Template;
use diesel::prelude::*;
use ruforo::models::{NewUgcRevision, Thread, Ugc, UgcRevision};
use ruforo::MyAppData;
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
    data: web::Data<MyAppData<'static>>,
    path: web::Path<(i32,)>,
    form: web::Form<NewPostFormData>,
) -> Result<HttpResponse, Error> {
    use crate::ugc::create_ugc;

    let conn = data
        .pool
        .get()
        .expect("couldn't get db connection from pool");

    create_ugc(
        &conn,
        NewUgcRevision {
            ip_id: None,
            user_id: None,
            content: Some((&form.content).to_owned()),
        },
    )
    .expect("unable to insert new ugc");

    Ok(HttpResponse::Found()
        .append_header(("Location", format!("/threads/{}/", path.into_inner().0)))
        .finish())
}

#[get("/threads/{thread_id}/")]
pub async fn read_thread(
    path: web::Path<(i32,)>,
    data: web::Data<MyAppData<'static>>,
) -> Result<HttpResponse, Error> {
    use ruforo::schema::threads::dsl::*;
    use ruforo::schema::ugc::dsl::*;

    let conn = data
        .pool
        .get()
        .expect("couldn't get db connection from pool");

    let our_thread: Thread = threads
        .find(path.into_inner().0)
        .get_result::<Thread>(&conn)
        .expect("error fetching thread");
    let our_ugc: Vec<Ugc> = ugc.get_results::<Ugc>(&conn).expect("error fetching ugc");
    let our_ugc_revision: Vec<UgcRevision> = UgcRevision::belonging_to(&our_ugc)
        .load::<UgcRevision>(&conn)
        .expect("error fetching ugc revisions");

    Ok(HttpResponse::Ok().body(
        ThreadTemplate {
            thread: our_thread,
            posts: our_ugc_revision,
        }
        .render()
        .unwrap(),
    ))
}
