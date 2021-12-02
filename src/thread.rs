use actix_web::{error, get, post, web, Error, HttpResponse};
use askama_actix::Template;
use chrono::prelude::Utc;
use diesel::prelude::*;
use ruforo::models::{NewUgcRevision, Post, Thread, UgcRevision};
use ruforo::MyAppData;
use serde::Deserialize;

pub struct PostForTemplate {
    pub id: i32,
    pub thread_id: i32,
    pub ip_id: Option<i32>,
    pub ugc_id: i32,
    pub user_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub content: Option<String>,
}

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadTemplate {
    pub thread: super::proof::threads::Model,
    pub posts: Vec<PostForTemplate>,
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
    // use ruforo::schema::threads::dsl::*;
    // use ruforo::schema::ugc::dsl::*;
    //
    // let conn = match data.pool.get() {
    //     Ok(conn) => conn,
    //     Err(err) => return Err(error::ErrorInternalServerError(err)),
    // };

    use super::proof::posts::Entity as Post;
    use super::proof::threads::Entity as Thread;
    use sea_orm::{entity::*, query::*};

    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let db = sea_orm::Database::connect(db_url.to_owned())
        .await
        .expect("manual psql injection failed lol");

    let our_thread = match Thread::find_by_id(path.into_inner().0).one(&db).await {
        Ok(our_thread) => match our_thread {
            Some(our_thread) => our_thread,
            None => return Err(error::ErrorNotFound("Thread not found.")),
        },
        Err(_) => return Err(error::ErrorInternalServerError("Could not find thread.")),
    };

    // Load posts, their ugc associations, and their living revision.
    let our_posts: Vec<PostForTemplate> = match Post::find()
        .find_also_linked(super::proof::posts::PostToUgcRevision)
        .filter(super::proof::posts::Column::ThreadId.eq(our_thread.id))
        .all(&db)
        .await
    {
        Ok(posts) => posts
            .into_iter()
            .map(|post| PostForTemplate {
                id: post.0.id,
                created_at: post.0.created_at,
                updated_at: post
                    .1
                    .as_ref()
                    .map(|x| x.created_at)
                    .unwrap_or(post.0.created_at),
                user_id: post.0.user_id,
                thread_id: post.0.thread_id,
                ip_id: post.1.as_ref().map(|x| x.ip_id).unwrap_or(None),
                ugc_id: post.1.as_ref().map(|x| x.ugc_id).unwrap(),
                content: post
                    .1
                    .as_ref()
                    .map(|x| x.to_owned().content)
                    .unwrap_or(None),
            })
            .collect(),
        Err(_) => {
            return Err(error::ErrorInternalServerError(
                "Could not find posts for this thread.",
            ));
        }
    };

    // let our_ugc: Vec<Ugc> = ugc.get_results::<Ugc>(&conn).expect("error fetching ugc");
    // let our_ugc_revision: Vec<UgcRevision> = UgcRevision::belonging_to(&our_ugc)
    //     .load::<UgcRevision>(&conn)
    //     .expect("error fetching ugc revisions");
    //
    // // Smash them together to get a renderable struct.
    // let mut render_posts: Vec<RenderPost> = Vec::new();
    // for post in &*our_posts {
    //     render_posts.push(RenderPost {
    //         post,
    //         ugc: our_ugc_revision.iter().find(|x| x.ugc_id == post.ugc_id),
    //     });
    // }

    Ok(HttpResponse::Ok().body(
        ThreadTemplate {
            thread: our_thread,
            posts: our_posts,
        }
        .render()
        .unwrap(),
    ))
}
