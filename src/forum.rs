use crate::init::get_db_pool;
use crate::middleware::ClientCtx;
use crate::orm::{posts, threads, user_names};
use crate::thread::{validate_thread_form, NewThreadFormData, ThreadForTemplate};
use actix_web::{error, get, post, web, Error, HttpRequest, HttpResponse, Responder};
use askama_actix::{Template, TemplateToResponse};
use sea_orm::DbErr;
use sea_orm::{entity::*, query::*, sea_query::Expr};

#[derive(Template)]
#[template(path = "forum.html")]
pub struct ForumTemplate<'a> {
    pub client: ClientCtx,
    pub forum: &'a crate::orm::forums::Model,
    pub threads: &'a Vec<ThreadForTemplate>,
}

#[derive(Template)]
#[template(path = "forums.html")]
pub struct ForumIndexTemplate<'a> {
    pub client: ClientCtx,
    pub forums: &'a Vec<crate::orm::forums::Model>,
}

#[post("/forums/{forum}/post-thread")]
pub async fn create_thread(
    client: ClientCtx,
    form: web::Form<NewThreadFormData>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    use crate::ugc::{create_ugc, NewUgcPartial};
    let forum_id = path.into_inner();

    // Run form data through validator.
    let form = validate_thread_form(form).map_err(|err| err)?;

    // Begin Transaction
    let txn = get_db_pool()
        .begin()
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Step 1. Create the UGC.
    let revision = create_ugc(
        &txn,
        NewUgcPartial {
            ip_id: None,
            user_id: None,
            content: &form.content,
        },
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    // Step 2. Create a thread.
    let thread = threads::ActiveModel {
        user_id: Set(client.get_id()),
        forum_id: Set(forum_id),
        created_at: Set(revision.created_at),
        title: Set(form.title.trim().to_owned()),
        subtitle: Set(form
            .subtitle
            .to_owned()
            .map(|s| s.trim().to_owned())
            .filter(|s| s.is_empty())),
        view_count: Set(0),
        post_count: Set(1),
        ..Default::default()
    };
    let thread_res = threads::Entity::insert(thread)
        .exec(&txn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Step 3. Create a post with the correct associations.
    let new_post = posts::ActiveModel {
        user_id: Set(client.get_id()),
        thread_id: Set(thread_res.last_insert_id),
        ugc_id: Set(revision.ugc_id),
        created_at: Set(revision.created_at),
        position: Set(1),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    // Step 4. Update the thread to include last, first post id info.
    threads::Entity::update_many()
        .col_expr(threads::Column::PostCount, Expr::value(1))
        .col_expr(threads::Column::FirstPostId, Expr::value(new_post.id))
        .col_expr(threads::Column::LastPostId, Expr::value(new_post.id))
        .col_expr(
            threads::Column::LastPostAt,
            Expr::value(revision.created_at),
        )
        .filter(threads::Column::Id.eq(thread_res.last_insert_id))
        .exec(&txn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Close transaction
    txn.commit()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            format!("/threads/{}/", thread_res.last_insert_id),
        ))
        .finish())
}

#[get("/forums/{forum}/")]
pub async fn view_forum(client: ClientCtx, path: web::Path<i32>) -> Result<impl Responder, Error> {
    use crate::orm::forums;

    let forum_id = path.into_inner();
    let forum = forums::Entity::find_by_id(forum_id)
        .one(get_db_pool())
        .await
        .map_err(|_| error::ErrorInternalServerError("Could not look up forum."))?
        .ok_or_else(|| error::ErrorNotFound("Forum not found."))?;

    let threads: Vec<ThreadForTemplate> = match threads::Entity::find()
        // Authoring User
        .left_join(user_names::Entity)
        .column_as(user_names::Column::Name, "username")
        // Last Post
        // TODO: This is an actual nightmare.
        //.join_join(JoinType::LeftJoin, threads::Relations::::to(), threads::Relation::LastPost<posts::Entity>::via())
        //.column_as(users::Column::Name, "username")
        // Execute
        .filter(threads::Column::ForumId.eq(forum_id))
        .order_by_desc(threads::Column::LastPostAt)
        .into_model::<ThreadForTemplate>()
        .all(get_db_pool())
        .await
    {
        Ok(threads) => threads,
        Err(_) => Default::default(),
    };

    Ok(ForumTemplate {
        client: client.to_owned(),
        forum: &forum,
        threads: &threads,
    }
    .to_response())
}

#[get("/forums")]
pub async fn view_forums(client: ClientCtx) -> Result<impl Responder, Error> {
    render_forum_list(client).await
}

pub async fn render_forum_list(client: ClientCtx) -> Result<impl Responder, Error> {
    use crate::orm::forums;

    let forums = match forums::Entity::find().all(get_db_pool()).await {
        Ok(forums) => forums,
        Err(_) => Default::default(),
    };

    Ok(ForumIndexTemplate {
        client: client.to_owned(),
        forums: &forums,
    }
    .to_response())
}
