use crate::attachment::AttachmentForTemplate;
use crate::init::get_db_pool;
use crate::middleware::ClientCtx;
use crate::orm::{posts, ugc_deletions, ugc_revisions, user_names};
use crate::thread::get_url_for_pos;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use askama_actix::{Template, TemplateToResponse};
use chrono::prelude::Utc;
use sea_orm::{entity::*, query::*, sea_query::Expr, DatabaseConnection, DbErr, FromQueryResult};
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
    // join ugc UgcDeletions
    pub deleted_by: Option<i32>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
    pub deleted_reason: Option<String>,
    // join user
    pub username: Option<String>,
}

impl PostForTemplate {
    pub fn render(&self) -> String {
        match &self.content {
            Some(content) => crate::bbcode::bbcode_to_html_ugly(&content),
            None => "".to_owned(),
        }
    }
}

#[derive(Template)]
#[template(path = "post_delete.html")]
pub struct PostDeleteTemplate<'a> {
    pub client: ClientCtx,
    pub post: &'a PostForTemplate,
}

#[derive(Template)]
#[template(path = "post_update.html")]
pub struct PostUpdateTemplate<'a> {
    pub client: ClientCtx,
    pub post: &'a PostForTemplate,
}

#[derive(Deserialize)]
pub struct NewPostFormData {
    pub content: String,
}

#[get("/posts/{post_id}/delete")]
pub async fn delete_post(client: ClientCtx, path: web::Path<i32>) -> Result<impl Responder, Error> {
    let post = get_post_for_template(get_db_pool(), path.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    if !client.can_delete_post(&post) {
        return Err(error::ErrorForbidden(
            "You do not have permission to delete this post.",
        ));
    }

    Ok(PostDeleteTemplate {
        client,
        post: &post,
    }
    .to_response())
}

#[post("/posts/{post_id}/delete")]
pub async fn destroy_post(
    client: ClientCtx,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let db = get_db_pool();
    let post = get_post_for_template(db, path.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    if !client.can_delete_post(&post) {
        return Err(error::ErrorForbidden(
            "You do not have permission to delete this post.",
        ));
    }

    if post.deleted_at.is_some() {
        ugc_deletions::Entity::update_many()
            .col_expr(ugc_deletions::Column::UserId, Expr::value(client.get_id()))
            .filter(ugc_deletions::Column::Id.eq(post.id))
            .exec(db)
            .await
            .map_err(error::ErrorInternalServerError)?;
    } else {
        ugc_deletions::Entity::insert(ugc_deletions::ActiveModel {
            id: Set(post.ugc_id),
            user_id: Set(client.get_id()),
            deleted_at: Set(Utc::now().naive_utc()),
            reason: Set(Some("Temporary reason holder".to_owned())),
        })
        .exec(db)
        .await
        .map_err(error::ErrorInternalServerError)?;

        // Spawn a thread to handle post-deletion work.
        actix_web::rt::spawn(async move {
            use crate::thread::update_thread_after_reply_is_deleted;

            // Update subsequent posts's position.
            posts::Entity::update_many()
                .col_expr(posts::Column::Position, Expr::cust("position - 1"))
                .filter(
                    Condition::all()
                        .add(posts::Column::ThreadId.eq(post.thread_id))
                        .add(posts::Column::Position.gt(post.position)),
                )
                .exec(db)
                .await;

            // Update post_count and last_post info.
            update_thread_after_reply_is_deleted(post.thread_id).await;
        });
    }

    Ok(HttpResponse::Found()
        .append_header(("Location", get_url_for_pos(post.thread_id, post.position)))
        .finish())
}

#[get("/posts/{post_id}/edit")]
pub async fn edit_post(client: ClientCtx, path: web::Path<i32>) -> Result<impl Responder, Error> {
    let post: PostForTemplate = get_post_for_template(get_db_pool(), path.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    if !client.can_update_post(&post) {
        return Err(error::ErrorForbidden(
            "You do not have permission to update this post.",
        ));
    }

    Ok(PostUpdateTemplate {
        client,
        post: &post,
    }
    .to_response())
}

#[post("/posts/{post_id}/edit")]
pub async fn update_post(
    client: ClientCtx,
    path: web::Path<i32>,
    form: web::Form<NewPostFormData>,
) -> Result<impl Responder, Error> {
    use crate::ugc::{create_ugc_revision, NewUgcPartial};

    let db = get_db_pool();

    let post: PostForTemplate = get_post_for_template(db, path.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    if !client.can_update_post(&post) {
        return Err(error::ErrorForbidden(
            "You do not have permission to update this post.",
        ));
    }

    create_ugc_revision(
        db,
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
pub async fn view_post_by_id(path: web::Path<i32>) -> Result<HttpResponse, Error> {
    view_post(path.into_inner()).await
}

// Permalink for a specific post.
#[get("/threads/{thread_id}/post-{post_id}")]
pub async fn view_post_in_thread(path: web::Path<(i32, i32)>) -> Result<HttpResponse, Error> {
    view_post(path.into_inner().1).await
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
        .left_join(ugc_deletions::Entity)
        .column_as(ugc_deletions::Column::UserId, "deleted_by")
        .column_as(ugc_deletions::Column::DeletedAt, "deleted_at")
        .column_as(ugc_deletions::Column::Reason, "deleted_reason")
        .into_model::<PostForTemplate>()
        .one(db)
        .await
}

async fn view_post(id: i32) -> Result<HttpResponse, Error> {
    let post = posts::Entity::find_by_id(id)
        .one(get_db_pool())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    Ok(HttpResponse::Found()
        .append_header(("Location", get_url_for_pos(post.thread_id, post.position)))
        .finish())
}
