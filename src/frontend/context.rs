use std::time::{Duration, Instant};

/// Contextual information passed to the page container.
/// Initialized in Middleware. Passed in a Handler.
#[derive(Debug)]
pub struct Context {
    pub request_start: Instant,
}

impl Context {
    /// Returns Duration representing request time.
    pub fn request_time(&self) -> Duration {
        Instant::now() - self.request_start
    }

    /// Returns human readable representing request time.
    pub fn request_time_as_string(&self) -> String {
        let us = self.request_time().as_micros();
        if us > 5000 {
            format!("{}ms", us / 1000)
        } else {
            format!("{}μs", us)
        }
    }
}
