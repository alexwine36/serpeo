mod analyze_html;
mod crawler;
mod lighthouse;
use html_parser::page_parser::MetaTagInfo;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use thiserror::Error;
use url::Url;

pub use analyze_html::{analyze_html_content, Headings, Images, Links, Performance};
pub use crawler::{CrawlResult, Crawler, CrawlerError, UrlSource};
pub use lighthouse::{run_lighthouse_analysis, CommandOutput, LighthouseMetrics, ShellCommand};
pub mod analyzer;

pub use analyzer::{AnalysisProgress, AnalysisStatus, Analyzer, AnalyzerError, PageAnalysis};

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct SeoAnalysis {
    meta_tags: MetaTagInfo,
    headings: Headings,
    images: Images,
    links: Links,
    performance: Performance,
    lighthouse_metrics: Option<LighthouseMetrics>,
}

#[derive(Error, Debug)]
pub enum SeoError {
    #[error("Failed to fetch URL: {0}")]
    FetchError(String),
    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),
    #[error("Failed to analyze content: {0}")]
    AnalysisError(String),
    #[error("Lighthouse error: {0}")]
    LighthouseError(String),
}

pub async fn analyze_url<S: ShellCommand>(shell: &S, url: String) -> Result<SeoAnalysis, SeoError> {
    let start_time = Instant::now();

    // Validate and parse URL
    let parsed_url = Url::parse(&url).map_err(|e| SeoError::UrlParseError(e.to_string()))?;

    // Fetch the webpage
    let response = reqwest::get(parsed_url.clone())
        .await
        .map_err(|e| SeoError::FetchError(e.to_string()))?;

    let html = response
        .text()
        .await
        .map_err(|e| SeoError::FetchError(e.to_string()))?;

    // Analyze HTML synchronously
    let (meta_tags, headings, images, links, is_mobile_responsive) =
        analyze_html_content(&html, &parsed_url)
            .map_err(|e| SeoError::AnalysisError(e.to_string()))?;

    let performance = Performance {
        load_time: format!("{:.2}s", start_time.elapsed().as_secs_f32()),
        mobile_responsive: is_mobile_responsive,
    };

    // Run lighthouse analysis
    let lighthouse_metrics = run_lighthouse_analysis(shell, url).await.ok();

    Ok(SeoAnalysis {
        meta_tags,
        headings,
        images,
        links,
        performance,
        lighthouse_metrics,
    })
}

pub async fn crawl_url(url: &str) -> Result<CrawlResult, SeoError> {
    let crawler = Crawler::new(url).map_err(|e| SeoError::UrlParseError(e.to_string()))?;
    crawler
        .crawl()
        .await
        .map_err(|e| SeoError::AnalysisError(e.to_string()))
}

// pub async fn analyze_crawl_result<F>(
//     url: &str,
//     crawl_result: CrawlResult,
//     progress_callback: F,
//     lighthouse_enabled: bool,
// ) -> Result<HashMap<String, PageAnalysis>, SeoError>
// where
//     F: FnMut(AnalysisProgress) + Send + Sync + 'static,
// {
//     let analyzer = Analyzer::new(url, lighthouse_enabled)
//         .map_err(|e| SeoError::UrlParseError(e.to_string()))?;
//     analyzer
//         .analyze_crawl_result(crawl_result, progress_callback)
//         .await
//         .map_err(|e| SeoError::AnalysisError(e.to_string()))
// }
