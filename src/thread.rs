use actix_web::{error, get, post, web, Error, HttpResponse};
use askama_actix::Template;
use diesel::prelude::*;
use ruforo::models::{NewUgcRevision, Post, RenderPost, Thread, Ugc, UgcRevision};
use ruforo::MyAppData;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadTemplate {
    pub thread: Thread,
    pub posts: Vec<RenderPost>,
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

    let thread_match = threads
        .find(path.into_inner().0)
        .get_result::<Thread>(&conn);

    if thread_match.is_err() {
        return Err(error::ErrorNotFound("Thread not found."));
    }

    let our_thread: Thread = thread_match.unwrap();
    let our_posts = Post::belonging_to(&our_thread)
        .load::<Post>(&conn)
        .expect("error fetching posts");

    let our_ugc: Vec<Ugc> = ugc.get_results::<Ugc>(&conn).expect("error fetching ugc");
    let our_ugc_revision: Vec<UgcRevision> = UgcRevision::belonging_to(&our_ugc)
        .load::<UgcRevision>(&conn)
        .expect("error fetching ugc revisions");

    let mut render_posts: Vec<RenderPost> = Vec::new();
    for post in our_posts {
        render_posts.push(RenderPost {
            post: post,
            ugc: our_ugc_revision.iter().find(|x| x.ugc_id == post.ugc_id),
        });
    }

    Ok(HttpResponse::Ok().body(
        ThreadTemplate {
            thread: our_thread,
            posts: render_posts,
        }
        .render()
        .unwrap(),
    ))
}
