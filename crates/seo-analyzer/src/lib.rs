mod analyze_html;
mod lighthouse;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use thiserror::Error;
use url::Url;

pub use analyze_html::{analyze_html_content, Headings, Images, Links, MetaTags, Performance};
pub use lighthouse::{run_lighthouse_analysis, CommandOutput, LighthouseMetrics, ShellCommand};

#[derive(Debug, Serialize, Deserialize)]
pub struct SeoAnalysis {
    meta_tags: MetaTags,
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

    // Run lighthouse analysis
    let lighthouse_metrics = run_lighthouse_analysis(shell, url).await.ok();

    let performance = Performance {
        load_time: format!("{:.2}s", start_time.elapsed().as_secs_f32()),
        mobile_responsive: is_mobile_responsive,
    };

    Ok(SeoAnalysis {
        meta_tags,
        headings,
        images,
        links,
        performance,
        lighthouse_metrics,
    })
}
