use crate::orm::users::Model as User;

/// Represents information about this request's client.
#[derive(Debug, Default)]
pub struct Client {
    pub user: Option<User>,
}

impl Client {
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
