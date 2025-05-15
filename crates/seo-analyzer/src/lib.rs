// mod lighthouse;

pub use seo_plugins::{
    site_analyzer::{AnalysisProgress, AnalysisProgressType, CrawlResult, SiteAnalyzer},
    utils::{crawl_config::CrawlConfig, page::Page},
};
use thiserror::Error;

// pub use lighthouse::{run_lighthouse_analysis, CommandOutput, LighthouseMetrics, ShellCommand};

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

pub async fn crawl_url(
    config: &CrawlConfig,
    progress_callback: Box<dyn Fn(AnalysisProgress) + Send + Sync + 'static>,
) -> Result<CrawlResult, SeoError> {
    let url = config
        .get_url()
        .map_err(|e| SeoError::UrlParseError(e.to_string()))?;
    let site = SiteAnalyzer::new_with_default(url);
    site.with_progress_callback(move |progress| {
        progress_callback(progress);
    })
    .await
    .crawl()
    .await
    .map_err(|e| SeoError::AnalysisError(e.to_string()))
}
