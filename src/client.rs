use reqwest::Client;
use std::time::Duration;

/// Shared application state — one HTTP client for the lifetime of the process.
/// `Client` internally manages a connection pool, so creating it once and
/// cloning the handle into each request is the correct pattern.
#[derive(Clone)]
pub struct AppState {
    pub client: Client,
}

impl AppState {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }
}
