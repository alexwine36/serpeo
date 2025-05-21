use std::collections::{HashMap, HashSet};

use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use tauri_specta::Event;
use thiserror::Error;
use tokio::sync::{Mutex, MutexGuard};
use url::Url;

use crate::utils::{
    config::RuleResult,
    link_parser::{FromUrl, LinkParseError, LinkType, parse_link},
    page::{Page, PageError},
    registry::PluginRegistry,
    sitemap_parser::{SitemapParser, SitemapParserError},
};

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

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct CrawlResult {
    pub page_results: Vec<PageLink>,
    pub site_result: Vec<RuleResult>,
    total_pages: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub enum AnalysisProgressType {
    FoundLink,
    AnalyzedPage(PageLink),
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct AnalysisProgress {
    pub progress_type: AnalysisProgressType,
    pub url: Option<String>,
    pub total_pages: u32,
    pub completed_pages: u32,
}

pub type ProgressCallback = Option<Box<dyn Fn(AnalysisProgress) + Send + Sync + 'static>>;

#[derive(Debug, Error)]
pub enum SiteAnalyzerError {
    #[error("Failed to parse URL: {0}")]
    UrlParseError(#[from] LinkParseError),
    #[error("Failed to join URL: {0}")]
    UrlJoinError(#[from] url::ParseError),
    #[error("Page error: {0}")]
    PageError(#[from] PageError),
    #[error("Sitemap parser error: {0}")]
    SitemapParserError(#[from] SitemapParserError),
}

pub struct SiteAnalyzer {
    url: Url,
    links: Arc<Mutex<HashMap<String, PageLink>>>,
    registry: Arc<Mutex<PluginRegistry>>,
    progress_callback: Arc<Mutex<ProgressCallback>>,
}

impl SiteAnalyzer {
    pub fn new<T: FromUrl>(url: T, registry: PluginRegistry) -> Result<Self, SiteAnalyzerError> {
        let url = url.to_url().map_err(SiteAnalyzerError::UrlParseError)?;
        Ok(Self {
            url,
            links: Arc::new(Mutex::new(HashMap::new())),
            registry: Arc::new(Mutex::new(registry)),
            progress_callback: Arc::new(Mutex::new(None)),
        })
    }

    pub fn new_with_default<T: FromUrl>(url: T) -> Result<Self, SiteAnalyzerError> {
        let url = url.to_url().map_err(SiteAnalyzerError::UrlParseError)?;
        Ok(Self {
            url,
            links: Arc::new(Mutex::new(HashMap::new())),
            registry: Arc::new(Mutex::new(PluginRegistry::default_with_config())),
            progress_callback: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn with_progress_callback(
        self,
        callback: impl Fn(AnalysisProgress) + Send + Sync + 'static,
    ) -> Self {
        let _ = self
            .progress_callback
            .lock()
            .await
            .insert(Box::new(callback));
        self
    }

    pub fn get_links(&self) -> HashMap<String, PageLink> {
        futures::executor::block_on(async { self.links.lock().await.clone() })
    }

    async fn report_progress(
        &self,
        progress_type: AnalysisProgressType,
        url: Option<String>,
        links: &MutexGuard<'_, HashMap<String, PageLink>>,
    ) {
        if let Some(callback) = &self.progress_callback.lock().await.as_ref() {
            callback(AnalysisProgress {
                progress_type,
                url,
                total_pages: links
                    .values()
                    .filter(|link| link.link_type == LinkType::Internal)
                    .count() as u32,
                completed_pages: links
                    .values()
                    .filter(|link| link.link_type == LinkType::Internal && link.result.is_some())
                    .count() as u32,
            });
        }
    }

    async fn fetch_sitemap(&self) -> Result<HashSet<String>, SiteAnalyzerError> {
        let sitemap_parser =
            SitemapParser::new(self.url.clone()).map_err(SiteAnalyzerError::SitemapParserError)?;
        let sitemap_urls = sitemap_parser.get_sitemap().await?;
        Ok(sitemap_urls)
    }

    async fn record_page_result(
        &mut self,
        url: &Url,
        result: PageResult,
    ) -> Result<(), SiteAnalyzerError> {
        let mut links = self.links.lock().await;
        if let Some(link) = links.get_mut(&url.to_string()) {
            link.result = Some(result);
            self.report_progress(
                AnalysisProgressType::AnalyzedPage(link.clone()),
                Some(url.to_string()),
                &links,
            )
            .await;
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
    ) -> Result<(), SiteAnalyzerError> {
        let link = parse_link(url, self.url.clone()).map_err(SiteAnalyzerError::UrlParseError)?;
        {
            if page_link_source.link_source_type == LinkSourceType::Sitemap {
                println!("adding link: {}", url);
            }
        }
        let url_string = Self::clean_url(link.href.clone());
        let url_string2 = Self::clean_url(link.href.clone());
        let url_string3 = Self::clean_url(link.href.clone());
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
            self.report_progress(AnalysisProgressType::FoundLink, Some(url_string3), &links)
                .await;
        }
        Ok(())
    }

    async fn process_page(&mut self, url: Url) -> Result<(), SiteAnalyzerError> {
        let page = Page::from_url(url.clone())
            .await
            .map_err(SiteAnalyzerError::PageError);

        if page.is_err() {
            let _ = self
                .record_page_result(
                    &url,
                    PageResult {
                        error: true,
                        results: vec![],
                    },
                )
                .await;
            return Ok(());
        }

        let page = page?;
        let results = {
            let registry = self.registry.lock().await;
            registry.analyze_async(&page).await?
        };

        // Record the page results
        self.record_page_result(
            &url,
            PageResult {
                error: false,
                results,
            },
        )
        .await?;

        // Extract and add any new links found on the page
        let links = page.extract_links().map_err(SiteAnalyzerError::PageError)?;
        for link in links {
            self.add_link(
                &link.href,
                PageLinkSource {
                    link_source_type: LinkSourceType::Link,
                    url: url.to_string(),
                },
            )
            .await?;
        }

        Ok(())
    }

    pub async fn crawl(&mut self) -> Result<CrawlResult, SiteAnalyzerError> {
        let sitemap_urls = self.fetch_sitemap().await?;
        println!("sitemap_urls: {:#?}", sitemap_urls);
        for sitemap_url in sitemap_urls {
            self.add_link(
                &sitemap_url,
                PageLinkSource {
                    link_source_type: LinkSourceType::Sitemap,
                    url: self
                        .url
                        .join(&sitemap_url)
                        .map_err(SiteAnalyzerError::UrlJoinError)?
                        .to_string(),
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

        loop {
            // Get all unprocessed internal links
            let links = self.links.lock().await;
            let internal_links: Vec<String> = links
                .iter()
                .filter(|(_, link)| link.link_type == LinkType::Internal && link.result.is_none())
                .map(|(url, _)| url.clone())
                .collect();
            drop(links);

            if internal_links.is_empty() {
                break;
            }

            // Process pages concurrently with a limit of 10 concurrent requests
            let mut stream = stream::iter(internal_links)
                .map(|url| {
                    #[allow(clippy::unwrap_used)]
                    let url = url
                        .to_url()
                        .map_err(SiteAnalyzerError::UrlParseError)
                        .unwrap();
                    let registry = self.registry.clone();
                    let links = self.links.clone();
                    let progress_callback = self.progress_callback.clone();
                    async move {
                        let mut analyzer = SiteAnalyzer {
                            url: url.clone(),
                            links,
                            registry,
                            progress_callback,
                        };
                        analyzer.process_page(url).await
                    }
                })
                .buffer_unordered(10);

            while let Some(result) = stream.next().await {
                result?;
            }
        }

        let site_result = self.registry.lock().await.analyze_site(self).await?;
        let links = self.links.lock().await;
        let page_results: Vec<PageLink> = links.values().cloned().collect();
        Ok(CrawlResult {
            page_results,
            site_result,
            total_pages: links.len() as u32,
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
        let mut site = SiteAnalyzer::new_with_default(base_url).unwrap();
        let results = site.crawl().await.unwrap();
        let links = site.links.lock().await;
        // println!("links: {:#?}", links);
        // assert_eq!(links.len(), 7);
        let base_url = format!("http://{}", addr);
        let page1 = links.get(&format!("{}/page1", base_url)).unwrap();
        assert_eq!(page1.found_in.len(), 2);
        for path in [
            "/", "/page1", "/page2", "/page3", "/page4", "/page5", "/page6",
        ] {
            assert!(
                links.get(&format!("{}{}", base_url, path)).is_some(),
                "path: {} not found",
                path
            );
        }

        for result in results.page_results {
            // println!("result: {:#?}", result);
            if result.link_type == LinkType::Internal {
                assert!(result.result.is_some());
                assert!(result.result.unwrap().results.len() > 0);
            }
        }
        assert!(results.site_result.len() > 0);
        assert!(
            results
                .site_result
                .iter()
                .any(|result| result.rule_id == "meta_description_uniqueness")
        );
        let meta_description_uniqueness = results
            .site_result
            .iter()
            .find(|result| result.rule_id == "meta_description_uniqueness")
            .unwrap();
        assert!(!meta_description_uniqueness.passed);
        let orphaned_page = results
            .site_result
            .iter()
            .find(|result| result.rule_id == "orphaned_page.check")
            .unwrap();
        assert!(!orphaned_page.passed);
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
