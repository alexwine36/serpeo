use std::collections::HashSet;

use reqwest::Client;
use thiserror::Error;
use url::Url;

use super::link_parser::{FromUrl, parse_link};
use super::page::{Page, PageError};

#[derive(Debug, Error)]
pub enum SitemapParserError {
    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),
    #[error("Failed to parse HTML: {0}")]
    HtmlParseError(String),
    #[error("Failed to parse sitemap: {0}")]
    SitemapError(String),
    #[error("Page error: {0}")]
    PageError(#[from] PageError),
}

pub struct SitemapParser {
    _url: Url,
    base_url: Url,
    client: Client,
}

impl SitemapParser {
    pub fn new<T: FromUrl>(url: T) -> Result<Self, SitemapParserError> {
        let url = url
            .to_url()
            .map_err(|e| SitemapParserError::UrlParseError(e.to_string()))?;
        let base_url = url.clone();
        Ok(Self {
            _url: url,
            base_url,
            client: Client::new(),
        })
    }

    pub async fn get_sitemap(&self) -> Result<HashSet<String>, SitemapParserError> {
        let sitemap_urls = self.fetch_sitemap().await?;
        // let mut all_urls = HashSet::new();
        Ok(sitemap_urls)
        // for sitemap_url in sitemap_urls {
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
        tokio::task::spawn_blocking(move || -> Result<HashSet<String>, SitemapParserError> {
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
        })
        .await
        .map_err(|e| SitemapParserError::SitemapError(e.to_string()))?
    }

    async fn fetch_sitemap(&self) -> Result<HashSet<String>, SitemapParserError> {
        let mut all_urls = HashSet::new();

        // Try to find sitemap URL from HTML first
        let mut sitemap_urls = HashSet::new();
        if let Some(discovered_url) = self.discover_sitemap_url().await? {
            let link = parse_link(&discovered_url, self.base_url.clone()).unwrap();
            sitemap_urls.insert(normalize_url(&link.href));
        } else {
            // Fallback to common sitemap locations
            for path in &[
                "/sitemap.xml",
                "/sitemap_index.xml",
                "/sitemap-index.xml",
                "/sitemap/sitemap.xml",
            ] {
                if let Ok(url) = self.base_url.join(path) {
                    sitemap_urls.insert(normalize_url(url.as_ref()));
                }
            }
        }

        // Process each potential sitemap URL
        for sitemap_url in sitemap_urls {
            println!("Fetching sitemap: {}", sitemap_url);
            let response = match self.client.get(&sitemap_url).send().await {
                Ok(resp) => resp,
                Err(_) => continue,
            };

            let text = match response.text().await {
                Ok(t) => t,
                Err(_) => continue,
            };

            let urls = self.parse_sitemap_urls(&text).await?;

            // Check if this is a sitemap index
            let is_index = text.contains("<sitemapindex");
            println!("Is index: {}", is_index);
            if is_index {
                // Fetch each referenced sitemap
                for url in urls {
                    if let Ok(resp) = self.client.get(&url).send().await {
                        if let Ok(text) = resp.text().await {
                            if let Ok(sub_urls) = self.parse_sitemap_urls(&text).await {
                                all_urls.extend(sub_urls);
                            }
                        }
                    }
                }
            } else {
                all_urls.extend(urls);
            }
        }

        Ok(all_urls)
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
        println!("Sitemap URLs: {:?}", sitemap_urls);
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
