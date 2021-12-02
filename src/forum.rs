use actix_web::{error, get, post, web, Error, HttpResponse};
use askama_actix::Template;
use chrono::prelude::Utc;
use diesel::prelude::*;
use ruforo::models::Thread;
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
    use ruforo::models::{NewPost, NewThread, NewUgcRevision, Post, UgcRevision};
    use ruforo::schema::posts::dsl::*;
    use ruforo::schema::threads::dsl::*;

    let conn = match data.pool.get() {
        Ok(conn) => conn,
        Err(err) => return Err(error::ErrorInternalServerError(err)),
    };

    // Step 1. Create the UGC.
    let revision: UgcRevision = match create_ugc(
        &conn,
        NewUgcRevision {
            ip_id: None,
            user_id: None,
            content: Some((&form.content).to_owned()),
        },
    ) {
        Ok(revision) => revision,
        Err(err) => return Err(err),
    };

    // Step 2. Create a thread.
    let thread: Thread = match insert_into(threads)
        .values(NewThread {
            user_id: None,
            created_at: Utc::now().naive_utc(),
            title: form.title.trim().to_owned(),
            subtitle: form
                .subtitle
                .to_owned()
                .map(|s| s.trim().to_string())
                .filter(|s| s.len() != 0),
        })
        .get_result::<Thread>(&conn)
    {
        Ok(thread) => thread,
        Err(err) => return Err(error::ErrorInternalServerError(err)),
    };

    // Step 3. Create a post with the correct associations.
    insert_into(posts)
        .values(NewPost {
            thread_id: thread.id,
            ugc_id: revision.id,
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

    let conn = match data.pool.get() {
        Ok(conn) => conn,
        Err(err) => return Err(error::ErrorInternalServerError(err)),
    };

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
