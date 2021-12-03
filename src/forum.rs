use crate::orm::{posts, threads};
use actix_web::{error, get, post, web, Error, HttpResponse};
use askama_actix::Template;
use ruforo::MyAppData;
use sea_orm::{entity::*, Set};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewThreadFormData {
    title: String,
    subtitle: Option<String>,
    content: String,
}

#[derive(Template)]
#[template(path = "forum.html")]
pub struct ForumTemplate {
    pub threads: Vec<threads::Model>,
}

#[post("/forums/post-thread")]
pub async fn create_thread(
    data: web::Data<MyAppData<'static>>,
    form: web::Form<NewThreadFormData>,
) -> Result<HttpResponse, Error> {
    use crate::ugc::{create_ugc, NewUgcPartial};

    // TODO: This belongs in a transaction!
    // Step 1. Create the UGC.
    let revision = create_ugc(
        &data.pool,
        NewUgcPartial {
            ip_id: None,
            user_id: None,
            content: form.content.to_owned(),
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
        ..Default::default()
    };
    let thread_res = threads::Entity::insert(thread)
        .exec(&data.pool)
        .await
        .map_err(|_| error::ErrorInternalServerError("Failed to insert new thread."))?;

    // Step 3. Create a post with the correct associations.
    posts::ActiveModel {
        thread_id: Set(thread_res.last_insert_id),
        ugc_id: revision.id,
        created_at: revision.created_at.to_owned(),
        ..Default::default()
    }
    .insert(&data.pool)
    .await
    .map_err(|_| error::ErrorInternalServerError("Failed to insert new post."))?;

    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            format!("/threads/{}/", thread_res.last_insert_id),
        ))
        .finish())
}

#[get("/forums/")]
pub async fn read_forum(data: web::Data<MyAppData<'static>>) -> Result<HttpResponse, Error> {
    match threads::Entity::find().all(&data.pool).await {
        Ok(threads) => {
            return Ok(HttpResponse::Ok().body(ForumTemplate { threads }.render().unwrap()))
        }
        Err(_) => return Err(error::ErrorNotFound("Thread not found.")),
    }
}
