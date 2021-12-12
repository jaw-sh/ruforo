use actix_web::error::ErrorInternalServerError;
use actix_web::{dev, Error, FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{err, ok, Ready};
use sea_orm::FromQueryResult;

/// Represents information about this request's client.
#[derive(Debug, Default)]
pub struct Client {
    pub user: Option<ClientUser>,
}

impl Client {
    /// Returns either the user's id or None.
    pub fn get_id(&self) -> Option<i32> {
        self.user.as_ref().map(|u| u.id)
    }
    /// Returns either the user's name or the word for guest.
    /// TODO: l10n "Guest"
    pub fn get_name(&self) -> String {
        match &self.user {
            Some(user) => user.name.to_owned(),
            None => "Guest".to_owned(),
        }
    }

    pub fn is_user(&self) -> bool {
        self.user.is_some()
    }
}

// TODO: Move this implementation to a macro or something?
impl Client {
    pub fn can_post_in_thread(&self, thread: &crate::orm::threads::Model) -> bool {
        self.can_post_in_forum()
    }

    pub fn can_post_in_forum(&self) -> bool {
        true
    }

    pub fn can_delete_post(&self, post: &crate::post::PostForTemplate) -> bool {
        false
    }

    pub fn can_update_post(&self, post: &crate::post::PostForTemplate) -> bool {
        self.is_user() && self.get_id() == post.user_id
    }

    pub fn can_read_post(&self, post: &crate::post::PostForTemplate) -> bool {
        true
    }
}

impl FromRequest for Client {
    /// The associated error which can be returned.
    type Error = Error;

    /// Future that resolves to a Self.
    type Future = Ready<Result<Self, Self::Error>>;

    /// Create a Self from request parts asynchronously.
    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        match req.extensions().get::<Client>() {
            Some(client) => ok(Client {
                // TODO: This is probably slow.
                // We can't just use the request cycle's extension beacuse of lifetime constraints.
                user: client.user.to_owned(),
            }),
            None => err(ErrorInternalServerError(
                "Web server could not generate identity data.",
            )),
        }
    }
}

/// A mini struct for holding only what information we need about a client.
#[derive(Clone, Debug, FromQueryResult)]
pub struct ClientUser {
    pub id: i32,
    pub name: String,
}
