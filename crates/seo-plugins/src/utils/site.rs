use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use markup5ever::QualName;
use reqwest::Client;
use scraper::{
    ElementRef, Html, Selector,
    node::{Attributes, Element},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::Instant;
use url::Url;

use super::{
    config::RuleResult,
    link_parser::{FromUrl, LinkType, parse_link},
    page::{Page, PageError},
    registry::PluginRegistry,
    sitemap_parser::SitemapParser,
};

#[derive(Debug, Error)]
pub enum SiteError {
    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),
    #[error("Page error: {0}")]
    PageError(#[from] PageError),
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, PartialEq, Eq, Hash)]
pub enum LinkSourceType {
    Sitemap,
    Root,
    Link,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, PartialEq, Eq, Hash)]
pub struct PageLinkSource {
    pub link_source_type: LinkSourceType,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct PageResult {
    pub error: bool,
    pub results: Vec<RuleResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct PageLink {
    pub url: String,
    pub link_type: LinkType,
    pub found_in: HashSet<PageLinkSource>,
    pub result: Option<PageResult>,
}

pub struct Site {
    url: Url,
    links: Arc<Mutex<HashMap<String, PageLink>>>,
    registry: Arc<Mutex<PluginRegistry>>,
}

impl Site {
    pub fn new<T: FromUrl>(url: T, registry: PluginRegistry) -> Result<Self, SiteError> {
        let url = url
            .to_url()
            .map_err(|e| SiteError::UrlParseError(e.to_string()))?;

        Ok(Self {
            url,
            links: Arc::new(Mutex::new(HashMap::new())),
            registry: Arc::new(Mutex::new(registry)),
        })
    }

    async fn fetch_sitemap(&self) -> Result<HashSet<String>, SiteError> {
        let sitemap_parser = SitemapParser::new(self.url.clone()).unwrap();
        let sitemap_urls = sitemap_parser.get_sitemap().await.unwrap();
        Ok(sitemap_urls)
    }

    async fn record_page_result(&mut self, url: Url, result: PageResult) -> Result<(), SiteError> {
        let mut links = self.links.lock().await;
        if let Some(link) = links.get_mut(&url.to_string()) {
            link.result = Some(result);
        }
        Ok(())
    }

    fn clean_url(url: String) -> String {
        let mut url = url;
        if let Some(query_start) = url.find('?') {
            url.truncate(query_start);
        }
        if let Some(hash_start) = url.find('#') {
            url.truncate(hash_start);
        }
        url
    }

    async fn add_link(
        &mut self,
        url: &str,
        page_link_source: PageLinkSource,
    ) -> Result<(), SiteError> {
        let link = parse_link(url, self.url.clone()).unwrap();
        let url_string = Self::clean_url(link.href.clone());
        let url_string2 = Self::clean_url(link.href.clone());
        let mut links = self.links.lock().await;
        if let Some(existing) = links.get_mut(&url_string) {
            existing.found_in.insert(page_link_source);
        } else {
            let mut found_in = HashSet::new();
            found_in.insert(page_link_source);
            links.insert(
                url_string,
                PageLink {
                    url: url_string2,
                    link_type: link.link_type,
                    found_in,
                    result: None,
                },
            );
        }
        Ok(())
    }

    pub async fn crawl(&mut self) -> Result<Vec<PageLink>, SiteError> {
        let sitemap_urls = self.fetch_sitemap().await?;
        for sitemap_url in sitemap_urls {
            self.add_link(
                &sitemap_url,
                PageLinkSource {
                    link_source_type: LinkSourceType::Sitemap,
                    url: self.url.join("/sitemap.xml").unwrap().to_string(),
                },
            )
            .await?;
        }
        {
            self.add_link(
                &self.url.to_string(),
                PageLinkSource {
                    link_source_type: LinkSourceType::Root,
                    url: self.url.to_string(),
                },
            )
            .await?;
        }

        while let Some(url) = {
            let links = self.links.lock().await;
            links
                .values()
                .filter(|link| link.link_type == LinkType::Internal)
                .find_map(|link| {
                    if link.result.is_none() {
                        Some(link.url.clone())
                    } else {
                        None
                    }
                })
        } {
            println!("processing page: {}", url);
            self.process_page(Url::parse(&url).unwrap()).await?;
        }

        Ok(self.links.lock().await.values().cloned().collect())
    }

    async fn process_page(&mut self, url: Url) -> Result<(), SiteError> {
        let page = Page::from_url(url.clone())
            .await
            .map_err(|e| SiteError::PageError(e));

        if let Err(e) = page {
            return self
                .record_page_result(
                    url,
                    PageResult {
                        error: true,
                        results: vec![],
                    },
                )
                .await;
        } else if let Ok(page) = page {
            let url_string = url.to_string();
            let links = page.extract_links().map_err(|e| SiteError::PageError(e))?;

            // TODO: Run plugins on the page
            let results = {
                let registry = self.registry.lock().await;
                registry.analyze(&page)
            };
            // println!("results: {:#?}", results);
            self.record_page_result(
                url,
                PageResult {
                    error: false,
                    results,
                },
            )
            .await?;

            for link in links {
                self.add_link(
                    &link.href,
                    PageLinkSource {
                        link_source_type: LinkSourceType::Link,
                        url: url_string.clone(),
                    },
                )
                .await?;
            }
        }

        Ok(())
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

    #[test]
    fn simple_url_test() {
        let base_url = Url::parse("https://example.com/something").unwrap();
        let test_url = base_url.join("/test").unwrap();
        let test_url_2 = base_url.join("test/test").unwrap();
        assert_eq!(test_url.to_string(), "https://example.com/test");
        assert_eq!(test_url_2.to_string(), "https://example.com/test/test");
    }

    #[tokio::test]
    async fn test_site_crawl() {
        let addr = start_server().await;
        let base_url = format!("http://{}", addr);
        let mut site = Site::new(base_url, PluginRegistry::default_with_config()).unwrap();
        site.crawl().await.unwrap();
        let links = site.links.lock().await;
        println!("links: {:#?}", links);
        assert_eq!(links.len(), 7);
        let base_url = format!("http://{}", addr);
        let page1 = links.get(&format!("{}/page1", base_url)).unwrap();
        assert_eq!(page1.found_in.len(), 2);
        for path in [
            "/", "/page1", "/page2", "/page3", "/page4", "/page5", "/page6",
        ] {
            assert!(links.get(&format!("{}{}", base_url, path)).is_some());
        }

        // let page1 = links.get(&format!("{}/page1", base_url)).unwrap();
        // assert_eq!(page1.found_in.len(), 3);
    }

    async fn start_server() -> SocketAddr {
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
                            "/" => Ok::<_, Infallible>(Response::new(Body::from(format!(
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
                                        <a href="/page1">Page 1</a>
                                        <a href="/page2">Page 2</a>

                                        
                                    </body>
                                </html>
                            "#,
                            )))),
                            "/page1" => Ok::<_, Infallible>(Response::new(Body::from(format!(
                                r#"
                                <!DOCTYPE html>
                                <html>
                                    <head>
                                        <meta charset="utf-8">
                                        <title>Test Page</title>
                                        <meta name="description" content="Test description">
                                        
                                        
                                    </head>
                                    <body>
                                        <a href="/page5">Page 5</a>
                                        

                                        
                                    </body>
                                </html>
                            "#,
                            )))),
                            "/page5" => Ok::<_, Infallible>(Response::new(Body::from(format!(
                                r#"
                                <!DOCTYPE html>
                                <html>
                                    <head>
                                        <meta charset="utf-8">
                                        <title>Test Page</title>
                                        <meta name="description" content="Test description">
                                        
                                        
                                    </head>
                                    <body>
                                        <a href="/page6">Page 6</a>
                                        

                                        
                                    </body>
                                </html>
                            "#,
                            )))),
                            "/page6" => Ok::<_, Infallible>(Response::new(Body::from(format!(
                                r#"
                                <!DOCTYPE html>
                                <html>
                                    <head>
                                        <meta charset="utf-8">
                                        <title>Test Page</title>
                                        <meta name="description" content="Test description">
                                        
                                        
                                    </head>
                                    <body>
                                        <a href="/page4">Page 4</a>
                                        

                                        
                                    </body>
                                </html>
                            "#,
                            )))),
                            "/sitemap.xml" => {
                                let sitemap = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">
                                    <url><loc>{}/page3</loc></url>
                                    <url><loc>{}/page1</loc></url>
                                    
                                    </urlset>"#,
                                    base_url, base_url
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
}
