use actix_web::{error, get, post, web, Error, HttpResponse};
use askama_actix::Template;
use chrono::prelude::Utc;
use diesel::prelude::*;
use ruforo::models::{NewUgcRevision, Post, RenderPost, Thread, Ugc, UgcRevision};
use ruforo::MyAppData;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadTemplate<'a> {
    pub thread: Thread,
    pub posts: Vec<RenderPost<'a>>,
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
    use diesel::insert_into;
    use ruforo::models::NewPost;
    use ruforo::schema::posts::dsl::*;
    use ruforo::schema::threads::dsl::*;

    let conn = match data.pool.get() {
        Ok(conn) => conn,
        Err(err) => return Err(error::ErrorInternalServerError(err)),
    };

    let our_thread: Thread = match threads
        .find(path.into_inner().0)
        .get_result::<Thread>(&conn)
    {
        Ok(our_thread) => our_thread,
        Err(_) => return Err(error::ErrorNotFound("Thread not found.")),
    };

    let ugc_revision: UgcRevision = match create_ugc(
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

    match insert_into(posts)
        .values(NewPost {
            thread_id: our_thread.id,
            ugc_id: ugc_revision.ugc_id,
            created_at: Utc::now().naive_utc(),
            user_id: None,
        })
        .get_result::<Post>(&conn)
    {
        Ok(_) => Ok(HttpResponse::Found()
            .append_header(("Location", format!("/threads/{}/", our_thread.id)))
            .finish()),
        Err(err) => return Err(error::ErrorInternalServerError(err)),
    }
}

#[get("/threads/{thread_id}/")]
pub async fn read_thread(
    path: web::Path<(i32,)>,
    data: web::Data<MyAppData<'static>>,
) -> Result<HttpResponse, Error> {
    use ruforo::schema::threads::dsl::*;
    use ruforo::schema::ugc::dsl::*;

    let conn = match data.pool.get() {
        Ok(conn) => conn,
        Err(err) => return Err(error::ErrorInternalServerError(err)),
    };

    let our_thread: Thread = match threads
        .find(path.into_inner().0)
        .get_result::<Thread>(&conn)
    {
        Ok(our_thread) => our_thread,
        Err(_) => return Err(error::ErrorNotFound("Thread not found.")),
    };

    // Load posts, their ugc associations, and their living revision.
    let our_posts: Vec<Post> = Post::belonging_to(&our_thread)
        .load::<Post>(&conn)
        .expect("error fetching posts");
    let our_ugc: Vec<Ugc> = ugc.get_results::<Ugc>(&conn).expect("error fetching ugc");
    let our_ugc_revision: Vec<UgcRevision> = UgcRevision::belonging_to(&our_ugc)
        .load::<UgcRevision>(&conn)
        .expect("error fetching ugc revisions");

    // Smash them together to get a renderable struct.
    let mut render_posts: Vec<RenderPost> = Vec::new();
    for post in &*our_posts {
        render_posts.push(RenderPost {
            post,
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
