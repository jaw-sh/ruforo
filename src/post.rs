use crate::orm::posts::Model as Post;
use crate::orm::ugc_revisions::Model as UgcRevision;
use crate::MainData;
use actix_web::{error, get, post, web, Error, HttpResponse};
use askama_actix::Template;
use sea_orm::entity::*;
use serde::Deserialize;

pub struct PostForTemplate<'a> {
    pub id: i32,
    pub thread_id: i32,
    pub ip_id: Option<i32>,
    pub ugc_id: i32,
    pub user_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub content: Option<&'a str>,
}

impl<'a> PostForTemplate<'a> {
    pub fn from_orm(post: &Post, revision: &'a Option<UgcRevision>) -> Self {
        match revision {
            Some(r) => PostForTemplate {
                id: post.id,
                created_at: post.created_at,
                updated_at: r.created_at,
                user_id: post.user_id,
                thread_id: post.thread_id,
                ugc_id: post.ugc_id,
                ip_id: r.ip_id,
                content: Some(&r.content),
            },
            None => PostForTemplate {
                id: post.id,
                created_at: post.created_at,
                updated_at: post.created_at,
                user_id: post.user_id,
                thread_id: post.thread_id,
                ugc_id: post.ugc_id,
                ip_id: None,
                content: None,
            },
        }
    }
}

#[derive(Template)]
#[template(path = "post.html")]
pub struct PostFormTemplate<'a> {
    pub post: PostForTemplate<'a>,
}

#[derive(Deserialize)]
pub struct NewPostFormData {
    pub content: String,
}

#[get("/posts/{post_id}/edit")]
pub async fn edit_post(
    data: web::Data<MainData<'static>>,
    path: web::Path<(i32,)>,
) -> Result<HttpResponse, Error> {
    use crate::orm::posts;

    let result = posts::Entity::find_by_id(path.into_inner().0)
        .find_also_linked(super::orm::posts::PostToUgcRevision)
        .one(&data.pool)
        .await
        .map_err(|_| error::ErrorInternalServerError("Could not look up post."))?
        .ok_or(error::ErrorNotFound("Post not found."))?;

    Ok(HttpResponse::Ok().body(
        PostFormTemplate {
            post: PostForTemplate::from_orm(&result.0, &result.1),
        }
        .render()
        .unwrap(),
    ))
}

#[post("/posts/{post_id}/edit")]
pub async fn update_post(
    data: web::Data<MainData<'static>>,
    path: web::Path<(i32,)>,
    form: web::Form<NewPostFormData>,
) -> Result<HttpResponse, Error> {
    use crate::orm::posts;
    use crate::ugc::{create_ugc_revision, NewUgcPartial};

    let post = posts::Entity::find_by_id(path.into_inner().0)
        .one(&data.pool)
        .await
        .map_err(|_| error::ErrorInternalServerError("Could not look up post."))?
        .ok_or(error::ErrorNotFound("Post not found."))?;

    create_ugc_revision(
        &data.pool,
        post.ugc_id,
        NewUgcPartial {
            ip_id: None,
            user_id: None,
            content: &form.content,
        },
    )
    .await
    .map_err(|err| error::ErrorInternalServerError(err))?;

    Ok(HttpResponse::Found()
        .append_header(("Location", format!("/threads/{}/", post.thread_id)))
        .finish())
}
