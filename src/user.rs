use crate::orm::users::Model as User;
use actix_web::error::ErrorInternalServerError;
use actix_web::{dev, web, App, Error, FromRequest, HttpRequest};
use futures_util::future::{err, ok, Ready};

/// Represents information about this request's client.
#[derive(Debug, Default)]
pub struct Client {
    pub user: Option<User>,
}

impl Client {
    /// Returns either the user's id or None.
    pub fn get_id(&self) -> Option<i32> {
        self.user.as_ref().map_or(None, |u| Some(u.id))
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

impl FromRequest for Client {
    /// The associated error which can be returned.
    type Error = Error;

    /// Future that resolves to a Self.
    type Future = Ready<Result<Self, Self::Error>>;

    /// Create a Self from request parts asynchronously.
    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        match req.extensions_mut().get::<Client>() {
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
