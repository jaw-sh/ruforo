use crate::frontend::TemplateToPubResponse;
use crate::orm::{posts, threads, users};
use crate::thread::{validate_thread_form, NewThreadFormData, ThreadForTemplate};
use crate::user::Client;
use crate::MainData;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use askama_actix::Template;
use sea_orm::{entity::*, query::*, sea_query::Expr};

#[derive(Template)]
#[template(path = "forum.html")]
pub struct ForumTemplate<'a> {
    pub threads: &'a Vec<ThreadForTemplate>,
}

#[post("/forums/post-thread")]
pub async fn create_thread(
    client: Client,
    data: web::Data<MainData<'_>>,
    form: web::Form<NewThreadFormData>,
) -> Result<impl Responder, Error> {
    use crate::ugc::{create_ugc, NewUgcPartial};

    // Run form data through validator.
    let form = validate_thread_form(form).map_err(|err| err)?;

    // Begin Transaction
    let txn = data
        .pool
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
        created_at: revision.created_at.to_owned(),
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
        ugc_id: revision.ugc_id,
        created_at: revision.created_at.clone(),
        position: Set(1),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(error::ErrorInternalServerError)?;

    // Step 4. Update the thread to include last, first post id info.
    let post_id = new_post.id.clone().unwrap(); // TODO: Change once SeaQL 0.5.0 is out
    threads::Entity::update_many()
        .col_expr(threads::Column::PostCount, Expr::value(1))
        .col_expr(threads::Column::FirstPostId, Expr::value(post_id))
        .col_expr(threads::Column::LastPostId, Expr::value(post_id))
        .col_expr(
            threads::Column::LastPostAt,
            Expr::value(revision.created_at.clone().unwrap()), // TODO: Change once SeaQL 0.5.0 is out
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

#[get("/forums")]
pub async fn view_forum(data: web::Data<MainData<'_>>) -> Result<impl Responder, Error> {
    let threads: Vec<ThreadForTemplate> = threads::Entity::find()
        // Authoring User
        .left_join(users::Entity)
        .column_as(users::Column::Name, "username")
        // Last Post
        // TODO: This is an actual nightmare.
        //.join_join(JoinType::LeftJoin, threads::Relations::::to(), threads::Relation::LastPost<posts::Entity>::via())
        //.column_as(users::Column::Name, "username")
        // Execute
        .order_by_desc(threads::Column::LastPostAt)
        .into_model::<ThreadForTemplate>()
        .all(&data.pool)
        .await
        .map_err(error::ErrorNotFound)?;

    Ok(ForumTemplate { threads: &threads }.to_pub_response())
}
