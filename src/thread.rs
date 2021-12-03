use crate::orm::posts::Entity as Post;
use crate::orm::threads::Entity as Thread;
use crate::MainData;
use actix_web::{error, get, post, web, Error, HttpResponse};
use askama_actix::Template;
use sea_orm::QueryFilter;
use sea_orm::entity::*;
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
    pub thread: super::orm::threads::Model,
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
    use crate::orm::posts;
    use crate::ugc::{create_ugc, NewUgcPartial};

    let our_thread = Thread::find_by_id(path.into_inner().0)
        .one(&data.pool)
        .await
        .map_err(|_| error::ErrorInternalServerError("Could not look up thread."))?
        .ok_or_else(|| error::ErrorNotFound("Thread not found."))?;

    let ugc_revision = create_ugc(
        &data.pool,
        NewUgcPartial {
            ip_id: None,
            user_id: None,
            content: form.content.to_owned(),
        },
    )
    .await
    .map_err(|err| error::ErrorInternalServerError(err))?;

    posts::ActiveModel {
        thread_id: Set(our_thread.id),
        ugc_id: ugc_revision.id,
        created_at: ugc_revision.created_at.to_owned(),
        ..Default::default()
    }
    .insert(&data.pool)
    .await
    .map_err(|_| error::ErrorInternalServerError("Failed to insert new post."))?;

    Ok(HttpResponse::Found()
        .append_header(("Location", format!("/threads/{}/", our_thread.id)))
        .finish())
}

#[get("/threads/{thread_id}/")]
pub async fn read_thread(
    path: web::Path<(i32,)>,
    data: web::Data<MainData<'static>>,
) -> Result<HttpResponse, Error> {
    let our_thread = Thread::find_by_id(path.into_inner().0)
        .one(&data.pool)
        .await
        .map_err(|_| error::ErrorInternalServerError("Could not look up thread."))?
        .ok_or_else(|| error::ErrorNotFound("Thread not found."))?;

    // Load posts, their ugc associations, and their living revision.
    let our_posts: Vec<PostForTemplate> = match Post::find()
        .find_also_linked(super::orm::posts::PostToUgcRevision)
        .filter(super::orm::posts::Column::ThreadId.eq(our_thread.id))
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
                    content: Some(ugc.content.to_owned()),
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

    Ok(HttpResponse::Ok().body(
        ThreadTemplate {
            thread: our_thread,
            posts: our_posts,
        }
        .render()
        .unwrap(),
    ))
}
