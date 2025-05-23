use std::collections::{HashMap, HashSet};

use futures::stream::{self, StreamExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use tauri_specta::Event;
use thiserror::Error;

use url::Url;

use crate::utils::{
    config::{RuleResult, SiteCheckContext},
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
    AnalyzedSite(Vec<RuleResult>),
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
    #[error("Link not found: {0}")]
    LinkNotFound(String),
}

pub struct SiteAnalyzer {
    url: Url,
    links: Arc<RwLock<HashMap<String, PageLink>>>,
    registry: Arc<RwLock<PluginRegistry>>,
    progress_callback: Arc<RwLock<ProgressCallback>>,
}

impl SiteAnalyzer {
    pub fn new<T: FromUrl>(url: T, registry: PluginRegistry) -> Result<Self, SiteAnalyzerError> {
        let url = url.to_url().map_err(SiteAnalyzerError::UrlParseError)?;
        Ok(Self {
            url,
            links: Arc::new(RwLock::new(HashMap::new())),
            registry: Arc::new(RwLock::new(registry)),
            progress_callback: Arc::new(RwLock::new(None)),
        })
    }

    pub fn new_with_default<T: FromUrl>(url: T) -> Result<Self, SiteAnalyzerError> {
        let url = url.to_url().map_err(SiteAnalyzerError::UrlParseError)?;
        Ok(Self {
            url,
            links: Arc::new(RwLock::new(HashMap::new())),
            registry: Arc::new(RwLock::new(PluginRegistry::default_with_config())),
            progress_callback: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn with_progress_callback(
        &self,
        callback: impl Fn(AnalysisProgress) + Send + Sync + 'static,
    ) -> &Self {
        let _ = self.progress_callback.write().insert(Box::new(callback));
        self
    }

    pub fn get_links(&self) -> HashMap<String, PageLink> {
        self.links.read().clone()
    }

    async fn report_progress(
        &self,
        progress_type: AnalysisProgressType,
        url: Option<String>,
        // links: &RwLockReadGuard<'_, HashMap<String, PageLink>>,
    ) {
        if let Some(callback) = &self.progress_callback.read().as_ref() {
            callback(AnalysisProgress {
                progress_type,
                url,
                total_pages: self
                    .links
                    .read()
                    .values()
                    .filter(|link| link.link_type == LinkType::Internal)
                    .count() as u32,
                completed_pages: self
                    .links
                    .read()
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

    async fn record_site_results(
        &self,
        site_result: &[RuleResult],
    ) -> Result<(), SiteAnalyzerError> {
        for result in site_result {
            match &result.context {
                SiteCheckContext::Urls(urls) => {
                    for url in urls {
                        self.add_link(
                            url,
                            PageLinkSource {
                                link_source_type: LinkSourceType::Link,
                                url: self.url.to_string(),
                            },
                        )
                        .await?;
                        self.record_page_result(
                            &url.to_url().map_err(SiteAnalyzerError::UrlParseError)?,
                            PageResult {
                                error: false,
                                results: vec![result.clone()],
                            },
                        )
                        .await?;
                    }
                }
                SiteCheckContext::Values(values) => {}

                _ => {}
            }
        }
        Ok(())
    }

    async fn record_page_result(
        &self,
        url: &Url,
        result: PageResult,
    ) -> Result<(), SiteAnalyzerError> {
        // let links = self.links;
        // let mut links = self.links.write();
        {
            if let Some(link) = self.links.write().get_mut(&url.to_string()) {
                if link.result.is_none() {
                    link.result = Some(result);
                } else if !result.results.is_empty() {
                    link.result
                        .as_mut()
                        .expect("link.result is None")
                        .results
                        .extend(result.results);
                }
            }
        }
        {
            let link = self
                .links
                .read()
                .get(&url.to_string())
                .ok_or(SiteAnalyzerError::LinkNotFound(url.to_string()))?
                .clone();
            self.report_progress(
                AnalysisProgressType::AnalyzedPage(link.clone()),
                Some(url.to_string()),
                // &links,
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
        &self,
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

        if let Some(existing) = self.links.write().get_mut(&url_string) {
            existing.found_in.insert(page_link_source);
        } else {
            let mut found_in = HashSet::new();
            found_in.insert(page_link_source);
            self.links.write().insert(
                url_string,
                PageLink {
                    url: url_string2,
                    link_type: link.link_type,
                    found_in,
                    result: None,
                },
            );
            println!("links length: {}", self.links.read().len());
            self.report_progress(AnalysisProgressType::FoundLink, Some(url_string3))
                .await;
        }
        Ok(())
    }

    async fn process_page(&self, url: Url) -> Result<(), SiteAnalyzerError> {
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
            let registry = self.registry.read().clone();
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

    pub async fn crawl(&self) -> Result<CrawlResult, SiteAnalyzerError> {
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
                self.url.as_ref(),
                PageLinkSource {
                    link_source_type: LinkSourceType::Root,
                    url: self.url.to_string(),
                },
            )
            .await?;
        }

        loop {
            // Get all unprocessed internal links
            let internal_links: Vec<String> = self
                .links
                .read()
                .iter()
                .filter(|(_, link)| link.link_type == LinkType::Internal && link.result.is_none())
                .map(|(url, _)| url.clone())
                .collect();
            // drop(links);

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
                    async move { self.process_page(url).await }
                })
                .buffer_unordered(10);

            while let Some(result) = stream.next().await {
                result?;
            }
        }

        let registry = self.registry.read().clone();

        let site_result = registry.analyze_site(self).await?;

        self.record_site_results(&site_result).await?;

        let links = self.links.read();
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
    use std::sync::Mutex as StdMutex;
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
    async fn test_site_crawl_with_progress_callback() {
        let addr = start_weird_site_server().await;
        let base_url = format!("http://{}", addr);
        let site = SiteAnalyzer::new_with_default(base_url).unwrap();
        site.crawl().await.unwrap();
        let links = site.links.read();
        let error_links = links
            .values()
            .filter(|link| link.result.is_some() && link.result.as_ref().unwrap().error)
            .collect::<Vec<_>>();
        println!("error_links: {:#?}", error_links);
        assert!(error_links.is_empty());
        assert!(error_links.len() < 10);
        assert!(error_links.len() < links.len());
        assert!(error_links.len() < links.len() / 2);
        assert!(error_links.len() < links.len() / 3);
    }

    #[tokio::test]
    async fn test_site_crawl_with_progress() {
        let addr = start_weird_site_server().await;
        let base_url = format!("http://{}", addr);
        let local_results: Arc<StdMutex<HashMap<String, PageLink>>> =
            Arc::new(StdMutex::new(HashMap::new()));
        let local_results_clone = local_results.clone();
        let site = SiteAnalyzer::new_with_default(base_url).unwrap();
        let site = site
            .with_progress_callback(move |progress| {
                let url = progress.url.clone().unwrap();
                let url_clone = url.clone();
                match progress.progress_type {
                    AnalysisProgressType::FoundLink => {
                        local_results_clone.lock().unwrap().insert(
                            url_clone,
                            PageLink {
                                url: url.to_string(),
                                link_type: LinkType::Internal,
                                found_in: HashSet::new(),
                                result: None,
                            },
                        );
                    }
                    AnalysisProgressType::AnalyzedPage(link) => {
                        local_results_clone
                            .lock()
                            .unwrap()
                            .insert(link.url.clone(), link);
                    }
                    _ => {}
                }
            })
            .await;
        let results = site.crawl().await.unwrap();
        let links = local_results.lock().unwrap();
        assert_eq!(links.len() as u32, results.total_pages);
        let base_url = format!("http://{}", addr);

        let test_paths = [
            "/post1",
            "/post2",
            "/post3",
            "/sitemap-page-1",
            "/sitemap-page-2",
            "/sitemap-page-3",
            "/category1",
            "/category2",
            "/category3",
        ];

        for path in test_paths {
            assert!(
                results
                    .page_results
                    .iter()
                    .any(|result| result.url == format!("{}{}", base_url, path)),
                "Results - path: {} not found",
                path
            );
            assert!(
                links.get(&format!("{}{}", base_url, path)).is_some(),
                "path: {} not found",
                path
            );
        }
    }

    #[tokio::test]
    async fn test_site_crawl() {
        let addr = start_server().await;
        let base_url = format!("http://{}", addr);
        let site = SiteAnalyzer::new_with_default(base_url).unwrap();
        let results = site.crawl().await.unwrap();
        let links = site.links.read();
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
                assert!(!result.result.unwrap().results.is_empty());
            }
        }
        assert!(!results.site_result.is_empty());
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
                                        <a href="/page1">Page 1</a>
                                        <a href="/page2">Page 2</a>

                                        
                                    </body>
                                </html>
                            "#
                                .to_string(),
                            ))),
                            "/page1" => Ok::<_, Infallible>(Response::new(Body::from(
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
                            "#
                                .to_string(),
                            ))),
                            "/page5" => Ok::<_, Infallible>(Response::new(Body::from(
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
                            "#
                                .to_string(),
                            ))),
                            "/page6" => Ok::<_, Infallible>(Response::new(Body::from(
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
                            "#
                                .to_string(),
                            ))),
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

    async fn start_weird_site_server() -> SocketAddr {
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
                                        <a href="/page1">Page 1</a>
                                        <a href="/page2">Page 2</a>

                                        
                                    </body>
                                </html>
                            "#
                                .to_string(),
                            ))),
                            "/page1" => Ok::<_, Infallible>(Response::new(Body::from(
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
                            "#
                                .to_string(),
                            ))),
                            "/page5" => Ok::<_, Infallible>(Response::new(Body::from(
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
                            "#
                                .to_string(),
                            ))),
                            "/page6" => Ok::<_, Infallible>(Response::new(Body::from(
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
                            "#
                                .to_string(),
                            ))),
                            "/sitemap_index.xml" => {
                                let sitemap = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
                                    <sitemap><loc>{}/post-sitemap.xml</loc></sitemap>
                                    <sitemap><loc>{}/page-sitemap.xml</loc></sitemap>
                                    <sitemap><loc>{}/category-sitemap.xml</loc></sitemap>
                                    
                                    </sitemapindex>"#,
                                    base_url, base_url, base_url
                                );
                                Ok(Response::new(Body::from(sitemap)))
                            }
                            "/post-sitemap.xml" => {
                                let sitemap = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">
                                    <url><loc>{}/post1</loc></url>
                                    <url><loc>{}/post2</loc></url>
                                    <url><loc>{}/post3</loc></url>
                                    </urlset>"#,
                                    base_url, base_url, base_url
                                );
                                Ok(Response::new(Body::from(sitemap)))
                            }
                            "/page-sitemap.xml" => {
                                let sitemap = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">
                                    <url><loc>{}/sitemap-page-1</loc></url>
                                    <url><loc>{}/sitemap-page-2</loc></url>
                                    <url><loc>{}/sitemap-page-3</loc></url>
                                    </urlset>"#,
                                    base_url, base_url, base_url
                                );
                                Ok(Response::new(Body::from(sitemap)))
                            }
                            "/category-sitemap.xml" => {
                                let sitemap = format!(
                                    r#"<?xml version="1.0" encoding="UTF-8"?>
                                    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">
                                    <url><loc>{}/category1</loc></url>
                                    <url><loc>{}/category2</loc></url>
                                    <url><loc>{}/category3</loc></url>
                                    </urlset>"#,
                                    base_url, base_url, base_url
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
