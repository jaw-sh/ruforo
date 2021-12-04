use crate::orm::{posts, threads};
use crate::thread::{validate_thread_form, NewThreadFormData};
use crate::MainData;
use actix_web::{error, get, post, web, Error, HttpResponse};
use askama_actix::Template;
use sea_orm::sea_query::Expr;
use sea_orm::{entity::*, query::*, Set};

#[derive(Template)]
#[template(path = "forum.html")]
pub struct ForumTemplate {
    pub threads: Vec<threads::Model>,
}

#[post("/forums/post-thread")]
pub async fn create_thread(
    data: web::Data<MainData<'static>>,
    form: web::Form<NewThreadFormData>,
) -> Result<HttpResponse, Error> {
    use crate::ugc::{create_ugc, NewUgcPartial};

    // Run form data through validator.
    let form = validate_thread_form(form).map_err(|err| err)?;

    // Begin Transaction
    let txn = data
        .pool
        .begin()
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

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
    .map_err(|err| error::ErrorInternalServerError(err))?;

    // Step 2. Create a thread.
    let thread = threads::ActiveModel {
        //user_id
        created_at: revision.created_at.to_owned(),
        title: Set(form.title.trim().to_owned()),
        subtitle: Set(form
            .subtitle
            .to_owned()
            .map(|s| s.trim().to_owned())
            .filter(|s| s.len() != 0)),
        post_count: Set(1),
        ..Default::default()
    };
    let thread_res = threads::Entity::insert(thread)
        .exec(&txn)
        .await
        .map_err(|_| error::ErrorInternalServerError("Failed to insert new thread."))?;

    // Step 3. Create a post with the correct associations.
    let new_post = posts::ActiveModel {
        thread_id: Set(thread_res.last_insert_id),
        ugc_id: revision.ugc_id,
        created_at: revision.created_at.clone(),
        position: Set(1),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|err| error::ErrorInternalServerError(err))?;

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
        .map_err(|_| error::ErrorInternalServerError("Failed to update UGC to living revision."))?;

    // Close transaction
    txn.commit()
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            format!("/threads/{}/", thread_res.last_insert_id),
        ))
        .finish())
}

#[get("/forums/")]
pub async fn view_forum(data: web::Data<MainData<'static>>) -> Result<HttpResponse, Error> {
    match threads::Entity::find().all(&data.pool).await {
        Ok(threads) => {
            return Ok(HttpResponse::Ok().body(ForumTemplate { threads }.render().unwrap()))
        }
        Err(err) => return Err(error::ErrorNotFound(err)),
    }
}
