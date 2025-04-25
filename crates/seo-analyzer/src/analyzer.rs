use futures::stream::{self, StreamExt};
use html_parser::page_parser::{PageAnalysis, PageParser, PageParserError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
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
    #[error("Page parser error: {0}")]
    PageParserError(#[from] PageParserError),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum AnalysisStatus {
    Pending,
    InProgress,
    Complete,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnalysisResult {
    analysis: PageAnalysis,
    status: AnalysisStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct AnalysisProgress {
    pub total_urls: u32,
    pub completed_urls: u32,
    pub results: HashMap<String, AnalysisResult>,
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
        let results = Arc::new(Mutex::new(HashMap::<String, AnalysisResult>::new()));
        let completed = Arc::new(AtomicU32::new(0));
        let callback = Arc::new(Mutex::new(progress_callback));

        // Initialize results with existing analyses and pending status for others
        for (url, source) in crawl_result.urls {
            if let Some(analysis) = source.analysis {
                results.lock().await.insert(
                    url.clone(),
                    AnalysisResult {
                        analysis,
                        status: AnalysisStatus::Complete,
                    },
                );
                completed.fetch_add(1, Ordering::Relaxed);
            } else {
                let parser = PageParser::new(&url)?;
                let analysis = parser.analyze_page().await?;
                results.lock().await.insert(
                    url.clone(),
                    AnalysisResult {
                        analysis,
                        status: AnalysisStatus::Pending,
                    },
                );
            }
        }

        // Report initial state
        callback.lock().await(AnalysisProgress {
            total_urls,
            completed_urls: completed.load(Ordering::Relaxed),
            results: results.lock().await.clone(),
        });

        // Only analyze URLs that don't have analysis yet
        let urls_to_analyze: Vec<String> = results
            .lock()
            .await
            .iter()
            .filter(|(_, res)| matches!(res.status, AnalysisStatus::Pending))
            .map(|(url, _)| url.clone())
            .collect();

        let chunks: Vec<Vec<String>> = urls_to_analyze
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
                            if let Some(res) = results_lock.get_mut(&url_clone) {
                                res.status = AnalysisStatus::InProgress;
                            }
                            callback.lock().await(AnalysisProgress {
                                total_urls,
                                completed_urls: completed.load(Ordering::Relaxed),
                                results: results_lock.clone(),
                            });
                        }

                        match self.analyze_page(&url_clone, &client).await {
                            Ok(analysis) => {
                                Ok::<(String, PageAnalysis), AnalyzerError>((url_clone, analysis))
                            }
                            Err(e) => {
                                let parser = PageParser::new(&url_clone)?;
                                let analysis = parser.analyze_page().await?;
                                Ok::<(String, PageAnalysis), AnalyzerError>((url_clone, analysis))
                            }
                        }
                    }
                })
                .buffer_unordered(MAX_CONCURRENT_ANALYSES)
                .collect::<Vec<_>>()
                .await;

            for result in analyses {
                if let Ok((url, analysis)) = result {
                    let mut results_lock = results.lock().await;
                    if let Some(res) = results_lock.get_mut(&url) {
                        res.status = AnalysisStatus::Complete;
                    }
                    completed.fetch_add(1, Ordering::Relaxed);

                    callback.lock().await(AnalysisProgress {
                        total_urls,
                        completed_urls: completed.load(Ordering::Relaxed),
                        results: results_lock.clone(),
                    });
                }
            }
        }

        let results_guard = results.lock().await;
        Ok(results_guard
            .iter()
            .map(|(url, res)| (url.clone(), res.analysis.clone()))
            .collect())
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

        let mut parser = PageParser::new(url)?;
        parser.set_content(html);
        parser.analyze_page().await.map_err(|e| e.into())
    }
}
