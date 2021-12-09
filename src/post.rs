use crate::orm::{posts, ugc_revisions, users};
use crate::MainData;
use actix_web::{error, get, post, web, Error, HttpResponse};
use askama_actix::Template;
use sea_orm::{entity::*, query::*, FromQueryResult};
use serde::Deserialize;

/// A fully joined struct representing the post model and its relational data.
#[derive(Debug, FromQueryResult)]
pub struct PostForTemplate {
    pub id: i32,
    pub thread_id: i32,
    pub ugc_id: i32,
    pub user_id: Option<i32>,
    pub position: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    // join ugc
    pub content: Option<String>,
    pub ip_id: Option<i32>,
    // join user
    pub username: Option<String>,
}

#[derive(Template)]
#[template(path = "post.html")]
pub struct PostEditTemplate<'a> {
    pub post: &'a PostForTemplate,
}

#[derive(Deserialize)]
pub struct NewPostFormData {
    pub content: String,
}

#[get("/posts/{post_id}/edit")]
pub async fn edit_post(
    data: web::Data<MainData<'_>>,
    path: web::Path<(i32,)>,
) -> Result<HttpResponse, Error> {
    let post: PostForTemplate = posts::Entity::find_by_id(path.into_inner().0)
        .left_join(users::Entity)
        .column_as(users::Column::Name, "username")
        .left_join(ugc_revisions::Entity)
        .column_as(ugc_revisions::Column::Content, "content")
        .column_as(ugc_revisions::Column::IpId, "ip_id")
        .column_as(ugc_revisions::Column::CreatedAt, "updated_id")
        .into_model::<PostForTemplate>()
        .one(&data.pool)
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    Ok(HttpResponse::Ok().body(PostEditTemplate { post: &post }.render().unwrap()))
}

#[post("/posts/{post_id}/edit")]
pub async fn update_post(
    data: web::Data<MainData<'_>>,
    path: web::Path<(i32,)>,
    form: web::Form<NewPostFormData>,
) -> Result<HttpResponse, Error> {
    use crate::orm::posts;
    use crate::ugc::{create_ugc_revision, NewUgcPartial};

    let post = posts::Entity::find_by_id(path.into_inner().0)
        .one(&data.pool)
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

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

async fn view_post(data: web::Data<MainData<'_>>, id: i32) -> Result<HttpResponse, Error> {
    use crate::thread::get_url_for_pos;

    let post = posts::Entity::find_by_id(id)
        .one(&data.pool)
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    Ok(HttpResponse::Found()
        .append_header(("Location", get_url_for_pos(post.thread_id, post.position)))
        .finish())
}

#[get("/posts/{post_id}")]
pub async fn view_post_by_id(
    data: web::Data<MainData<'_>>,
    path: web::Path<(i32,)>,
) -> Result<HttpResponse, Error> {
    view_post(data, path.into_inner().0).await
}

// Permalink for a specific post.
#[get("/threads/{thread_id}/post-{post_id}")]
pub async fn view_post_in_thread(
    data: web::Data<MainData<'_>>,
    path: web::Path<(i32, i32)>,
) -> Result<HttpResponse, Error> {
    view_post(data, path.into_inner().1).await
}
