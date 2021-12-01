use actix_web::{get, post, web, Error, HttpResponse};
use askama_actix::Template;
use chrono::prelude::Utc;
use diesel::prelude::*;
use ruforo::models::{NewPost, NewThread, NewUgcRevision, Post, Thread, UgcRevision};
use ruforo::DbPool;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "forum.html")]
pub struct ForumTemplate {
    pub threads: Vec<Thread>,
}

#[derive(Deserialize)]
pub struct NewThreadFormData {
    title: String,
    subtitle: Option<String>,
    content: String,
}

#[post("/forums/post-thread")]
pub async fn create_thread(
    __pool: web::Data<DbPool>,
    __form: web::Form<NewThreadFormData>,
) -> Result<HttpResponse, Error> {
    use crate::ugc::create_ugc;
    use diesel::insert_into;
    use ruforo::schema::posts::dsl::*;
    use ruforo::schema::threads::dsl::*;

    let _conn = __pool.get().expect("couldn't get db connection from pool");

    // Step 1. Create the UGC.
    let _ugc: UgcRevision = create_ugc(
        __pool,
        NewUgcRevision {
            ip_id: None,
            user_id: None,
            content: Some((&__form.content).to_owned()),
        },
    )
    .expect("couldn't create ugc for new thread");

    // Step 2. Create a thread.
    let _thread: Thread = insert_into(threads)
        .values(NewThread {
            user_id: None,
            created_at: Utc::now().naive_utc(),
            title: __form.title.to_owned(),
            subtitle: __form.subtitle.to_owned(),
        })
        .get_result::<Thread>(&_conn)
        .expect("couldn't insert thread");

    // Step 3. Create a post with the correct associations.
    let _post: Post = insert_into(posts)
        .values(NewPost {
            thread_id: _thread.id,
            ugc_id: _ugc.id,
            user_id: None,
            created_at: Utc::now().naive_utc(),
        })
        .get_result::<Post>(&_conn)
        .expect("couldn't insert post");

    Ok(HttpResponse::Found()
        .append_header(("Location", format!("/threads/{}/", _thread.id)))
        .finish())
}

#[get("/forums/")]
pub async fn read_forum(__pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    use ruforo::schema::threads::dsl::*;

    let _conn = __pool.get().expect("couldn't get db connection from pool");
    let _threads: Vec<Thread> = threads
        .get_results::<Thread>(&_conn)
        .expect("error fetching ugc");

    Ok(HttpResponse::Ok().body(ForumTemplate { threads: _threads }.render().unwrap()))
}
