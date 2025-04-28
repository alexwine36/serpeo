use std::{collections::HashMap, time::Duration};

use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum PageError {
    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),
    #[error("Failed to fetch URL: {0}")]
    FetchError(String),
    #[error("Document not set: {0}")]
    DocumentNotSet(String),
    #[error("Config not set")]
    ConfigNotSet,
}

pub struct Page {
    url: Option<Url>,
    html: Option<String>,
}

impl Page {
    pub fn from_html(html: String) -> Self {
        Self {
            url: None,
            html: Some(html),
        }
    }

    pub fn set_url<T: FromUrl>(&mut self, url: T) {
        self.url = Some(url.to_url().unwrap());
    }

    async fn fetch(&mut self, url: &Url) -> Result<(), PageError> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| PageError::FetchError(e.to_string()))?;

        let response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| PageError::FetchError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PageError::FetchError(format!(
                "Failed to fetch URL: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| PageError::FetchError(e.to_string()))?;

        self.set_content(body);
        Ok(())
    }
    pub fn set_content(&mut self, html: String) {
        self.html = Some(html);
    }

    pub async fn from_url<T: FromUrl>(url: T) -> Result<Self, PageError> {
        let url = url.to_url().unwrap();
        // let html = self.fetch(&url).await;

        let client = Client::
        builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
             .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| PageError::FetchError(e.to_string()))?;

        let response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| PageError::FetchError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PageError::FetchError(format!(
                "Failed to fetch URL: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| PageError::FetchError(e.to_string()))?;

        Ok(Self {
            url: Some(url),
            html: Some(body),
        })
    }

    pub fn get_document(&self) -> Result<Html, PageError> {
        let html = self
            .html
            .as_ref()
            .ok_or(PageError::DocumentNotSet("Document is not set".to_string()))?;
        Ok(Html::parse_document(html))
    }

    pub fn extract_images(&self, document: &Html) -> Vec<Image> {
        let img_selector = Selector::parse("img").unwrap();
        let mut images = Vec::new();

        for img in document.select(&img_selector) {
            let src = img.value().attr("src").unwrap_or_default().to_string();
            let alt = img.value().attr("alt").map(|s| s.to_string());
            let srcset = img.value().attr("srcset").map(|s| s.to_string());
            images.push(Image { src, alt, srcset });
        }

        images
    }

    pub fn extract_meta_tags(&self, document: &Html) -> MetaTagInfo {
        let mut meta_tags = MetaTagInfo::default();

        let title_selector = Selector::parse("title").unwrap();
        if let Some(title) = document.select(&title_selector).next() {
            meta_tags.title = Some(title.inner_html());
        }
        let link_selector = Selector::parse("link").unwrap();
        for link in document.select(&link_selector) {
            if let Some(rel) = link.value().attr("rel") {
                if rel == "canonical" {
                    meta_tags.canonical = link.value().attr("href").map(|s| s.to_string());
                }
                if rel == "sitemap" {
                    meta_tags.sitemap = link.value().attr("href").map(|s| s.to_string());
                }
                if rel == "shortcut icon" || rel == "icon" {
                    meta_tags.favicon = link.value().attr("href").map(|s| s.to_string());
                }
                if rel == "manifest" {
                    meta_tags.webmanifest = link.value().attr("href").map(|s| s.to_string());
                }
                if rel == "script" {
                    if let Some(src) = link.value().attr("src") {
                        meta_tags.scripts.push(src.to_string());
                    }
                }
                if rel == "stylesheet" {
                    if let Some(href) = link.value().attr("href") {
                        meta_tags.styles.push(href.to_string());
                    }
                }
            }
        }

        let meta_selector = Selector::parse("meta").unwrap();
        for meta in document.select(&meta_selector) {
            if let Some(name) = meta.value().attr("name") {
                match name {
                    "description" => {
                        meta_tags.description = meta.value().attr("content").map(|s| s.to_string());
                    }
                    "robots" => {
                        meta_tags.robots = meta.value().attr("content").map(|s| s.to_string());
                    }
                    "keywords" => {
                        meta_tags.keywords = meta.value().attr("content").map(|s| s.to_string());
                    }
                    "viewport" => {
                        meta_tags.viewport = meta.value().attr("content").map(|s| s.to_string());
                    }
                    "generator" => {
                        if let Some(content) = meta.value().attr("content") {
                            meta_tags.generators.push(content.to_string());
                        }
                    }

                    _ => {}
                }
            }
            if let Some(property) = meta.value().attr("property") {
                if property.starts_with("og:") {
                    let key = property.trim_start_matches("og:");
                    if let Some(value) = meta.value().attr("content") {
                        meta_tags.og_tags.insert(key.to_string(), value.to_string());
                    }
                } else if property.starts_with("twitter:") {
                    let key = property.trim_start_matches("twitter:");
                    if let Some(value) = meta.value().attr("content") {
                        meta_tags
                            .twitter_tags
                            .insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        meta_tags
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub struct MetaTagInfo {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub robots: Option<String>,
    pub canonical: Option<String>,
    pub sitemap: Option<String>,
    pub favicon: Option<String>,
    pub viewport: Option<String>,
    pub generators: Vec<String>,
    pub webmanifest: Option<String>,
    pub og_tags: HashMap<String, String>,
    pub scripts: Vec<String>,
    pub styles: Vec<String>,
    pub twitter_tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
    pub srcset: Option<String>,
}

pub trait FromUrl {
    fn to_url(self) -> Result<Url, PageError>;
}

impl FromUrl for Url {
    fn to_url(self) -> Result<Url, PageError> {
        Ok(self)
    }
}

impl FromUrl for String {
    fn to_url(self) -> Result<Url, PageError> {
        Url::parse(&self).map_err(|e| PageError::UrlParseError(e.to_string()))
    }
}

impl FromUrl for &String {
    fn to_url(self) -> Result<Url, PageError> {
        Url::parse(self).map_err(|e| PageError::UrlParseError(e.to_string()))
    }
}

impl FromUrl for &str {
    fn to_url(self) -> Result<Url, PageError> {
        Url::parse(self).map_err(|e| PageError::UrlParseError(e.to_string()))
    }
}
