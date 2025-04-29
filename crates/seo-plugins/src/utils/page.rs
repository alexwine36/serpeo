use std::{collections::HashMap, time::Duration};

use markup5ever::QualName;
use reqwest::Client;
use scraper::{
    ElementRef, Html, Selector,
    node::{Attributes, Element},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use tokio::time::Instant;
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
    #[error("Element not found")]
    ElementNotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Page {
    url: Option<Url>,
    html: Option<String>,
    meta_tags: Option<MetaTagInfo>,
    images: Option<Vec<Image>>,
    content_length: Option<u64>,
    redirected: Option<bool>,
    elapsed: Option<f32>,
}

const FALLBACK_URL: &str = "https://example.com";

impl Page {
    pub fn from_html(html: String) -> Self {
        Self {
            url: None,
            html: Some(html),
            meta_tags: None,
            images: None,
            content_length: None,
            redirected: None,
            elapsed: None,
        }
    }

    pub fn set_url<T: FromUrl>(&mut self, url: T) {
        self.url = Some(url.to_url().unwrap());
    }

    pub fn get_url(&self) -> Option<Url> {
        self.url.clone()
    }

    pub fn get_html(&self) -> Option<String> {
        self.html.clone()
    }

    pub fn get_content_length(&self) -> Option<u64> {
        self.content_length.clone()
    }

    pub fn get_redirected(&self) -> Option<bool> {
        self.redirected.clone()
    }

    pub fn set_content(&mut self, html: String) {
        self.html = Some(html);
    }

    pub fn get_elapsed(&self) -> Option<f32> {
        self.elapsed.clone()
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
        let start_time = Instant::now();
        let response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| PageError::FetchError(e.to_string()))?;

        let elapsed = start_time.elapsed().as_millis() as f32;
        let redirected = response.status().is_redirection();
        let content_length = response
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok().and_then(|s| s.parse::<u64>().ok()));

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
            meta_tags: None,
            images: None,
            content_length,
            redirected: Some(redirected),
            elapsed: Some(elapsed),
        })
    }

    pub fn get_document(&self) -> Result<Html, PageError> {
        let html = self
            .html
            .as_ref()
            .ok_or(PageError::DocumentNotSet("Document is not set".to_string()))?;
        Ok(Html::parse_document(html))
    }

    pub fn get_element(&self, selector: &str) -> Result<Element, PageError> {
        let document = self.get_document().unwrap();
        let selector = Selector::parse(selector).unwrap();
        let element = document
            .select(&selector)
            .next()
            .ok_or(PageError::ElementNotFound)?;
        Ok(element.value().clone())
    }

    fn format_element(&self, element: &ElementRef) -> StaticElement {
        let element = *element;
        StaticElement {
            name: element.value().name.clone(),
            attrs: element.value().attrs.clone(),
            text: element.text().collect::<String>(),
        }
    }

    pub fn get_elements(&self, selector: &str) -> Vec<StaticElement> {
        let document = self.get_document().unwrap();
        let selector = Selector::parse(selector).unwrap();
        let mut elements: Vec<StaticElement> = Vec::new();
        for element in document.select(&selector) {
            // println!("element: {:?}", element.text());
            elements.push(self.format_element(&element));
        }
        elements
    }
}

impl Page {
    // Images
    fn set_images(&mut self) {
        let document = self.get_document().unwrap();
        let img_selector = Selector::parse("img").unwrap();
        let mut images = Vec::new();

        for img in document.select(&img_selector) {
            let src = img.value().attr("src").unwrap_or_default().to_string();
            let alt = img.value().attr("alt").map(|s| s.to_string());
            let srcset = img.value().attr("srcset").map(|s| s.to_string());
            images.push(Image { src, alt, srcset });
        }

        self.images = Some(images);
    }

    fn get_images(&mut self) -> Option<Vec<Image>> {
        if self.images.is_none() {
            self.set_images();
        }
        self.images.clone()
    }

    pub fn extract_images(&mut self) -> Vec<Image> {
        self.get_images().unwrap_or_default()
    }

    // Meta Tags
    fn set_meta_tags(&mut self) {
        let document = self.get_document().unwrap();
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
            if let Some(charset) = meta.value().attr("charset") {
                meta_tags.charset = Some(charset.to_string());
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

        self.meta_tags = Some(meta_tags);
    }

    fn get_meta_tags(&mut self) -> MetaTagInfo {
        if self.meta_tags.is_none() {
            self.set_meta_tags();
        }
        self.meta_tags.clone().unwrap_or_default()
    }

    pub fn extract_meta_tags(&mut self) -> MetaTagInfo {
        self.get_meta_tags()
    }

    // Links
    pub fn extract_links(&self) -> Vec<Links> {
        let document = self.get_document().unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let mut links = Vec::new();

        for link in document.select(&link_selector) {
            if let Some(href) = link.value().attr("href") {
                let base_url = self
                    .url
                    .clone()
                    .unwrap_or(Url::parse(FALLBACK_URL).unwrap());
                let url = base_url.join(href).unwrap_or_else(|_| base_url.clone());
                let link_type = if url.host_str() == Some(base_url.host_str().unwrap()) {
                    LinkType::Internal
                } else {
                    LinkType::External
                };
                let path = url.path().to_string();
                links.push(Links {
                    href: url.to_string(),
                    path,
                    link_type,
                });
            }
        }

        links
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
    pub charset: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
    pub srcset: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
pub enum LinkType {
    Internal,
    External,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct Links {
    pub href: String,
    pub path: String,
    pub link_type: LinkType,
}

pub struct StaticElement {
    pub name: QualName,
    pub attrs: Attributes,
    pub text: String,
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_document_extract_links_no_base_url() {
        let page = Page::from_html(
            r#"
            <html>
                <body>
                    <a href="/test">Test</a>
                    <a href="https://cool-site.com/test">Test</a>
                </body>
            </html>
            "#
            .to_string(),
        );
        let links = page.extract_links();
        println!("links: {:?}", links);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].href, format!("{}/test", FALLBACK_URL));
        assert_eq!(links[0].path, "/test");
        assert_eq!(links[0].link_type, LinkType::Internal);
        assert_eq!(links[1].href, "https://cool-site.com/test");
        assert_eq!(links[1].path, "/test");
        assert_eq!(links[1].link_type, LinkType::External);
    }
    #[test]
    fn test_document_extract_links_with_base_url() {
        let mut page = Page::from_html(
            r#"
            <html>
                <body>
                    <a href="/test">Test</a>    
                    <a href="https://cool-site.com/test">Test</a>
                </body>
            </html>
            "#
            .to_string(),
        );
        const TEST_URL: &str = "https://sample.com";
        page.set_url(TEST_URL);
        let links = page.extract_links();
        println!("links: {:?}", links);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].href, format!("{}/test", TEST_URL));
        assert_eq!(links[0].path, "/test");
        assert_eq!(links[0].link_type, LinkType::Internal);
        assert_eq!(links[1].href, "https://cool-site.com/test");
        assert_eq!(links[1].path, "/test");
        assert_eq!(links[1].link_type, LinkType::External);
    }
}
