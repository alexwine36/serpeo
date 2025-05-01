use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum CrawlConfigError {
    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CrawlConfig {
    pub base_url: String,
    pub max_concurrent_requests: u32,
    pub request_delay_ms: u32,
}

impl CrawlConfig {
    pub fn new(base_url: String, max_concurrent_requests: u32, request_delay_ms: u32) -> Self {
        Self {
            base_url,
            max_concurrent_requests,
            request_delay_ms,
        }
    }
    pub fn get_url(&self) -> Result<Url, CrawlConfigError> {
        Url::parse(&self.base_url).map_err(|e| CrawlConfigError::UrlParseError(e.to_string()))
    }
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            base_url: "https://stem-programs.newspacenexus.org/".to_string(),
            max_concurrent_requests: 10,
            request_delay_ms: 100,
        }
    }
}
