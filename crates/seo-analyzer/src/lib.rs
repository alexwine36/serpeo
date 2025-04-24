mod crawler;
mod lighthouse;
use html_parser::page_parser::{Heading, Image, Links, MetaTagInfo, Performance};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use thiserror::Error;
use url::Url;

pub use crawler::{CrawlResult, Crawler, CrawlerError, UrlSource};
pub use lighthouse::{run_lighthouse_analysis, CommandOutput, LighthouseMetrics, ShellCommand};
pub mod analyzer;

pub use analyzer::{AnalysisProgress, AnalysisStatus, Analyzer, AnalyzerError, PageAnalysis};

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct SeoAnalysis {
    meta_tags: MetaTagInfo,
    headings: Vec<Heading>,
    images: Vec<Image>,
    links: Vec<Links>,
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

    // Create and fetch page using PageParser
    let mut parser = html_parser::page_parser::PageParser::new(&url)
        .map_err(|e| SeoError::UrlParseError(e.to_string()))?;
    parser
        .fetch()
        .await
        .map_err(|e| SeoError::FetchError(e.to_string()))?;

    // Extract all information using PageParser
    // let meta_tags = parser.extract_meta_tags();
    // let headings = parser.extract_headings();
    // let images = parser.extract_images();
    // let links = parser.extract_links();

    let page_analysis = parser
        .analyze_page()
        .await
        .map_err(|e| SeoError::AnalysisError(e.to_string()))?;

    let performance = Performance {
        load_time: format!("{:.2}s", start_time.elapsed().as_secs_f32()),
        mobile_responsive: page_analysis.meta_tags.viewport.is_some(), // This should be determined by viewport meta tag or other means
    };

    // Run lighthouse analysis
    let lighthouse_metrics = run_lighthouse_analysis(shell, url).await.ok();

    Ok(SeoAnalysis {
        meta_tags: page_analysis.meta_tags,
        headings: page_analysis.headings,
        images: page_analysis.images,
        links: page_analysis.links,
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
