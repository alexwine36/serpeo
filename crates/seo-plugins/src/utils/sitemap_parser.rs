use std::collections::{HashMap, HashSet};

use parking_lot::RwLock;
use reqwest::Client;
use thiserror::Error;
use url::Url;

use super::link_parser::{FromUrl, LinkParseError, parse_link};
use super::page::{Page, PageError};

#[derive(Debug, Error)]
pub enum SitemapParserError {
    #[error("Failed to parse URL: {0}")]
    UrlParseError(#[from] LinkParseError),
    #[error("Failed to parse HTML: {0}")]
    HtmlParseError(String),
    #[error("Failed to parse sitemap: {0}")]
    SitemapError(String),
    #[error("Page error: {0}")]
    PageError(#[from] PageError),
    #[error("Client error: {0}")]
    ClientError(String),
}

type SitemapUrls = HashMap<String, Option<HashSet<String>>>;

pub struct SitemapParser {
    _url: Url,
    base_url: Url,
    client: Client,
    sitemap_urls: RwLock<SitemapUrls>,
}

impl SitemapParser {
    pub fn new<T: FromUrl>(url: T) -> Result<Self, SitemapParserError> {
        let url = url.to_url().map_err(SitemapParserError::UrlParseError)?;
        let base_url = url.clone();
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
            .build()
            .map_err(|e| SitemapParserError::ClientError(e.to_string()))?;
        Ok(Self {
            _url: url,
            base_url,
            client,
            sitemap_urls: RwLock::new(SitemapUrls::new()),
        })
    }

    pub async fn get_sitemap(&self) -> Result<HashSet<String>, SitemapParserError> {
        self.fetch_sitemap().await?;

        let mut results = HashSet::new();
        let sitemaps = self.sitemap_urls.read();
        // println!("Sitemaps: {:?}", sitemaps);
        for (sitemap, urls) in sitemaps.iter() {
            if let Some(urls) = urls {
                println!("Sitemap: {:?}, urls: {:?}", sitemap, urls.clone().len());
                results.extend(urls.iter().cloned());
            }
        }
        Ok(results)
    }

    async fn discover_sitemap_url(&self) -> Result<Option<String>, SitemapParserError> {
        let parser = Page::from_url(self.base_url.clone())
            .await
            .map_err(SitemapParserError::PageError)?;

        let meta = parser.extract_meta_tags();
        let sitemap_url = meta.sitemap;

        Ok(sitemap_url)
    }

    async fn parse_sitemap_urls(
        &self,
        content: &str,
    ) -> Result<HashSet<String>, SitemapParserError> {
        let content_clone = content.to_string();

        let document = match roxmltree::Document::parse(&content_clone) {
            Ok(doc) => doc,
            Err(_) => return Ok(HashSet::new()), // Return empty set on parse failure
        };

        let mut urls = HashSet::new();
        for node in document.descendants() {
            if node.has_tag_name("loc") {
                if let Some(url) = node.text() {
                    urls.insert(normalize_url(url));
                }
            }
        }
        Ok(urls)
    }

    async fn fetch_sitemap(&self) -> Result<(), SitemapParserError> {
        // Try to find sitemap URL from HTML first
        // let mut sitemap_urls = HashSet::new();
        if let Some(discovered_url) = self.discover_sitemap_url().await? {
            let link = parse_link(&discovered_url, self.base_url.clone())
                .map_err(SitemapParserError::UrlParseError)?;
            self.sitemap_urls
                .write()
                .insert(normalize_url(&link.href), None);
        } else {
            // Fallback to common sitemap locations
            for path in &[
                "/sitemap.xml",
                "/sitemap_index.xml",
                "/sitemap-index.xml",
                "/sitemap/sitemap.xml",
            ] {
                if let Ok(url) = self.base_url.join(path) {
                    self.sitemap_urls
                        .write()
                        .insert(normalize_url(url.as_ref()), None);
                }
            }
        }

        loop {
            let sitemap_urls: Vec<String> = {
                let sitemap_urls = self.sitemap_urls.read().clone();
                sitemap_urls
                    .iter()
                    .filter(|(_, is_processed)| is_processed.is_none())
                    .map(|(url, _)| url.clone())
                    .collect()
            };

            if sitemap_urls.is_empty() {
                break;
            }

            let futures = sitemap_urls.iter().map(|url| self.read_sitemap(url));
            let _results = futures::future::join_all(futures).await;
        }

        Ok(())
    }

    async fn read_sitemap(&self, sitemap_url: &str) -> Result<(), SitemapParserError> {
        println!("Fetching sitemap: {}", sitemap_url);
        let response = self
            .client
            .get(sitemap_url)
            .send()
            .await
            .map_err(|e| SitemapParserError::ClientError(e.to_string()))?;

        let text = response
            .text()
            .await
            .map_err(|e| SitemapParserError::ClientError(e.to_string()))?;

        let urls = self.parse_sitemap_urls(&text).await?;
        let urls_clone = urls.clone();
        {
            let mut sitemap_urls = self.sitemap_urls.write();
            let current_url = sitemap_urls.entry(sitemap_url.to_string()).or_insert(None);
            *current_url = Some(urls);
        }
        let is_index = text.contains("<sitemapindex");
        if is_index {
            for url in urls_clone {
                self.sitemap_urls.write().insert(normalize_url(&url), None);
            }
        }

        Ok(())
    }
}

fn normalize_url(url: &str) -> String {
    url.to_string()
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
    async fn test_remote_sitemap_parse() {
        let base_url = "https://employer.directory.boomerang-nm.com/";
        let sitemap_parser = SitemapParser::new(base_url).unwrap();
        let sitemap_urls = sitemap_parser.get_sitemap().await.unwrap();
        // println!("Sitemap URLs: {:?}", sitemap_urls);
        assert!(!sitemap_urls.is_empty());
    }

    #[tokio::test]
    async fn test_basic_sitemap_parse() {
        let addr = start_base_sitemap_server().await;
        let base_url = format!("http://{}", addr);
        let sitemap_parser = SitemapParser::new(base_url).unwrap();
        let sitemap_urls = sitemap_parser.get_sitemap().await.unwrap();
        println!("Sitemap URLs: {:?}", sitemap_urls);
        assert!(!sitemap_urls.is_empty());
        let base_url_clone = format!("http://{}", addr);
        for path in &["/base", "/index-0", "/index-1"] {
            assert!(
                sitemap_urls.contains(&format!("{}{}", base_url_clone, path)),
                "Sitemap URL not found: {}",
                format!("{}{}", base_url_clone, path)
            );
        }
    }

    #[tokio::test]
    async fn test_basic_sitemap_on_other_path() {
        let addr = start_base_sitemap_server().await;
        let base_url = format!("http://{}/other-path", addr);
        let sitemap_parser = SitemapParser::new(base_url).unwrap();
        let sitemap_urls = sitemap_parser.get_sitemap().await.unwrap();
        println!("Sitemap URLs: {:?}", sitemap_urls);
        assert!(!sitemap_urls.is_empty());
        let base_url_clone = format!("http://{}", addr);
        {
            let path = &"/defined";
            assert!(
                sitemap_urls.contains(&format!("{}{}", base_url_clone, path)),
                "Sitemap URL not found: {}",
                format!("{}{}", base_url_clone, path)
            );
        }
    }
    #[tokio::test]
    async fn test_empty_sitemap_server() {
        let addr = start_empty_sitemap_server().await;
        let base_url = format!("http://{}", addr);
        let sitemap_parser = SitemapParser::new(base_url).unwrap();
        let sitemap_urls = sitemap_parser.get_sitemap().await.unwrap();
        println!("Sitemap URLs: {:?}", sitemap_urls);
        assert!(sitemap_urls.is_empty());
    }

    #[tokio::test]
    async fn test_relative_sitemap_parse() {
        let addr = start_base_sitemap_server().await;
        let base_url = format!("http://{}/relative-sitemap", addr);
        let sitemap_parser = SitemapParser::new(base_url).unwrap();
        let sitemap_urls = sitemap_parser.get_sitemap().await.unwrap();
        println!("Sitemap URLs: {:?}", sitemap_urls);
        assert!(!sitemap_urls.is_empty());
    }

    async fn start_base_sitemap_server() -> SocketAddr {
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
                                <!DOCTYPE html>
                                <html>
                                    <head>
                                        <meta charset="utf-8">
                                        <title>Test Page</title>
                                        <meta name="description" content="Test description">
                                        
                                        
                                    </head>
                                    <body>
                                        <a href="/page1">Page 1</a>
                                        <a href="/page2">Page 2</a>
                                        <a href="/page1?param=value">Page 1 with params</a>
                                        <a href="/page1#section">Page 1 with hash</a>
                                        <a href="/page1?param=value#section">Page 1 with both</a>
                                        <a href="https://external.com">External</a>
                                    </body>
                                </html>
                            "#
                                .to_string(),
                            ))),
                            "/other-path" => {
                                Ok::<_, Infallible>(Response::new(Body::from(format!(
                                    r#"
                                <!DOCTYPE html>
                                <html>
                                    <head>
                                        <meta charset="utf-8">
                                        <title>Test Page</title>
                                        <meta name="description" content="Test description">
                                        <link rel="canonical" href="{}/success">
                                        <link rel="sitemap" href="{}/defined.xml">
                                    </head>
                                    <body>
                                        <a href="/page1">Page 1</a>
                                        <a href="/page2">Page 2</a>
                                        <a href="/page1?param=value">Page 1 with params</a>
                                        <a href="/page1#section">Page 1 with hash</a>
                                        <a href="/page1?param=value#section">Page 1 with both</a>
                                        <a href="https://external.com">External</a>
                                    </body>
                                </html>
                            "#,
                                    base_url, base_url
                                ))))
                            }
                            "/relative-sitemap" => {
                                Ok::<_, Infallible>(Response::new(Body::from(format!(
                                    r#"
                                <!DOCTYPE html>
                                <html>
                                    <head>
                                        <meta charset="utf-8">
                                        <title>Test Page</title>
                                        <meta name="description" content="Test description">
                                        <link rel="canonical" href="{}/success">
                                        <link rel="sitemap" href="/relative-sitemap.xml">
                                    </head>
                                    <body>
                                        <a href="/page1">Page 1</a>
                                        <a href="/page2">Page 2</a>
                                        <a href="/page1?param=value">Page 1 with params</a>
                                        <a href="/page1#section">Page 1 with hash</a>
                                        <a href="/page1?param=value#section">Page 1 with both</a>
                                        <a href="https://external.com">External</a>
                                    </body>
                                </html>
                            "#,
                                    base_url,
                                ))))
                            }
                            "/sitemap_index.xml" => {
                                let sitemap_index = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
                                        <sitemap><loc>{}/sitemap-0.xml</loc></sitemap>
                                    </sitemapindex>"#,
                                    base_url
                                );
                                Ok(Response::new(Body::from(sitemap_index)))
                            }
                            "/sitemap-index.xml" => {
                                let sitemap_index = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
                                        <sitemap><loc>{}/sitemap-1.xml</loc></sitemap>
                                    </sitemapindex>"#,
                                    base_url
                                );
                                Ok(Response::new(Body::from(sitemap_index)))
                            }
                            "/relative-sitemap.xml" => {
                                let sitemap_index = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
                                        <sitemap><loc>{}/sitemap-1.xml</loc></sitemap>
                                    </sitemapindex>"#,
                                    base_url
                                );
                                Ok(Response::new(Body::from(sitemap_index)))
                            }
                            "/sitemap-0.xml" => {
                                let sitemap = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">
                                    <url><loc>{}/index-0</loc></url>
                                   
                                    </urlset>"#,
                                    base_url
                                );
                                Ok(Response::new(Body::from(sitemap)))
                            }
                            "/sitemap-1.xml" => {
                                let sitemap = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">
                                    <url><loc>{}/index-1</loc></url>
                                   
                                    </urlset>"#,
                                    base_url
                                );
                                Ok(Response::new(Body::from(sitemap)))
                            }
                            "/sitemap.xml" => {
                                let sitemap = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">
                                    <url><loc>{}/base</loc></url>
                                    
                                    </urlset>"#,
                                    base_url,
                                );
                                Ok(Response::new(Body::from(sitemap)))
                            }
                            "/defined.xml" => {
                                let sitemap = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">
                                    <url><loc>{}/defined</loc></url>
                                    
                                    </urlset>"#,
                                    base_url,
                                );
                                Ok(Response::new(Body::from(sitemap)))
                            }

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

    async fn start_empty_sitemap_server() -> SocketAddr {
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
                                <!DOCTYPE html>
                                <html>
                                    <head>
                                        <meta charset="utf-8">
                                        <title>Test Page</title>
                                        <meta name="description" content="Test description">
                                        
                                        
                                    </head>
                                    <body>
                                        <a href="/page1">Page 1</a>
                                        <a href="/page2">Page 2</a>
                                        <a href="/page1?param=value">Page 1 with params</a>
                                        <a href="/page1#section">Page 1 with hash</a>
                                        <a href="/page1?param=value#section">Page 1 with both</a>
                                        <a href="https://external.com">External</a>
                                    </body>
                                </html>
                            "#
                                .to_string(),
                            ))),
                            "/other-path" => {
                                Ok::<_, Infallible>(Response::new(Body::from(format!(
                                    r#"
                                <!DOCTYPE html>
                                <html>
                                    <head>
                                        <meta charset="utf-8">
                                        <title>Test Page</title>
                                        <meta name="description" content="Test description">
                                        <link rel="canonical" href="{}/success">
                                        <link rel="sitemap" href="{}/defined.xml">
                                    </head>
                                    <body>
                                        <a href="/page1">Page 1</a>
                                        <a href="/page2">Page 2</a>
                                        <a href="/page1?param=value">Page 1 with params</a>
                                        <a href="/page1#section">Page 1 with hash</a>
                                        <a href="/page1?param=value#section">Page 1 with both</a>
                                        <a href="https://external.com">External</a>
                                    </body>
                                </html>
                            "#,
                                    base_url, base_url
                                ))))
                            }

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
