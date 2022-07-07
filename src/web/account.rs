use crate::db::get_db_pool;
use crate::middleware::ClientCtx;
use crate::user::Profile as UserProfile;
use actix_multipart::Multipart;
use actix_web::{error, get, post, Error, HttpResponse, Responder};
use askama_actix::{Template, TemplateToResponse};
use chrono::Utc;
use sea_orm::entity::*;

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(update_avatar).service(view_account);
}

#[derive(Template)]
#[template(path = "account.html")]
pub struct AccountTemplate {
    pub client: ClientCtx,
    pub profile: UserProfile,
}

#[post("/account/avatar")]
async fn update_avatar(client: ClientCtx, mutipart: Option<Multipart>) -> impl Responder {
    use crate::filesystem::{
        deduplicate_payload, insert_payload_as_attachment, save_field_as_temp_file,
    };
    use crate::orm::user_avatars;
    use futures::TryStreamExt;

    if !client.is_user() {
        return Err(error::ErrorUnauthorized(
            "You must be logged in to do that.",
        ));
    }

    // TODO: Button to delete avatars.

    if let Some(mut fields) = mutipart {
        while let Ok(Some(mut field)) = fields.try_next().await {
            let disposition = field.content_disposition();
            if let Some(field_name) = disposition.get_name() {
                match field_name {
                    "avatar" => {
                        // Save the file to a temporary location and get payload data.
                        let payload = match save_field_as_temp_file(&mut field).await? {
                            Some(payload) => payload,
                            None => {
                                return Err(error::ErrorBadRequest("Upload is empty or improper."))
                            }
                        };

                        // Pass file through deduplication and receive a response..
                        let response = match deduplicate_payload(&payload).await {
                            Some(response) => response,
                            None => match insert_payload_as_attachment(payload, None).await? {
                                Some(response) => response,
                                None => {
                                    return Err(error::ErrorBadRequest(
                                        "Upload is empty or improper.",
                                    ))
                                }
                            },
                        };

                        match user_avatars::Entity::insert(user_avatars::ActiveModel {
                            user_id: Set(client.get_id().unwrap()),
                            attachment_id: Set(response.id),
                            created_at: Set(Utc::now().naive_utc()),
                        })
                        .exec(get_db_pool())
                        .await
                        {
                            Ok(_) => {}
                            Err(err) => log::warn!("SQL error when inserting avatar: {:?}", err),
                        };
                    }
                    _ => {
                        return Err(error::ErrorBadRequest(format!(
                            "Unknown field '{}'",
                            field_name
                        )))
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Found()
        .append_header(("Location", "/account"))
        .finish())
}

#[get("/account")]
async fn view_account(client: ClientCtx) -> Result<impl Responder, Error> {
    if !client.is_user() {
        return Err(error::ErrorUnauthorized(
            "You must be logged in to do that.",
        ));
    }

    let db = get_db_pool();
    let profile = UserProfile::get_by_id(db, client.get_id().unwrap())
        .await
        .map_err(error::ErrorInternalServerError)?
        .ok_or_else(|| error::ErrorInternalServerError("Unable to find account."))?;

    Ok(AccountTemplate { client, profile }.to_response())
}
