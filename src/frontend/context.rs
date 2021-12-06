use std::time::Instant;

/// Contextual information passed to the page container.
/// Initialized in Middleware. Passed in a Handler.
#[derive(Debug, Clone)]
pub struct Context {
    pub request_start: Instant,
}

impl Context {
    /// Returns human readable request time.
    pub fn request_time(&self) -> u128 {
        (Instant::now() - self.request_start).as_millis()
    }
}
