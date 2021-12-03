use crate::proof::threads;
use crate::proof::threads::Entity as Thread;
use actix_web::{error, get, post, web, Error, HttpResponse};
use askama_actix::Template;
use chrono::prelude::Utc;
use ruforo::MainData;
use sea_orm::QueryFilter;
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
    data: web::Data<MainData<'static>>,
    path: web::Path<(i32,)>,
    form: web::Form<NewPostFormData>,
) -> Result<HttpResponse, Error> {
    use crate::ugc::create_ugc;

    let our_thread: Thread = match threads
        .find(path.into_inner().0)
        .get_result::<Thread>(&data.pool)
    {
        Ok(our_thread) => our_thread,
        Err(_) => return Err(error::ErrorNotFound("Thread not found.")),
    };

    let ugc_revision: UgcRevision = match create_ugc(
        &data.pool,
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
        .get_result::<Post>(&data.pool)
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
    data: web::Data<MainData<'static>>,
) -> Result<HttpResponse, Error> {
    use super::proof::posts::Entity as Post;
    use sea_orm::{entity::*, query::*};

    let our_thread = match Thread::find_by_id(path.into_inner().0)
        .one(&data.pool)
        .await
    {
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
        .all(&data.pool)
        .await
    {
        Ok(posts) => posts
            .into_iter()
            .map(|post| match post.1 {
                Some(ugc) => PostForTemplate {
                    id: post.0.id,
                    created_at: post.0.created_at,
                    updated_at: ugc.created_at,
                    user_id: post.0.user_id,
                    thread_id: post.0.thread_id,
                    ugc_id: post.0.ugc_id,
                    ip_id: ugc.ip_id,
                    content: ugc.content.to_owned(),
                },
                None => PostForTemplate {
                    id: post.0.id,
                    created_at: post.0.created_at,
                    updated_at: post.0.created_at,
                    user_id: post.0.user_id,
                    thread_id: post.0.thread_id,
                    ugc_id: post.0.ugc_id,
                    ip_id: None,
                    content: None,
                },
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
