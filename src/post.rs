use crate::frontend::TemplateToPubResponse;
use crate::orm::{posts, ugc_revisions, user_names};
use crate::thread::get_url_for_pos;
use crate::user::Client;
use crate::session::MainData;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use askama_actix::Template;
use sea_orm::{entity::*, query::*, DatabaseConnection, DbErr, FromQueryResult};
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
#[template(path = "post_delete.html")]
pub struct PostDeleteTemplate<'a> {
    pub post: &'a PostForTemplate,
}

#[derive(Template)]
#[template(path = "post_update.html")]
pub struct PostUpdateTemplate<'a> {
    pub post: &'a PostForTemplate,
}

#[derive(Deserialize)]
pub struct NewPostFormData {
    pub content: String,
}

#[get("/posts/{post_id}/delete")]
pub async fn delete_post(
    client: Client,
    data: web::Data<MainData<'_>>,
    path: web::Path<(i32,)>,
) -> Result<impl Responder, Error> {
    let post = get_post_for_template(&data.pool, path.into_inner().0)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    if !client.can_delete_post(&post) {
        return Err(error::ErrorForbidden(
            "You do not have permission to delete this post.",
        ));
    }

    PostDeleteTemplate { post: &post }.to_pub_response()
}

#[post("/posts/{post_id}/delete")]
pub async fn destroy_post(
    client: Client,
    data: web::Data<MainData<'_>>,
    path: web::Path<(i32,)>,
) -> Result<impl Responder, Error> {
    let post = get_post_for_template(&data.pool, path.into_inner().0)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    if !client.can_delete_post(&post) {
        return Err(error::ErrorForbidden(
            "You do not have permission to delete this post.",
        ));
    }

    Ok(HttpResponse::Found()
        .append_header(("Location", get_url_for_pos(post.thread_id, post.position)))
        .finish())
}

#[get("/posts/{post_id}/edit")]
pub async fn edit_post(
    client: Client,
    data: web::Data<MainData<'_>>,
    path: web::Path<(i32,)>,
) -> Result<impl Responder, Error> {
    let post: PostForTemplate = get_post_for_template(&data.pool, path.into_inner().0)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    if !client.can_update_post(&post) {
        return Err(error::ErrorForbidden(
            "You do not have permission to update this post.",
        ));
    }

    PostUpdateTemplate { post: &post }.to_pub_response()
}

#[post("/posts/{post_id}/edit")]
pub async fn update_post(
    client: Client,
    data: web::Data<MainData<'_>>,
    path: web::Path<(i32,)>,
    form: web::Form<NewPostFormData>,
) -> Result<impl Responder, Error> {
    use crate::ugc::{create_ugc_revision, NewUgcPartial};

    let post: PostForTemplate = get_post_for_template(&data.pool, path.into_inner().0)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    if !client.can_update_post(&post) {
        return Err(error::ErrorForbidden(
            "You do not have permission to update this post.",
        ));
    }

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
    .map_err(error::ErrorInternalServerError)?;

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

/// Returns the result of a query selecting for a post by id with adjoined templating data.
/// TODO: It would be nice if this returned just the selector.
pub async fn get_post_for_template(
    db: &DatabaseConnection,
    id: i32,
) -> Result<Option<PostForTemplate>, DbErr> {
    posts::Entity::find_by_id(id)
        .left_join(user_names::Entity)
        .column_as(user_names::Column::Name, "username")
        .left_join(ugc_revisions::Entity)
        .column_as(ugc_revisions::Column::Content, "content")
        .column_as(ugc_revisions::Column::IpId, "ip_id")
        .column_as(ugc_revisions::Column::CreatedAt, "updated_at")
        .into_model::<PostForTemplate>()
        .one(db)
        .await
}

async fn view_post(data: web::Data<MainData<'_>>, id: i32) -> Result<HttpResponse, Error> {
    let post = posts::Entity::find_by_id(id)
        .one(&data.pool)
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    Ok(HttpResponse::Found()
        .append_header(("Location", get_url_for_pos(post.thread_id, post.position)))
        .finish())
}
