use futures::stream::{self, StreamExt};
use html_parser::page_parser::{MetaTagInfo, PageParser};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri_specta::Event;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use url::Url;

use crate::crawler::CrawlResult;

const MAX_CONCURRENT_ANALYSES: usize = 5;
const ANALYSIS_DELAY_MS: u64 = 200;

#[derive(Debug, Error)]
pub enum AnalyzerError {
    #[error("Failed to fetch URL: {0}")]
    FetchError(String),
    #[error("Failed to parse HTML: {0}")]
    HtmlParseError(String),
    #[error("Lighthouse analysis failed: {0}")]
    LighthouseError(String),
    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PageAnalysis {
    pub url: String,
    pub path: String,
    pub meta_tags: MetaTagInfo,
    pub h1_count: u32,
    pub image_alt_missing: u32,
    pub broken_links: Vec<String>,
    pub lighthouse_score: Option<f64>,
    pub status: AnalysisStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum AnalysisStatus {
    Pending,
    InProgress,
    Complete,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct AnalysisProgress {
    pub total_urls: u32,
    pub completed_urls: u32,
    pub results: HashMap<String, PageAnalysis>,
}

pub struct Analyzer {
    client: Client,
    base_url: Url,
    lighthouse_enabled: bool,
}

impl Analyzer {
    pub fn new(base_url: &str, lighthouse_enabled: bool) -> Result<Self, AnalyzerError> {
        let base_url =
            Url::parse(base_url).map_err(|e| AnalyzerError::UrlParseError(e.to_string()))?;

        Ok(Self {
            base_url,
            client: Client::new(),
            lighthouse_enabled,
        })
    }

    pub async fn analyze_crawl_result<F>(
        &self,
        crawl_result: CrawlResult,
        progress_callback: F,
    ) -> Result<HashMap<String, PageAnalysis>, AnalyzerError>
    where
        F: for<'a> FnMut(AnalysisProgress) + Send + Sync + 'static,
    {
        let total_urls = crawl_result.urls.len() as u32;
        let results = std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let completed = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let callback = Arc::new(Mutex::new(progress_callback));

        // Initialize results with pending status
        for url in crawl_result.urls.keys() {
            results.lock().await.insert(
                url.clone(),
                PageAnalysis {
                    url: url.clone(),
                    path: url.clone().replace(self.base_url.as_str(), ""),
                    meta_tags: MetaTagInfo::default(),
                    h1_count: 0,
                    image_alt_missing: 0,
                    broken_links: Vec::new(),
                    lighthouse_score: None,
                    status: AnalysisStatus::Pending,
                },
            );
        }

        // Report initial state
        callback.lock().await(AnalysisProgress {
            total_urls,
            completed_urls: 0,
            results: results.lock().await.clone(),
        });

        let urls: Vec<String> = crawl_result.urls.keys().cloned().collect();
        let chunks: Vec<Vec<String>> = urls
            .chunks(MAX_CONCURRENT_ANALYSES)
            .map(|chunk| chunk.to_vec())
            .collect();

        for chunk in chunks {
            let results = results.clone();
            let completed = completed.clone();
            let callback = callback.clone();

            let analyses = stream::iter(chunk)
                .map(|url| {
                    let client = self.client.clone();
                    let url_clone = url.clone();
                    let results = results.clone();
                    let completed = completed.clone();
                    let callback = callback.clone();

                    async move {
                        sleep(Duration::from_millis(ANALYSIS_DELAY_MS)).await;

                        // Update status to InProgress
                        {
                            let mut results_lock = results.lock().await;
                            if let Some(analysis) = results_lock.get_mut(&url_clone) {
                                analysis.status = AnalysisStatus::InProgress;
                            }
                            callback.lock().await(AnalysisProgress {
                                total_urls,
                                completed_urls: completed.load(Ordering::Relaxed),
                                results: results_lock.clone(),
                            });
                        }

                        match self.analyze_page(&url_clone, &client).await {
                            Ok(mut analysis) => {
                                if self.lighthouse_enabled {
                                    if let Ok(score) = self.run_lighthouse(&url_clone).await {
                                        analysis.lighthouse_score = Some(score);
                                    }
                                }
                                analysis.status = AnalysisStatus::Complete;
                                Ok::<(String, PageAnalysis), AnalyzerError>((url_clone, analysis))
                            }
                            Err(e) => {
                                let failed_analysis = PageAnalysis {
                                    url: url_clone.clone(),
                                    path: url.clone().replace(self.base_url.as_str(), ""),
                                    meta_tags: MetaTagInfo::default(),
                                    h1_count: 0,
                                    image_alt_missing: 0,
                                    broken_links: Vec::new(),
                                    lighthouse_score: None,
                                    status: AnalysisStatus::Failed(e.to_string()),
                                };
                                Ok::<(String, PageAnalysis), AnalyzerError>((
                                    url_clone,
                                    failed_analysis,
                                ))
                            }
                        }
                    }
                })
                .buffer_unordered(MAX_CONCURRENT_ANALYSES)
                .collect::<Vec<_>>()
                .await;

            for result in analyses {
                if let Ok((url, analysis)) = result {
                    results.lock().await.insert(url, analysis);
                    completed.fetch_add(1, Ordering::Relaxed);

                    callback.lock().await(AnalysisProgress {
                        total_urls,
                        completed_urls: completed.load(Ordering::Relaxed),
                        results: results.lock().await.clone(),
                    });
                }
            }
        }

        Ok(Arc::try_unwrap(results)
            .expect("Arc still has multiple owners")
            .into_inner())
    }
    async fn analyze_page(
        &self,
        url: &str,
        client: &Client,
    ) -> Result<PageAnalysis, AnalyzerError> {
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| AnalyzerError::FetchError(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| AnalyzerError::FetchError(e.to_string()))?;

        let html_clone = html.clone();
        let url_string = url.to_string();
        let url_string_2 = url.to_string();

        // Run HTML parsing in a blocking task
        let (meta_tags, h1_count, image_alt_missing, links) =
            tokio::task::spawn_blocking(move || {
                let mut parser = PageParser::new(url_string_2).unwrap();
                let document = Html::parse_document(&html_clone);
                parser.set_document(document.clone());
                let meta_tags = parser.extract_meta_tags();
                let h1_count = Self::count_h1_tags_sync(&document);
                let image_alt_missing = Self::count_missing_alt_tags_sync(&document);
                let links = document
                    .select(&Selector::parse("a[href]").unwrap())
                    .filter_map(|link| link.value().attr("href"))
                    .map(String::from)
                    .collect::<Vec<_>>();
                (meta_tags, h1_count, image_alt_missing, links)
            })
            .await
            .map_err(|e| AnalyzerError::HtmlParseError(e.to_string()))?;

        // Check broken links asynchronously
        let mut broken_links = Vec::new();
        for link in links {
            if link.starts_with("http") {
                match client.get(&link).send().await {
                    Ok(response) => {
                        if !response.status().is_success() {
                            broken_links.push(link);
                        }
                    }
                    Err(_) => {
                        broken_links.push(link);
                    }
                }
            }
        }

        Ok(PageAnalysis {
            url: url_string,
            path: url.replace(self.base_url.as_str(), ""),
            meta_tags,
            h1_count,
            image_alt_missing,
            broken_links,
            lighthouse_score: None,
            status: AnalysisStatus::Complete,
        })
    }

    // fn extract_meta_tags_sync(document: &Html) -> MetaTagInfo {
    //     let mut info = MetaTagInfo::default();

    //     // Title
    //     if let Some(title) = document.select(&Selector::parse("title").unwrap()).next() {
    //         info.title = title.text().collect::<String>().into();
    //     }

    //     // Meta tags
    //     let meta_selector = Selector::parse("meta").unwrap();
    //     for meta in document.select(&meta_selector) {
    //         match meta
    //             .value()
    //             .attr("name")
    //             .or_else(|| meta.value().attr("property"))
    //         {
    //             Some("description") => {
    //                 info.description = meta.value().attr("content").map(String::from);
    //             }
    //             Some("robots") => {
    //                 info.robots = meta.value().attr("content").map(String::from);
    //             }
    //             Some("og:title") => {
    //                 info.og_title = meta.value().attr("content").map(String::from);
    //             }
    //             Some("og:description") => {
    //                 info.og_description = meta.value().attr("content").map(String::from);
    //             }
    //             Some("og:image") => {
    //                 info.og_image = meta.value().attr("content").map(String::from);
    //             }
    //             Some("twitter:card") => {
    //                 info.twitter_card = meta.value().attr("content").map(String::from);
    //             }
    //             _ => {}
    //         }
    //     }

    //     // Canonical link
    //     if let Some(canonical) = document
    //         .select(&Selector::parse("link[rel='canonical']").unwrap())
    //         .next()
    //     {
    //         info.canonical = canonical.value().attr("href").map(String::from);
    //     }

    //     info
    // }

    fn count_h1_tags_sync(document: &Html) -> u32 {
        document.select(&Selector::parse("h1").unwrap()).count() as u32
    }

    fn count_missing_alt_tags_sync(document: &Html) -> u32 {
        document
            .select(&Selector::parse("img").unwrap())
            .filter(|img| img.value().attr("alt").is_none())
            .count() as u32
    }

    async fn run_lighthouse(&self, url: &str) -> Result<f64, AnalyzerError> {
        // This is a placeholder for actual Lighthouse integration
        // You would implement this based on your specific needs
        Ok(0.0)
    }
}
