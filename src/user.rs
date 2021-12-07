/// Represents information about this request's client.
pub struct Client {
    pub user: Option<bool>, // TODO: Replace with a model.
}

pub trait ClientUserInterface {
    fn get_name(&self) -> String;
}

impl ClientUserInterface for Client {
    fn get_name(&self) -> String {
        "Guest".to_owned()
    }
}
