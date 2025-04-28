use reqwest::{Client, Response};
use std::any::Any;
use thiserror::Error;
use tokio::time::Duration;
use url::Url;

use crate::utils::{
    config::{CheckResult, Rule, RuleConfig, SeoPlugin, Severity},
    registry::PluginRegistry,
};

#[derive(Debug, Error)]
pub enum NetworkRequestError {
    #[error("Failed to fetch URL: {0}")]
    FetchError(String),
    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),
}

#[derive(Debug, Clone)]
pub struct NetworkRequestPlugin {
    // url: Url,
    client: Client,
    // response: Option<Response>,
}

impl NetworkRequestPlugin {
    pub fn new() -> Result<Self, NetworkRequestError> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| NetworkRequestError::FetchError(e.to_string()))?;

        Ok(Self { client })
    }
    pub fn get_client(&self) -> &Client {
        &self.client
    }

    pub async fn fetch(&self, url: &str) -> Result<Response, NetworkRequestError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| NetworkRequestError::FetchError(e.to_string()))?;
        Ok(response)
    }

    pub async fn fetch_text(&self, url: &str) -> Result<String, NetworkRequestError> {
        let response = self.fetch(url).await?;
        let text = response
            .text()
            .await
            .map_err(|e| NetworkRequestError::FetchError(e.to_string()))?;
        Ok(text)
    }
}

impl SeoPlugin for NetworkRequestPlugin {
    fn name(&self) -> &str {
        "Network Request"
    }
    fn description(&self) -> &str {
        "Network Request"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn initialize(&mut self, _registry: &PluginRegistry) -> Result<(), String> {
        Ok(())
    }

    fn available_rules(&self) -> Vec<Rule> {
        vec![]
    }

    fn analyze(&self, _config: &RuleConfig) -> Vec<CheckResult> {
        vec![]
    }
}

pub trait FromUrl {
    fn to_url(self) -> Result<Url, NetworkRequestError>;
}

impl FromUrl for Url {
    fn to_url(self) -> Result<Url, NetworkRequestError> {
        Ok(self)
    }
}

impl FromUrl for String {
    fn to_url(self) -> Result<Url, NetworkRequestError> {
        Url::parse(&self).map_err(|e| NetworkRequestError::UrlParseError(e.to_string()))
    }
}

impl FromUrl for &String {
    fn to_url(self) -> Result<Url, NetworkRequestError> {
        Url::parse(self).map_err(|e| NetworkRequestError::UrlParseError(e.to_string()))
    }
}

impl FromUrl for &str {
    fn to_url(self) -> Result<Url, NetworkRequestError> {
        Url::parse(self).map_err(|e| NetworkRequestError::UrlParseError(e.to_string()))
    }
}
