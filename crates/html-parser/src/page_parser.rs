use futures::stream::StreamExt;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use thiserror::Error;
use tokio::time::Duration;
use url::Url;

#[derive(Debug, Error)]
pub enum PageParserError {
    #[error("Failed to fetch URL: {0}")]
    FetchError(String),
    #[error("Failed to parse HTML: {0}")]
    HtmlParseError(String),
    #[error("Lighthouse analysis failed: {0}")]
    LighthouseError(String),
    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),
    #[error("Document not set: {0}")]
    DocumentNotSet(String),
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
pub struct Heading {
    pub tag: String,
    pub text: String,
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

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct Performance {
    pub load_time: String,
    pub mobile_responsive: bool,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct BaseInfo {
    pub base_url: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct PageAnalysis {
    pub meta_tags: MetaTagInfo,
    pub headings: Vec<Heading>,
    pub images: Vec<Image>,
    pub links: Vec<Links>,
    pub base_info: BaseInfo,
}

fn normalize_url(url: &str) -> String {
    url.trim_end_matches('/').to_string()
}

pub trait FromBaseUrl {
    fn to_url(self) -> Result<Url, PageParserError>;
}

impl FromBaseUrl for Url {
    fn to_url(self) -> Result<Url, PageParserError> {
        Ok(self)
    }
}

impl FromBaseUrl for String {
    fn to_url(self) -> Result<Url, PageParserError> {
        Url::parse(&self).map_err(|e| PageParserError::UrlParseError(e.to_string()))
    }
}

impl FromBaseUrl for &String {
    fn to_url(self) -> Result<Url, PageParserError> {
        Url::parse(self).map_err(|e| PageParserError::UrlParseError(e.to_string()))
    }
}

impl FromBaseUrl for &str {
    fn to_url(self) -> Result<Url, PageParserError> {
        Url::parse(self).map_err(|e| PageParserError::UrlParseError(e.to_string()))
    }
}

pub struct PageParser {
    href: String,
    path: String,
    base_url: Url,
    html_content: Option<String>,
}
// impl Into<Url> for String {
//     fn into(self) -> Url {
//         return Url::parse(&self).unwrap();
//     }
// }

impl PageParser {
    pub fn new<T: FromBaseUrl>(base_url: T) -> Result<Self, PageParserError> {
        let base_url = base_url.to_url()?;
        let path = base_url.clone().path().to_string();
        let href = normalize_url(base_url.clone().as_str());
        Ok(PageParser {
            base_url,
            href,
            path,
            html_content: None,
        })
    }
    pub fn get_href(&self) -> String {
        self.href.clone()
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn set_content(&mut self, html: String) {
        self.html_content = Some(html);
    }

    pub fn set_document(&mut self, document: Html) {
        self.html_content = Some(document.html());
    }

    pub async fn fetch(&mut self) -> Result<(), PageParserError> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| PageParserError::FetchError(e.to_string()))?;

        let response = client
            .get(self.base_url.clone())
            .send()
            .await
            .map_err(|e| PageParserError::FetchError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PageParserError::FetchError(format!(
                "Failed to fetch URL: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| PageParserError::FetchError(e.to_string()))?;

        self.set_content(body);
        Ok(())
    }

    fn get_document(&self) -> Result<Html, PageParserError> {
        let html = self
            .html_content
            .as_ref()
            .ok_or(PageParserError::DocumentNotSet(
                "Document not set".to_string(),
            ))?;
        Ok(Html::parse_document(html))
    }

    pub fn extract_base(&self) -> BaseInfo {
        BaseInfo {
            base_url: self.base_url.to_string(),
            path: self.path.clone(),
        }
    }

    pub fn extract_meta_tags(&self) -> MetaTagInfo {
        let mut meta_tags = MetaTagInfo::default();
        let document = self.get_document().unwrap();

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
    pub fn extract_links(&self) -> Vec<Links> {
        let document = self.get_document().unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let mut links = Vec::new();

        for link in document.select(&link_selector) {
            if let Some(href) = link.value().attr("href") {
                let url = self
                    .base_url
                    .join(href)
                    .unwrap_or_else(|_| self.base_url.clone());
                let link_type = if url.host_str() == Some(self.base_url.host_str().unwrap()) {
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

    pub fn extract_images(&self) -> Vec<Image> {
        let document = self.get_document().unwrap();
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

    pub async fn analyze_page(&self) -> Result<PageAnalysis, PageParserError> {
        // let document = self.get_document()?;

        // Run all extractors concurrently
        let (meta_tags, headings, images, links) = tokio::join!(
            async { self.extract_meta_tags() },
            async { self.extract_headings() },
            async { self.extract_images() },
            async { self.extract_links() }
        );

        Ok(PageAnalysis {
            meta_tags,
            headings,
            images,
            links,
            base_info: self.extract_base(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Response, Server};
    use std::convert::Infallible;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

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
        let document = Html::parse_document(html);
        let mut parser = PageParser::new(Url::parse("https://example.com").unwrap()).unwrap();
        parser.set_content(html.to_string());

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

        let mut parser = PageParser::new(Url::parse("https://example.com").unwrap()).unwrap();
        parser.set_content(html.to_string());

        let links = parser.extract_links();

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
        let document = Html::parse_document(html);
        let mut parser = PageParser::new(Url::parse("https://example.com").unwrap()).unwrap();
        parser.set_content(html.to_string());

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
        let mut parser = PageParser::new(Url::parse(&mock_url).unwrap()).unwrap();
        assert_eq!(parser.path, "/");
        parser.fetch().await.unwrap();

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
