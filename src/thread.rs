use crate::attachment::AttachmentForTemplate;
use crate::init::get_db_pool;
use crate::middleware::ClientCtx;
use crate::orm::posts::Entity as Post;
use crate::orm::threads::Entity as Thread;
use crate::orm::{posts, threads, ugc_deletions, ugc_revisions};
use crate::post::{NewPostFormData, PostForTemplate};
use crate::template::{Paginator, PaginatorToHtml};
use actix_multipart::Multipart;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use askama_actix::{Template, TemplateToResponse};
use sea_orm::{entity::*, query::*, sea_query::Expr, DbErr, FromQueryResult, QueryFilter};
use serde::Deserialize;
use std::{collections::HashMap, str};

#[derive(Debug, FromQueryResult)]
pub struct ThreadForTemplate {
    pub id: i32,
    pub user_id: Option<i32>,
    pub created_at: chrono::naive::NaiveDateTime,
    pub title: String,
    pub subtitle: Option<String>,
    pub view_count: i32,
    pub post_count: i32,
    pub first_post_id: i32,
    pub last_post_id: i32,
    pub last_post_at: chrono::naive::NaiveDateTime,
    // join user
    pub username: Option<String>,
}

#[derive(Deserialize)]
pub struct NewThreadFormData {
    pub title: String,
    pub subtitle: Option<String>,
    pub content: String,
}

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadTemplate<'a> {
    pub client: ClientCtx,
    pub thread: crate::orm::threads::Model,
    pub paginator: Paginator,
    pub posts: &'a Vec<PostForTemplate>,
    pub attachments: &'a HashMap<i32, Vec<AttachmentForTemplate>>,
}

mod filters {
    pub fn ugc(s: &str) -> ::askama::Result<String> {
        Ok(crate::bbcode::bbcode_to_html(s))
    }
}

// TODO: Dynamic page sizing.
const POSTS_PER_PAGE: i32 = 20;

/// Returns which human-readable page number this position will appear in.
pub fn get_page_for_pos(pos: i32) -> i32 {
    ((std::cmp::max(1, pos) - 1) / POSTS_PER_PAGE) + 1
}

pub fn get_pages_in_thread(cnt: i32) -> i32 {
    ((std::cmp::max(1, cnt) - 1) / POSTS_PER_PAGE) + 1
}

/// Returns the relative URL for the thread at this position.
pub fn get_url_for_pos(thread_id: i32, pos: i32) -> String {
    let page = get_page_for_pos(pos);
    format!(
        "/threads/{}/{}",
        thread_id,
        if page == 1 {
            "".to_owned()
        } else {
            format!("page-{}", page)
        }
    )
}

/// Returns a Responder for a thread at a specific page.
async fn get_thread_and_replies_for_page(
    client: ClientCtx,
    thread_id: i32,
    page: i32,
) -> Result<impl Responder, Error> {
    use crate::attachment::get_attachments_for_ugc_by_id;
    use crate::orm::user_names;

    let db = get_db_pool();
    let thread = Thread::find_by_id(thread_id)
        .one(db)
        .await
        .map_err(|_| error::ErrorInternalServerError("Could not look up thread."))?
        .ok_or_else(|| error::ErrorNotFound("Thread not found."))?;

    // Update thread to include views.
    actix_web::rt::spawn(async move {
        Thread::update_many()
            .col_expr(
                threads::Column::ViewCount,
                Expr::value(thread.view_count + 1),
            )
            .filter(threads::Column::Id.eq(thread_id))
            .exec(db)
            .await
    });

    // Load posts, their ugc associations, and their living revision.
    let posts: Vec<PostForTemplate> = Post::find()
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
        .filter(posts::Column::ThreadId.eq(thread_id))
        .filter(
            posts::Column::Position.between((page - 1) * POSTS_PER_PAGE + 1, page * POSTS_PER_PAGE),
        )
        .order_by_asc(posts::Column::Position)
        .order_by_asc(posts::Column::CreatedAt)
        .into_model::<PostForTemplate>()
        .all(db)
        .await
        .map_err(|e| {
            log::error!("get_thread_and_replies_for_page: Post::find(): {}", e);
            error::ErrorInternalServerError("DB error")
        })?;

    let attachments = get_attachments_for_ugc_by_id(posts.iter().map(|p| p.ugc_id).collect()).await;

    let paginator = Paginator {
        base_url: format!("/threads/{}/", thread_id),
        this_page: page,
        page_count: get_pages_in_thread(thread.post_count),
    };

    Ok(ThreadTemplate {
        client,
        thread,
        posts: &posts,
        paginator,
        attachments: &attachments,
    }
    .to_response())
}

/// Updates the post_count and last_post information on a thread.
/// This DOES NOT update post positions. It only updates the thread.
pub async fn update_thread_after_reply_is_deleted(id: i32) -> Result<(), DbErr> {
    #[derive(Debug, FromQueryResult)]
    struct LastPost {
        id: i32,
        created_at: chrono::NaiveDateTime,
    }

    let db = get_db_pool();

    let last_post_query = Post::find()
        .select_only()
        .column_as(posts::Column::Id, "id")
        .column_as(posts::Column::CreatedAt, "created_at")
        .left_join(ugc_deletions::Entity)
        .filter(posts::Column::ThreadId.eq(id))
        .filter(ugc_deletions::Column::DeletedAt.is_null())
        .into_model::<LastPost>()
        .one(db);

    let post_count_query = Post::find()
        .left_join(ugc_deletions::Entity)
        .filter(posts::Column::ThreadId.eq(id))
        .filter(ugc_deletions::Column::DeletedAt.is_null())
        .into_model::<LastPost>()
        .count(db);

    let (last_post_res, post_count_res) = futures::join!(last_post_query, post_count_query);

    if post_count_res.is_err() {
        let err = post_count_res.unwrap_err();
        log::error!("post_count error in update_thread: {:#?}", err);
        return Err(err);
    }

    if last_post_res.is_err() {
        let err = last_post_res.unwrap_err();
        log::error!("last_post error in update_thread: {:#?}", err);
        return Err(err);
    } else if let Some(last_post) = last_post_res.unwrap() {
        let post_count = post_count_res.unwrap();

        let update_res = Thread::update_many()
            .col_expr(threads::Column::PostCount, Expr::value(post_count as i32))
            .col_expr(threads::Column::LastPostId, Expr::value(last_post.id))
            .col_expr(
                threads::Column::LastPostAt,
                Expr::value(last_post.created_at),
            )
            .exec(db)
            .await;

        if update_res.is_err() {
            let err = update_res.unwrap_err();
            log::error!("update query error in update_thread: {:#?}", err);
            return Err(err);
        }
    } else {
        log::error!("thread has no last_post when trying to update thread.");
    }

    Ok(())
}

#[post("/threads/{thread_id}/post-reply")]
pub async fn create_reply(
    client: ClientCtx,
    path: web::Path<(i32,)>,
    mutipart: Option<Multipart>,
) -> Result<impl Responder, Error> {
    use crate::filesystem::{get_upload_response_from_field, UploadResponse};
    use crate::orm::{posts, threads, ugc_attachments};
    use crate::ugc::{create_ugc, NewUgcPartial};
    use futures::{future::try_join_all, StreamExt, TryStreamExt};

    let mut content: String = "".to_owned();
    let mut uploads: Vec<(String, UploadResponse)> = Vec::new();

    // interpret user input
    // iterate over multipart stream
    if let Some(mut fields) = mutipart {
        while let Ok(Some(mut field)) = fields.try_next().await {
            let disposition = field.content_disposition();
            if let Some(field_name) = disposition.get_name() {
                match field_name {
                    "content" => {
                        // Stream multipart data to string.
                        // TODO: Cap this at a config option for post size.
                        let mut buf: Vec<u8> = Vec::with_capacity(65536);

                        while let Some(chunk) = field.next().await {
                            let bytes = chunk.map_err(|e| {
                                log::error!("create_reply: multipart read error: {}", e);
                                actix_web::error::ErrorBadRequest("Error interpreting user input.")
                            })?;

                            buf.extend(bytes.to_owned());
                        }

                        content = str::from_utf8(&buf).unwrap().to_owned();
                    }
                    "attachment" => match disposition.get_filename() {
                        Some(filename) => uploads.push((
                            filename.to_owned(),
                            get_upload_response_from_field(&mut field).await?,
                        )),
                        None => {
                            return Err(error::ErrorBadRequest(
                                "Did not supply filename for attachment.",
                            ));
                        }
                    },
                    _ => {
                        return Err(error::ErrorBadRequest(format!(
                            "Unrecognized field '{}'",
                            field_name,
                        )));
                    }
                }
            }
        }
    }

    // Begin Transaction
    let db = get_db_pool();
    let txn = db.begin().await.map_err(error::ErrorInternalServerError)?;

    let thread_id = path.into_inner().0;
    let our_thread = Thread::find_by_id(thread_id)
        .one(&txn)
        .await
        .map_err(|_| error::ErrorInternalServerError("Could not look up thread."))?
        .ok_or_else(|| error::ErrorNotFound("Thread not found."))?;

    // Insert ugc and first revision
    let user_id = client.get_id();
    let ugc_revision = create_ugc(
        &txn,
        NewUgcPartial {
            ip_id: None,
            user_id,
            content: &content,
        },
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    // Insert post
    let new_post = posts::ActiveModel {
        thread_id: Set(our_thread.id),
        user_id: Set(ugc_revision.user_id),
        ugc_id: Set(ugc_revision.ugc_id),
        created_at: Set(ugc_revision.created_at),
        position: Set(our_thread.post_count + 1),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    // Insert attachments, if any.
    let sneed = try_join_all(uploads.iter().map(|u| {
        ugc_attachments::ActiveModel {
            attachment_id: Set(u.1.id),
            ugc_id: Set(ugc_revision.ugc_id),
            ip_id: Set(None), // TODO
            user_id: Set(ugc_revision.user_id),
            created_at: Set(ugc_revision.created_at),
            filename: Set(u.0.to_owned()),
            ..Default::default()
        }
        .insert(&txn)
    }))
    .await
    .map_err(error::ErrorInternalServerError)?;

    // Commit transaction
    txn.commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Update thread
    let post_id = new_post.id;
    threads::Entity::update_many()
        .col_expr(
            threads::Column::PostCount,
            Expr::value(our_thread.post_count + 1),
        )
        .col_expr(threads::Column::LastPostId, Expr::value(post_id))
        .col_expr(
            threads::Column::LastPostAt,
            Expr::value(new_post.created_at),
        )
        .filter(threads::Column::Id.eq(thread_id))
        .exec(db)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            get_url_for_pos(our_thread.id, our_thread.post_count + 1),
        ))
        .finish())
}

#[get("/threads/{thread_id}/")]
pub async fn view_thread(client: ClientCtx, path: web::Path<i32>) -> Result<impl Responder, Error> {
    get_thread_and_replies_for_page(client, path.into_inner(), 1).await
}

#[get("/threads/{thread_id}/page-{page}")]
pub async fn view_thread_page(
    client: ClientCtx,
    path: web::Path<(i32, i32)>,
) -> Result<impl Responder, Error> {
    let params = path.into_inner();
    if params.1 > 1 {
        get_thread_and_replies_for_page(client, params.0, params.1).await
    } else {
        get_thread_and_replies_for_page(client, params.0, 1).await
        //Ok(HttpResponse::Found()
        //    .append_header(("Location", format!("/threads/{}/", params.0)))
        //    .finish())
    }
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
