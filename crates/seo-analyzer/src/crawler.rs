use futures::stream::{self, StreamExt};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use url::Url;

const MAX_CONCURRENT_REQUESTS: usize = 5;
const REQUEST_DELAY_MS: u64 = 100;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UrlSource {
    pub found_in_links: bool,
    pub found_in_sitemap: bool,
}

#[derive(Debug, Error)]
pub enum CrawlerError {
    #[error("Failed to fetch URL: {0}")]
    FetchError(String),
    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),
    #[error("Failed to parse HTML: {0}")]
    HtmlParseError(String),
    #[error("Failed to parse sitemap: {0}")]
    SitemapError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CrawlResult {
    pub urls: HashMap<String, UrlSource>,
    pub total_pages: u32,
}

pub struct Crawler {
    client: Client,
    visited_urls: Arc<Mutex<HashMap<String, UrlSource>>>,
    base_url: Url,
}

impl Crawler {
    fn normalize_url(url: &str) -> String {
        url.trim_end_matches('/').to_string()
    }

    pub fn new(base_url: &str) -> Result<Self, CrawlerError> {
        let base_url =
            Url::parse(base_url).map_err(|e| CrawlerError::UrlParseError(e.to_string()))?;

        Ok(Self {
            client: Client::new(),
            visited_urls: Arc::new(Mutex::new(HashMap::new())),
            base_url,
        })
    }

    pub async fn crawl(&self) -> Result<CrawlResult, CrawlerError> {
        // First try to fetch and parse sitemap
        let sitemap_urls = self.fetch_sitemap().await?;

        // Add sitemap URLs to visited_urls
        {
            let mut visited = self.visited_urls.lock().await;
            for url in &sitemap_urls {
                visited.insert(
                    Self::normalize_url(url),
                    UrlSource {
                        found_in_links: false,
                        found_in_sitemap: true,
                    },
                );
            }
        }

        // Start with the base URL
        let mut urls_to_crawl = vec![self.base_url.to_string()];
        let mut seen = HashSet::new();
        seen.insert(Self::normalize_url(self.base_url.as_ref()));

        while !urls_to_crawl.is_empty() {
            let batch: Vec<_> = urls_to_crawl
                .drain(..urls_to_crawl.len().min(MAX_CONCURRENT_REQUESTS))
                .collect();

            let results = stream::iter(batch)
                .map(|url| {
                    let client = self.client.clone();
                    let base_url = self.base_url.clone();
                    async move {
                        sleep(Duration::from_millis(REQUEST_DELAY_MS)).await;
                        self.process_page(&url, &client, &base_url).await
                    }
                })
                .buffer_unordered(MAX_CONCURRENT_REQUESTS)
                .collect::<Vec<_>>()
                .await;

            for result in results {
                if let Ok(new_urls) = result {
                    for url in new_urls {
                        let normalized_url = Self::normalize_url(&url);
                        if !seen.contains(&normalized_url) {
                            seen.insert(normalized_url);
                            urls_to_crawl.push(url);
                        }
                    }
                }
            }
        }

        let visited = self.visited_urls.lock().await;
        Ok(CrawlResult {
            urls: visited.clone(),
            total_pages: visited.len() as u32,
        })
    }

    async fn process_page(
        &self,
        url: &str,
        client: &Client,
        base_url: &Url,
    ) -> Result<Vec<String>, CrawlerError> {
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| CrawlerError::FetchError(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| CrawlerError::FetchError(e.to_string()))?;

        let base_url = base_url.clone();
        let html_clone = html.clone();

        let new_urls = tokio::task::spawn_blocking(move || {
            let document = Html::parse_document(&html_clone);
            let link_selector = Selector::parse("a[href]")
                .map_err(|e| CrawlerError::HtmlParseError(e.to_string()))?;

            let mut urls = Vec::new();
            for link in document.select(&link_selector) {
                if let Some(href) = link.value().attr("href") {
                    if let Ok(absolute_url) = base_url.join(href) {
                        if absolute_url.domain() == base_url.domain() {
                            urls.push(absolute_url.to_string());
                        }
                    }
                }
            }
            Ok::<_, CrawlerError>(urls)
        })
        .await
        .map_err(|e| CrawlerError::HtmlParseError(e.to_string()))??;

        let mut visited = self.visited_urls.lock().await;
        for url_string in &new_urls {
            let normalized_url = Self::normalize_url(url_string);
            let entry = visited.entry(normalized_url).or_insert(UrlSource {
                found_in_links: true,
                found_in_sitemap: false,
            });
            entry.found_in_links = true;
        }

        Ok(new_urls)
    }

    async fn discover_sitemap_url(&self) -> Result<Option<String>, CrawlerError> {
        let response = self
            .client
            .get(self.base_url.to_string())
            .send()
            .await
            .map_err(|e| CrawlerError::FetchError(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| CrawlerError::FetchError(e.to_string()))?;

        let html_clone = html.clone();
        let base_url = self.base_url.clone();

        let sitemap_url = tokio::task::spawn_blocking(move || {
            let document = Html::parse_document(&html_clone);
            let link_selector =
                Selector::parse("link[rel='sitemap'], link[type='application/xml+sitemap']")
                    .map_err(|e| CrawlerError::HtmlParseError(e.to_string()))?;

            for link in document.select(&link_selector) {
                if let Some(href) = link.value().attr("href") {
                    if let Ok(absolute_url) = base_url.join(href) {
                        return Ok(Some(absolute_url.to_string()));
                    }
                }
            }
            Ok(None)
        })
        .await
        .map_err(|e| CrawlerError::HtmlParseError(e.to_string()))??;

        Ok(sitemap_url)
    }

    async fn parse_sitemap_urls(&self, content: &str) -> Result<HashSet<String>, CrawlerError> {
        let content_clone = content.to_string();
        tokio::task::spawn_blocking(move || -> Result<HashSet<String>, CrawlerError> {
                let document = match roxmltree::Document::parse(&content_clone) {
                    Ok(doc) => doc,
                    Err(_) => return Ok(HashSet::new()), // Return empty set on parse failure
                };

                let mut urls = HashSet::new();
                for node in document.descendants() {
                    if node.has_tag_name("loc") {
                        if let Some(url) = node.text() {
                            urls.insert(Self::normalize_url(url));
                        }
                    }
                }
                Ok(urls)
            })
            .await
            .map_err(|e| CrawlerError::SitemapError(e.to_string()))?
    }

    async fn fetch_sitemap(&self) -> Result<HashSet<String>, CrawlerError> {
        let mut all_urls = HashSet::new();

        // Try to find sitemap URL from HTML first
        let mut sitemap_urls = HashSet::new();
        if let Some(discovered_url) = self.discover_sitemap_url().await? {
            sitemap_urls.insert(Self::normalize_url(&discovered_url));
        } else {
            // Fallback to common sitemap locations
            for path in &["/sitemap.xml", "/sitemap_index.xml", "/sitemap/sitemap.xml"] {
                if let Ok(url) = self.base_url.join(path) {
                    sitemap_urls.insert(Self::normalize_url(url.as_ref()));
                }
            }
        }

        // Process each potential sitemap URL
        for sitemap_url in sitemap_urls {
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

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Response, Server};
    use std::convert::Infallible;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

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

    #[tokio::test]
    async fn test_crawler() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}", addr);

        let crawler = Crawler::new(&base_url).unwrap();
        let result = crawler.crawl().await.unwrap();

        // Should find pages from both links and sitemap
        assert!(result.urls.contains_key(&format!("{}/page1", base_url)));
        assert!(result.urls.contains_key(&format!("{}/page2", base_url)));
        assert!(result.urls.contains_key(&format!("{}/page3", base_url)));

        // Page1 should be found in both links and sitemap
        let page1_source = result.urls.get(&format!("{}/page1", base_url)).unwrap();
        assert!(page1_source.found_in_links);
        assert!(page1_source.found_in_sitemap);

        // Page2 should only be found in links
        let page2_source = result.urls.get(&format!("{}/page2", base_url)).unwrap();
        assert!(page2_source.found_in_links);
        assert!(!page2_source.found_in_sitemap);

        // Page3 should only be found in sitemap
        let page3_source = result.urls.get(&format!("{}/page3", base_url)).unwrap();
        assert!(!page3_source.found_in_links);
        assert!(page3_source.found_in_sitemap);

        // External URLs should not be included
        assert!(!result.urls.contains_key("https://external.com"));
    }
}
