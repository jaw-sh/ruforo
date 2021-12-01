use actix_web::{get, post, web, Error, HttpResponse};
use askama_actix::Template;
use chrono::prelude::Utc;
use diesel::prelude::*;
use ruforo::models::{NewPost, NewThread, NewUgcRevision, Post, Thread, UgcRevision};
use ruforo::MyAppData;
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
    data: web::Data<MyAppData<'static>>,
    form: web::Form<NewThreadFormData>,
) -> Result<HttpResponse, Error> {
    use crate::ugc::create_ugc;
    use diesel::insert_into;
    use ruforo::schema::posts::dsl::*;
    use ruforo::schema::threads::dsl::*;

    let conn = data
        .pool
        .get()
        .expect("couldn't get db connection from pool");

    // Step 1. Create the UGC.
    let ugc: UgcRevision = create_ugc(
        &conn,
        NewUgcRevision {
            ip_id: None,
            user_id: None,
            content: Some((&form.content).to_owned()),
        },
    )
    .expect("couldn't create ugc for new thread");

    // Step 2. Create a thread.
    let thread: Thread = insert_into(threads)
        .values(NewThread {
            user_id: None,
            created_at: Utc::now().naive_utc(),
            title: form.title.to_owned(),
            subtitle: form.subtitle.to_owned(),
        })
        .get_result::<Thread>(&conn)
        .expect("couldn't insert thread");

    // Step 3. Create a post with the correct associations.
    insert_into(posts)
        .values(NewPost {
            thread_id: thread.id,
            ugc_id: ugc.id,
            user_id: None,
            created_at: Utc::now().naive_utc(),
        })
        .get_result::<Post>(&conn)
        .expect("couldn't insert post");

    Ok(HttpResponse::Found()
        .append_header(("Location", format!("/threads/{}/", thread.id)))
        .finish())
}

#[get("/forums/")]
pub async fn read_forum(data: web::Data<MyAppData<'static>>) -> Result<HttpResponse, Error> {
    use ruforo::schema::threads::dsl::*;

    let conn = data
        .pool
        .get()
        .expect("couldn't get db connection from pool");
    let our_threads: Vec<Thread> = threads
        .get_results::<Thread>(&conn)
        .expect("error fetching ugc");

    Ok(HttpResponse::Ok().body(
        ForumTemplate {
            threads: our_threads,
        }
        .render()
        .unwrap(),
    ))
}
