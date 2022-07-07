use super::thread::get_url_for_pos;
use crate::db::get_db_pool;
use crate::middleware::ClientCtx;
use crate::orm::{posts, ugc_deletions, ugc_revisions};
use crate::ugc::{create_ugc_revision, NewUgcPartial};
use crate::user::Profile as UserProfile;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use askama_actix::{Template, TemplateToResponse};
use chrono::prelude::Utc;
use sea_orm::{entity::*, query::*, sea_query::Expr};
use sea_orm::{DatabaseConnection, DbErr, FromQueryResult, QueryFilter};
use serde::Deserialize;

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(delete_post)
        .service(destroy_post)
        .service(edit_post)
        .service(update_post)
        .service(view_post_by_id)
        .service(view_post_in_thread)
        .service(view_post_history)
        .service(view_post_history_diff);
}

#[derive(Deserialize)]
pub struct NewPostFormData {
    pub content: String,
}

/// A fully joined struct representing the post model and its relational d&ata.
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
    pub ugc_revision_id: Option<i32>,
    pub content: Option<String>,
    pub ip_id: Option<i32>,
    // join ugc deletions
    pub deleted_by: Option<i32>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
    pub deleted_reason: Option<String>,
}

impl PostForTemplate {}

#[derive(Template)]
#[template(path = "post_delete.html")]
pub struct PostDeleteTemplate<'a> {
    pub client: ClientCtx,
    pub post: &'a PostForTemplate,
}

#[derive(Template)]
#[template(path = "post_diff.html")]
pub struct PostDiffTemplate<'a> {
    pub client: ClientCtx,
    pub post: &'a PostForTemplate,
    pub revisions: &'a Vec<ugc_revisions::Model>,
    pub diff: &'a Vec<dissimilar::Chunk<'a>>,
}

#[derive(Template)]
#[template(path = "post_history.html")]
pub struct PostHistoryTemplate<'a> {
    pub client: ClientCtx,
    pub post: &'a PostForTemplate,
    pub revisions: &'a Vec<(UgcRevisionLineItem, Option<UserProfile>)>,
}

#[derive(Template)]
#[template(path = "post_update.html")]
pub struct PostUpdateTemplate<'a> {
    pub client: ClientCtx,
    pub post: &'a PostForTemplate,
}

#[derive(FromQueryResult)]
pub struct UgcRevisionLineItem {
    pub id: i32,
    pub user_id: Option<i32>,
    pub ugc_id: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
pub struct UgcRevisionDiffFormData {
    pub new: i32,
    pub old: i32,
}

impl UgcRevisionLineItem {
    pub async fn get_for_ugc_id(
        db: &DatabaseConnection,
        id: i32,
    ) -> Result<Vec<(Self, Option<UserProfile>)>, DbErr> {
        crate::user::find_also_user(
            ugc_revisions::Entity::find().filter(ugc_revisions::Column::UgcId.eq(id)),
            ugc_revisions::Column::UserId,
        )
        .into_model::<UgcRevisionLineItem, UserProfile>()
        .all(db)
        .await
    }
}

#[get("/posts/{post_id}/delete")]
pub async fn delete_post(client: ClientCtx, path: web::Path<i32>) -> Result<impl Responder, Error> {
    let db = get_db_pool();
    let (post, user) = get_post_and_author_for_template(db, path.into_inner())
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
    let (post, user) = get_post_and_author_for_template(db, path.into_inner())
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
            use super::thread::update_thread_after_reply_is_deleted;

            // Update subsequent posts's position.
            let _post_res = posts::Entity::update_many()
                .col_expr(posts::Column::Position, Expr::cust("position - 1"))
                .filter(
                    Condition::all()
                        .add(posts::Column::ThreadId.eq(post.thread_id))
                        .add(posts::Column::Position.gt(post.position)),
                )
                .exec(db)
                .await
                .map_err(|e| log::error!("destroy_post thread: {}", e));

            // Update post_count and last_post info.
            let _thread_res = update_thread_after_reply_is_deleted(post.thread_id)
                .await
                .map_err(|e| log::error!("destroy_post thread: {}", e));
        });
    }

    Ok(HttpResponse::Found()
        .append_header(("Location", get_url_for_pos(post.thread_id, post.position)))
        .finish())
}

#[get("/posts/{post_id}/edit")]
pub async fn edit_post(client: ClientCtx, path: web::Path<i32>) -> Result<impl Responder, Error> {
    let db = get_db_pool();
    let (post, user) = get_post_and_author_for_template(db, path.into_inner())
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
    let db = get_db_pool();
    let (post, user) = get_post_and_author_for_template(db, path.into_inner())
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
            user_id: client.get_id(),
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

/// Render post revisions as a line item table.
#[get("/posts/{post_id}/history")]
pub async fn view_post_history(
    client: ClientCtx,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let db = get_db_pool();
    let (post, user) = get_post_and_author_for_template(db, path.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    // TODO: Auth

    let revisions = UgcRevisionLineItem::get_for_ugc_id(db, post.ugc_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(PostHistoryTemplate {
        client,
        post: &post,
        revisions: &revisions,
    }
    .to_response())
}
/// Render post edits with diffs highlighted.
#[post("/posts/{post_id}/history")]
pub async fn view_post_history_diff(
    client: ClientCtx,
    path: web::Path<i32>,
    form: web::Form<UgcRevisionDiffFormData>,
) -> Result<impl Responder, Error> {
    let db = get_db_pool();
    let (post, user) = get_post_and_author_for_template(db, path.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorNotFound("Post not found."))?;

    let revisions = ugc_revisions::Entity::find()
        .filter(ugc_revisions::Column::UgcId.eq(post.ugc_id))
        .filter(ugc_revisions::Column::Id.is_in([form.old, form.new]))
        .limit(2)
        .order_by_desc(ugc_revisions::Column::CreatedAt)
        .all(db)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // TODO: Auth

    if revisions.len() < 2 {
        return Err(error::ErrorBadRequest(
            "Requested revisions either do not exist or are not attached to this resource as expected.",
        ));
    }

    let diff = dissimilar::diff(&revisions[1].content, &revisions[0].content);

    Ok(PostDiffTemplate {
        client,
        post: &post,
        revisions: &revisions,
        diff: &diff,
    }
    .to_response())
}

/// Returns the result of a query selecting for a post by id with adjoined templating data.
/// TODO: It would be nice if this returned just the selector.
pub async fn get_post_and_author_for_template(
    db: &DatabaseConnection,
    id: i32,
) -> Result<Option<(PostForTemplate, Option<UserProfile>)>, DbErr> {
    crate::user::find_also_user(
        posts::Entity::find_by_id(id)
            .left_join(ugc_revisions::Entity)
            .column_as(ugc_revisions::Column::Id, "ugc_revision_id")
            .column_as(ugc_revisions::Column::Content, "content")
            .column_as(ugc_revisions::Column::IpId, "ip_id")
            .column_as(ugc_revisions::Column::CreatedAt, "updated_at")
            .left_join(ugc_deletions::Entity)
            .column_as(ugc_deletions::Column::UserId, "deleted_by")
            .column_as(ugc_deletions::Column::DeletedAt, "deleted_at")
            .column_as(ugc_deletions::Column::Reason, "deleted_reason"),
        posts::Column::UserId,
    )
    .into_model::<PostForTemplate, UserProfile>()
    .one(db)
    .await
}

pub async fn get_replies_and_author_for_template(
    db: &DatabaseConnection,
    id: i32,
    page: i32,
) -> Result<Vec<(PostForTemplate, Option<UserProfile>)>, DbErr> {
    crate::user::find_also_user(
        posts::Entity::find()
            .left_join(ugc_revisions::Entity)
            .column_as(ugc_revisions::Column::Id, "ugc_revision_id")
            .column_as(ugc_revisions::Column::Content, "content")
            .column_as(ugc_revisions::Column::IpId, "ip_id")
            .column_as(ugc_revisions::Column::CreatedAt, "updated_at")
            .left_join(ugc_deletions::Entity)
            .column_as(ugc_deletions::Column::UserId, "deleted_by")
            .column_as(ugc_deletions::Column::DeletedAt, "deleted_at")
            .column_as(ugc_deletions::Column::Reason, "deleted_reason"),
        posts::Column::UserId,
    )
    .filter(posts::Column::ThreadId.eq(id))
    .filter(posts::Column::Position.between(
        (page - 1) * super::thread::POSTS_PER_PAGE + 1,
        page * super::thread::POSTS_PER_PAGE,
    ))
    .order_by_asc(posts::Column::Position)
    .order_by_asc(posts::Column::CreatedAt)
    .into_model::<PostForTemplate, UserProfile>()
    .all(db)
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
