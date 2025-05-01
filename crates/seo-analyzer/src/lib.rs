pub mod crawler;
mod lighthouse;
use config::Config;
use html_parser::page_parser::{Heading, Image, Links, MetaTagInfo, Performance};

pub use seo_plugins::{
    site_analyzer::{AnalysisProgress, CrawlResult, SiteAnalyzer},
    utils::page::Page,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use thiserror::Error;

pub use crawler::{Crawler, CrawlerError, UrlSource};
pub use lighthouse::{run_lighthouse_analysis, CommandOutput, LighthouseMetrics, ShellCommand};
pub mod analyzer;
pub mod config;
pub use html_parser::page_parser::PageAnalysis;

pub use analyzer::{AnalysisStatus, Analyzer, AnalyzerError};

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct SeoAnalysis {
    results: CrawlResult,
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
    let url2 = url.clone();
    // Create and fetch page using PageParser
    let mut parser = html_parser::page_parser::PageParser::new(&url)
        .map_err(|e| SeoError::UrlParseError(e.to_string()))?;
    // let mut registry = PluginRegistry::default();
    // let mut config = RuleConfig::new();
    // let rules = registry.get_available_rules();
    // for rule in rules {
    //     config.enable_rule(rule.id);
    // }
    // registry.set_config(config);
    let mut site = SiteAnalyzer::new_with_default(url);
    let results = site
        .crawl()
        .await
        .map_err(|e| SeoError::AnalysisError(e.to_string()))?;
    println!("results: {:#?}", results);
    let page = Page::from_url(&url2)
        .await
        .map_err(|e| SeoError::UrlParseError(e.to_string()))?;
    // let results = registry.analyze(&page);
    parser
        .fetch()
        .await
        .map_err(|e| SeoError::FetchError(e.to_string()))?;

    let page_analysis = parser
        .analyze_page()
        .await
        .map_err(|e| SeoError::AnalysisError(e.to_string()))?;

    let performance = Performance {
        load_time: format!("{:.2}s", start_time.elapsed().as_secs_f32()),
        mobile_responsive: page_analysis.meta_tags.viewport.is_some(), // This should be determined by viewport meta tag or other means
    };

    // Run lighthouse analysis
    let lighthouse_metrics = run_lighthouse_analysis(shell, url2).await.ok();

    Ok(SeoAnalysis {
        results,
        // results: site
        //     .links
        //     .lock()
        //     .await
        //     .values()
        //     .map(|link| link.clone())
        //     .collect(),
        meta_tags: page_analysis.meta_tags,
        headings: page_analysis.headings,
        images: page_analysis.images,
        links: page_analysis.links,
        performance,
        lighthouse_metrics,
    })
}

pub async fn crawl_url(config: &Config) -> Result<crate::crawler::CrawlResultOrig, SeoError> {
    let crawler = Crawler::new(config).map_err(|e| SeoError::UrlParseError(e.to_string()))?;
    crawler
        .crawl()
        .await
        .map_err(|e| SeoError::AnalysisError(e.to_string()))
}
