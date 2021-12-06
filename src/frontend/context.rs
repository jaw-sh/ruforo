use chrono::prelude::{NaiveDateTime, Utc};

/// Contextual information passed to the page container.
/// Initialized in Middleware. Passed in a Handler.
#[derive(Debug, Clone)]
pub struct Context {
    pub request_start: NaiveDateTime,
}

impl Context {
    /// Returns human readable request time.
    pub fn request_time(&self) -> i64 {
        (Utc::now().naive_utc() - self.request_start).num_milliseconds()
    }
}
