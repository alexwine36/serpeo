use serde::{Deserialize, Serialize};
use specta::Type;
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Config {
    pub base_url: String,
    pub max_concurrent_requests: u32,
    pub request_delay_ms: u32,
}

impl Config {
    pub fn new(base_url: String, max_concurrent_requests: u32, request_delay_ms: u32) -> Self {
        Self {
            base_url,
            max_concurrent_requests,
            request_delay_ms,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: "https://example.com".to_string(),
            max_concurrent_requests: 10,
            request_delay_ms: 100,
        }
    }
}
