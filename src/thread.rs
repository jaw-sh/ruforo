use crate::orm::posts::Entity as Post;
use crate::orm::threads::Entity as Thread;
use crate::post::{NewPostFormData, PostForTemplate};
use crate::MainData;
use actix_web::{error, get, post, web, Error, HttpResponse};
use askama_actix::Template;
use sea_orm::entity::*;
use sea_orm::QueryFilter;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewThreadFormData {
    pub title: String,
    pub subtitle: Option<String>,
    pub content: String,
}

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadTemplate<'a> {
    pub thread: super::orm::threads::Model,
    pub posts: Vec<PostForTemplate<'a>>,
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
        .ok_or(error::ErrorNotFound("Thread not found."))?;

    let ugc_revision = create_ugc(
        &data.pool,
        NewUgcPartial {
            ip_id: None,
            user_id: None,
            content: &form.content,
        },
    )
    .await
    .map_err(|err| error::ErrorInternalServerError(err))?;

    posts::ActiveModel {
        thread_id: Set(our_thread.id),
        ugc_id: ugc_revision.ugc_id,
        created_at: ugc_revision.created_at,
        ..Default::default()
    }
    .insert(&data.pool)
    .await
    .map_err(|err| error::ErrorInternalServerError(err))?;

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
        .ok_or(error::ErrorNotFound("Thread not found."))?;

    // Load posts, their ugc associations, and their living revision.
    let results = Post::find()
        .find_also_linked(super::orm::posts::PostToUgcRevision)
        .filter(super::orm::posts::Column::ThreadId.eq(our_thread.id))
        .all(&data.pool)
        .await
        .map_err(|_| error::ErrorInternalServerError("Could not find posts for this thread."))?;

    let mut our_posts = Vec::new();
    for (p, u) in &results {
        our_posts.push(PostForTemplate::from_orm(&p, &u));
    }

    Ok(HttpResponse::Ok().body(
        ThreadTemplate {
            thread: our_thread,
            posts: our_posts,
        }
        .render()
        .unwrap(),
    ))
}

pub fn validate_thread_form(
    form: web::Form<NewThreadFormData>,
) -> Result<NewThreadFormData, Error> {
    let title = form.title.trim().to_owned();
    let subtitle = form.subtitle.to_owned().filter(|x| !x.is_empty());

    if title.is_empty() {
        return Err(error::ErrorUnprocessableEntity(
            "Threads must have a title.",
        ));
    }

    Ok(NewThreadFormData {
        title,
        subtitle,
        content: form.content.to_owned(),
    })
}
