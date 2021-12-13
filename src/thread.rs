use crate::frontend::TemplateToPubResponse;
use crate::orm::posts::Entity as Post;
use crate::orm::threads::Entity as Thread;
use crate::orm::{posts, threads, ugc_revisions, users};
use crate::post::{NewPostFormData, PostForTemplate};
use crate::template::{Paginator, PaginatorToHtml};
use crate::user::Client;
use crate::MainData;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use askama_actix::Template;
use sea_orm::{entity::*, query::*, sea_query::Expr, FromQueryResult, QueryFilter};
use serde::Deserialize;

// TODO: Dynamic page sizing.
const POSTS_PER_PAGE: i32 = 20;

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
    pub client: &'a Client,
    pub thread: crate::orm::threads::Model,
    pub posts: &'a Vec<PostForTemplate>,
    pub paginator: Paginator,
}

/// Returns which human-readable page number this position will appear in.
pub fn get_page_for_pos(pos: i32) -> i32 {
    ((pos - 1) / POSTS_PER_PAGE) + 1
}

pub fn get_pages_in_thread(post_count: i32) -> i32 {
    (post_count / POSTS_PER_PAGE) + 1
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
    client: &Client,
    data: &MainData<'_>,
    thread_id: i32,
    page: i32,
) -> Result<impl Responder, Error> {
    use crate::orm::user_names;

    let thread = Thread::find_by_id(thread_id)
        .one(&data.pool)
        .await
        .map_err(|_| error::ErrorInternalServerError("Could not look up thread."))?
        .ok_or_else(|| error::ErrorNotFound("Thread not found."))?;

    // Update thread to include views.
    actix_web::rt::spawn(async move {
        let pool = crate::session::new_db_pool().await.unwrap();
        Thread::update_many()
            .col_expr(
                threads::Column::ViewCount,
                Expr::value(thread.view_count + 1),
            )
            .filter(threads::Column::Id.eq(thread_id))
            .exec(&pool)
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
        //.find_also_related(users::Entity)
        //.find_also_linked(posts::PostToUgcRevision)
        .filter(posts::Column::ThreadId.eq(thread_id))
        .filter(
            posts::Column::Position.between((page - 1) * POSTS_PER_PAGE + 1, page * POSTS_PER_PAGE),
        )
        .order_by_asc(posts::Column::Position)
        .order_by_asc(posts::Column::CreatedAt)
        .into_model::<PostForTemplate>()
        .all(&data.pool)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let paginator = Paginator {
        base_url: format!("/threads/{}/", thread_id),
        this_page: page,
        page_count: get_pages_in_thread(thread.post_count),
    };

    ThreadTemplate {
        client: client,
        thread,
        posts: &posts,
        paginator,
    }
    .to_pub_response()
}

#[post("/threads/{thread_id}/post-reply")]
pub async fn create_reply(
    client: Client,
    data: web::Data<MainData<'_>>,
    path: web::Path<(i32,)>,
    form: web::Form<NewPostFormData>,
) -> Result<impl Responder, Error> {
    use crate::orm::{posts, threads};
    use crate::ugc::{create_ugc, NewUgcPartial};

    // Begin Transaction
    let txn = data
        .pool
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let thread_id = path.into_inner().0;
    let our_thread = Thread::find_by_id(thread_id)
        .one(&txn)
        .await
        .map_err(|_| error::ErrorInternalServerError("Could not look up thread."))?
        .ok_or_else(|| error::ErrorNotFound("Thread not found."))?;

    // Insert ugc and first revision
    let ugc_revision = create_ugc(
        &txn,
        NewUgcPartial {
            ip_id: None,
            user_id: client.get_id(),
            content: &form.content,
        },
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    // Insert post
    let new_post = posts::ActiveModel {
        thread_id: Set(our_thread.id),
        user_id: ugc_revision.user_id,
        ugc_id: ugc_revision.ugc_id,
        created_at: ugc_revision.created_at,
        position: Set(our_thread.post_count + 1),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    // Commit transaction
    txn.commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Update thread
    let post_id = new_post.id.clone().unwrap(); // TODO: Change once SeaQL 0.5.0 is out
    threads::Entity::update_many()
        .col_expr(
            threads::Column::PostCount,
            Expr::value(our_thread.post_count + 1),
        )
        .col_expr(threads::Column::LastPostId, Expr::value(post_id))
        .col_expr(
            threads::Column::LastPostAt,
            Expr::value(new_post.created_at.clone().unwrap()), // TODO: Change once SeaQL 0.5.0 is out
        )
        .filter(threads::Column::Id.eq(thread_id))
        .exec(&data.pool)
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
pub async fn view_thread(
    client: Client,
    path: web::Path<(i32,)>,
    data: web::Data<MainData<'_>>,
) -> Result<impl Responder, Error> {
    get_thread_and_replies_for_page(&client, &data, path.into_inner().0, 1).await
}

#[get("/threads/{thread_id}/page-{page}")]
pub async fn view_thread_page(
    client: Client,
    path: web::Path<(i32, i32)>,
    data: web::Data<MainData<'_>>,
) -> Result<impl Responder, Error> {
    let params = path.into_inner();

    if params.1 > 1 {
        get_thread_and_replies_for_page(&client, &data, params.0, params.1).await
    } else {
        get_thread_and_replies_for_page(&client, &data, params.0, 1).await
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
