use crate::frontend;
use crate::frontend::TemplateToPubResponse;
use crate::orm::posts::Entity as Post;
use crate::orm::threads::Entity as Thread;
use crate::post::{NewPostFormData, PostForTemplate};
use crate::MainData;
use actix_web::{error, get, post, web, Error, HttpResponse};
use askama_actix::Template;
use sea_orm::{entity::*, query::*, sea_query::Expr, QueryFilter};
use serde::Deserialize;

// TODO: Dynamic page sizing.
const POSTS_PER_PAGE: i32 = 20;

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

// Returns which human-readable page number this position will appear in.
pub fn get_page_for_pos(pos: i32) -> i32 {
    return ((pos - 1) / POSTS_PER_PAGE) + 1;
}

// Returns the relative URL for the thread at this position.
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

// Returns a rendered view for a thread at a specified page.
async fn get_thread_and_replies_for_page(
    data: &MainData<'_>,
    thread_id: i32,
    page: i32,
    ctx: &frontend::Context,
) -> Result<HttpResponse, Error> {
    use crate::orm::{posts, threads};
    use futures::{future::TryFutureExt, try_join};

    let thread = Thread::find_by_id(thread_id)
        .one(&data.pool)
        .await
        .map_err(|_| error::ErrorInternalServerError("Could not look up thread."))?
        .ok_or_else(|| error::ErrorNotFound("Thread not found."))?;

    // Update thread to include views.
    let tfuture = Thread::update_many()
        .col_expr(
            threads::Column::ViewCount,
            Expr::value(thread.view_count + 1),
        )
        .filter(threads::Column::Id.eq(thread_id))
        .exec(&data.pool)
        //.await
        .map_err(|err| error::ErrorInternalServerError(err));

    // Load posts, their ugc associations, and their living revision.
    let pfuture = Post::find()
        .find_also_linked(posts::PostToUgcRevision)
        .filter(posts::Column::ThreadId.eq(thread_id))
        .filter(posts::Column::Position.between((page - 1) * POSTS_PER_PAGE, page * POSTS_PER_PAGE))
        .order_by_asc(posts::Column::Position)
        .order_by_asc(posts::Column::CreatedAt)
        .all(&data.pool)
        //.await
        .map_err(|_| error::ErrorInternalServerError("Could not find posts for this thread."));

    // Multi-thread drifting!
    let (presults, _) =
        try_join!(pfuture, tfuture).map_err(|err| error::ErrorInternalServerError(err))?;

    let mut posts = Vec::new();
    for (p, u) in &presults {
        posts.push(PostForTemplate::from_orm(&p, &u));
    }

    Ok(ThreadTemplate { thread, posts }.to_pub_response(ctx))
}

#[post("/threads/{thread_id}/post-reply")]
pub async fn create_reply(
    data: web::Data<MainData<'_>>,
    path: web::Path<(i32,)>,
    form: web::Form<NewPostFormData>,
) -> Result<HttpResponse, Error> {
    use crate::orm::{posts, threads};
    use crate::ugc::{create_ugc, NewUgcPartial};

    // Begin Transaction
    let txn = data
        .pool
        .begin()
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

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
            user_id: None,
            content: &form.content,
        },
    )
    .await
    .map_err(|err| error::ErrorInternalServerError(err))?;

    // Insert post
    let new_post = posts::ActiveModel {
        thread_id: Set(our_thread.id),
        ugc_id: ugc_revision.ugc_id,
        created_at: ugc_revision.created_at,
        position: Set(our_thread.post_count + 1),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|err| error::ErrorInternalServerError(err))?;

    // Commit transaction
    txn.commit()
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

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
        .map_err(|err| error::ErrorInternalServerError(err))?;

    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            get_url_for_pos(our_thread.id, our_thread.post_count + 1),
        ))
        .finish())
}

#[get("/threads/{thread_id}/")]
pub async fn view_thread(
    path: web::Path<(i32,)>,
    data: web::Data<MainData<'_>>,
    ctx: web::ReqData<frontend::Context>,
) -> Result<HttpResponse, Error> {
    get_thread_and_replies_for_page(&data, path.into_inner().0, 1, &ctx).await
}

#[get("/threads/{thread_id}/page-{page}")]
pub async fn view_thread_page(
    path: web::Path<(i32, i32)>,
    data: web::Data<MainData<'_>>,
    ctx: web::ReqData<frontend::Context>,
) -> Result<HttpResponse, Error> {
    let params = path.into_inner();

    if params.1 < 2 {
        Ok(HttpResponse::Found()
            .append_header(("Location", format!("/threads/{}/", params.0)))
            .finish())
    } else {
        get_thread_and_replies_for_page(&data, params.0, params.1, &ctx).await
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
