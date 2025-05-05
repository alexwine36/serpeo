use reqwest::{Client, redirect};
use scraper::{
    ElementRef, Html, Selector,
    node::{Attributes, Element},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::{collections::HashMap, num::NonZeroU16, time::Duration};
use thiserror::Error;
use tokio::time::Instant;
use url::Url;

use super::link_parser::{FromUrl, Link, parse_link};

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
    #[error("Selector parse error: {0}")]
    SelectorParseError(String),
    #[error("Link parse error: {0}")]
    LinkParseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Page {
    url: Option<Url>,
    html: Option<String>,
    meta_tags: Arc<StdMutex<Option<MetaTagInfo>>>,
    images: Arc<StdMutex<Option<Vec<Image>>>>,
    content_length: Option<u64>,
    elapsed: Option<f32>,
    status_code: Option<NonZeroU16>,
}

const FALLBACK_URL: &str = "https://example.com";

impl Page {
    pub fn from_html(html: String) -> Self {
        Self {
            url: None,
            html: Some(html),
            meta_tags: Arc::new(StdMutex::new(None)),
            images: Arc::new(StdMutex::new(None)),
            content_length: None,
            
            elapsed: None,
            status_code: None,
        }
    }

    pub fn set_url<T: FromUrl>(&mut self, url: T) {
        self.url = Some(url.to_url().unwrap());
    }

    pub fn get_url(&self) -> Url {
        self.url
            .clone()
            .unwrap_or(Url::parse(FALLBACK_URL).unwrap())
    }

    pub fn get_html(&self) -> Option<String> {
        self.html.clone()
    }

    pub fn get_content_length(&self) -> Option<u64> {
        self.content_length.clone()
    }

    pub fn get_redirected(&self) -> bool {
        if self.status_code.is_some() && self.status_code.unwrap() >= NonZeroU16::new(300).unwrap() && self.status_code.unwrap() < NonZeroU16::new(400).unwrap() {
            true
        } else {
            false
        }
    }

    pub fn set_content(&mut self, html: String) {
        self.html = Some(html);
    }

    pub fn get_elapsed(&self) -> Option<f32> {
        self.elapsed.clone()
    }

    pub fn get_status_code(&self) -> Option<NonZeroU16> {
        self.status_code.clone()
    }

    pub async fn from_url<T: FromUrl>(url: T) -> Result<Self, PageError> {
        let url = url.to_url().unwrap();
        let redirect_status_code = Arc::new(AtomicU16::new(0));
        let redirect_status_code_clone = redirect_status_code.clone();
        let client = Client::
        builder()
        .redirect(redirect::Policy::custom(move |attempt| {
            redirect_status_code_clone.store(attempt.status().into(), Ordering::Relaxed);
            
            redirect::Policy::default().redirect(attempt)
        }))
        // .redirect(redirect::Policy::none())
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| PageError::FetchError(e.to_string()))?;
        let start_time = Instant::now();
        let response =
            client.get(url.clone()).send().await.map_err(|e| {
                PageError::FetchError(format!("Failed to fetch URL: {} {}", url, e))
            })?;

        let elapsed = start_time.elapsed().as_millis() as f32;
        let mut status_code = u16::from(response.status());
        if redirect_status_code.load(Ordering::Relaxed) != 0 {
            status_code = redirect_status_code.load(Ordering::Relaxed);
        }
        
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
            meta_tags: Arc::new(StdMutex::new(None)),
            images: Arc::new(StdMutex::new(None)),
            content_length,
            elapsed: Some(elapsed),
            status_code: NonZeroU16::new(status_code),
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
        let selector =
            Selector::parse(selector).map_err(|e| PageError::SelectorParseError(e.to_string()))?;
        let element = document
            .select(&selector)
            .next()
            .ok_or(PageError::ElementNotFound)?;
        Ok(element.value().clone())
    }
}

impl Page {
    // Images
    fn set_images(&self) {
        let document = self.get_document().unwrap();
        let img_selector = Selector::parse("img").unwrap();
        let mut images = Vec::new();

        for img in document.select(&img_selector) {
            let src = img.value().attr("src").unwrap_or_default().to_string();
            let alt = img.value().attr("alt").map(|s| s.to_string());
            let srcset = img.value().attr("srcset").map(|s| s.to_string());
            images.push(Image { src, alt, srcset });
        }

        let _ = self.images.lock().unwrap().insert(images);
    }

    fn get_images(&self) -> Option<Vec<Image>> {
        if self.images.lock().unwrap().is_none() {
            self.set_images();
        }
        self.images.lock().unwrap().clone()
    }

    pub fn extract_images(&self) -> Vec<Image> {
        self.get_images().unwrap_or_default()
    }

    // Meta Tags
    fn set_meta_tags(&self) {
        
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

        let _ = self.meta_tags.lock().unwrap().insert(meta_tags);
    }

    fn get_meta_tags(&self) -> MetaTagInfo {
        if self.meta_tags.lock().unwrap().is_none() {
            self.set_meta_tags();
        }
        self.meta_tags.lock().unwrap().clone().unwrap_or_default()
    }

    pub fn extract_meta_tags(&self) -> MetaTagInfo {
        self.get_meta_tags()
    }

    // Links
    pub fn extract_links(&self) -> Result<Vec<Link>, PageError> {
        let document = self.get_document().unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let mut links = Vec::new();

        for link in document.select(&link_selector) {
            if let Some(href) = link.value().attr("href") {
                let base_url = self
                    .url
                    .clone()
                    .unwrap_or(Url::parse(FALLBACK_URL).unwrap());
                let link = parse_link(href, base_url)
                    .map_err(|e| PageError::LinkParseError(e.to_string()))?;
                links.push(link);
            }
        }

        Ok(links)
    }
    pub fn extract_headings(&self) -> Vec<Heading> {
        let document = self.get_document().unwrap();
        let mut headings: Vec<Heading> = Vec::new();

        for i in 1..=6 {
            let selector = format!("h{}", i);
            let heading_selector = Selector::parse(&selector).unwrap();
            let res = document.select(&heading_selector);
            for heading in res {
                let text = heading.inner_html();
                headings.push(Heading {
                    tag: selector.clone(),
                    text,
                });
            }
        }

        headings
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

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct Heading {
    pub tag: String,
    pub text: String,
}

#[cfg(test)]
mod tests {

    use crate::utils::link_parser::LinkType;

    use super::*;
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Response, Server};
    use std::convert::Infallible;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

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
        let links = page.extract_links().unwrap();
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
        let links = page.extract_links().unwrap();
        println!("links: {:?}", links);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].href, format!("{}/test", TEST_URL));
        assert_eq!(links[0].path, "/test");
        assert_eq!(links[0].link_type, LinkType::Internal);
        assert_eq!(links[1].href, "https://cool-site.com/test");
        assert_eq!(links[1].path, "/test");
        assert_eq!(links[1].link_type, LinkType::External);
    }

    #[tokio::test]
    async fn test_extract_meta_tags() {
        let html = r#"
                <html>
                    <head>
                        <title>Test Page</title>
                        <meta name="description" content="This is a test description">
                        <meta name="robots" content="index, follow">
                        <meta property="og:title" content="Test OG Title">
                        <meta property="twitter:card" content="summary">
                        <link rel="sitemap" href="https://example.com/sitemap.xml">
                    </head>
                </html>
            "#;

        let parser = Page::from_html(html.to_string());
        // parser.set_content(html.to_string());

        let meta_tags = parser.extract_meta_tags();

        assert_eq!(meta_tags.title, Some("Test Page".to_string()));
        assert_eq!(
            meta_tags.description,
            Some("This is a test description".to_string())
        );
        assert_eq!(meta_tags.robots, Some("index, follow".to_string()));
        assert_eq!(
            meta_tags.og_tags.get("title"),
            Some(&"Test OG Title".to_string())
        );
        assert_eq!(
            meta_tags.twitter_tags.get("card"),
            Some(&"summary".to_string())
        );
        assert_eq!(
            meta_tags.sitemap,
            Some("https://example.com/sitemap.xml".to_string())
        );
    }

    #[tokio::test]
    async fn test_extract_links() {
        let html = r#"
                <html>
                    <body>
                        <a href="/internal-link">Internal Link</a>
                        <a href="https://external.com">External Link</a>
                    </body>
                </html>
            "#;

        let mut parser = Page::from_html(html.to_string());
        parser.set_url(Url::parse("https://example.com").unwrap());

        let links = parser.extract_links().unwrap();

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].href, "https://example.com/internal-link");
        assert_eq!(links[0].link_type, LinkType::Internal);
        assert_eq!(links[1].href, "https://external.com/");
        assert_eq!(links[1].link_type, LinkType::External);
    }

    #[tokio::test]
    async fn test_extract_headings() {
        let html = r#"
                <html>
                    <body>
                        <h1>Main Heading</h1>
                        <h2>Subheading</h2>
                        <h3>Another Subheading</h3>
                    </body>
                </html>
            "#;

        let mut parser = Page::from_html(html.to_string());
        parser.set_url(Url::parse("https://example.com").unwrap());

        let headings = parser.extract_headings();

        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].tag, "h1");
        assert_eq!(headings[0].text, "Main Heading");
        assert_eq!(headings[1].tag, "h2");
        assert_eq!(headings[1].text, "Subheading");
        assert_eq!(headings[2].tag, "h3");
        assert_eq!(headings[2].text, "Another Subheading");
    }

    #[tokio::test]
    async fn test_fetch_document() {
        let addr = start_test_server().await;
        let mock_url = format!("http://{}", addr);
        let parser = Page::from_url(mock_url).await.unwrap();

        // assert_eq!(parser.path, "/");
        // parser.fetch().await.unwrap();

        let tags = parser.extract_meta_tags();
        assert_eq!(tags.title, Some("Test Page".to_string()));
        assert_eq!(
            tags.description,
            Some("This is a test description".to_string())
        );
        assert_eq!(tags.robots, Some("index, follow".to_string()));
        assert_eq!(
            tags.og_tags.get("title"),
            Some(&"Test OG Title".to_string())
        );
        assert_eq!(tags.twitter_tags.get("card"), Some(&"summary".to_string()));
        assert_eq!(tags.canonical, Some("https://example.com".to_string()));
    }

    async fn start_test_server() -> SocketAddr {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = TcpListener::bind(addr).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);
        let make_svc = make_service_fn(move |_conn| {
            let base_url = base_url.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let base_url = base_url.clone();
                    async move {
                        match req.uri().path() {
                            "/" => Ok::<_, Infallible>(Response::new(Body::from(
                                r#"
                                <html>
                                <head>
                                    <title>Test Page</title>
                                    <meta name="description" content="This is a test description">
                                    <meta name="robots" content="index, follow">
                                    <meta property="og:title" content="Test OG Title">
                                    <meta property="twitter:card" content="summary">
                                    <link rel="canonical" href="https://example.com">
                                </head>
                                    <body>
                                        <a href="/page1">Page 1</a>
                                        <a href="/page2">Page 2</a>
                                        <a href="https://external.com">External</a>
                                    </body>
                                </html>
                            "#,
                            ))),
                            "/sitemap.xml" => {
                                let sitemap = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">
                                    <url><loc>{}/page1</loc></url>
                                    <url><loc>{}/page3</loc></url>
                                    </urlset>"#,
                                    base_url, base_url
                                );
                                Ok(Response::new(Body::from(sitemap)))
                            }
                            "/page1" => Ok(Response::new(Body::from(
                                "<html><body>Page 1</body></html>",
                            ))),
                            "/page2" => Ok(Response::new(Body::from(
                                "<html><body>Page 2</body></html>",
                            ))),
                            "/page3" => Ok(Response::new(Body::from(
                                "<html><body>Page 3</body></html>",
                            ))),
                            _ => Ok(Response::new(Body::from("404"))),
                        }
                    }
                }))
            }
        });

        tokio::spawn(async move {
            Server::from_tcp(listener.into_std().unwrap())
                .unwrap()
                .serve(make_svc)
                .await
                .unwrap();
        });

        addr
    }
}
