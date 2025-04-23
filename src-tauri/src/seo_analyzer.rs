use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use thiserror::Error;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaTags {
    title: String,
    description: String,
    keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Headings {
    h1: i32,
    h2: i32,
    h3: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Images {
    total: i32,
    with_alt: i32,
    without_alt: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    internal: i32,
    external: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Performance {
    load_time: String,
    mobile_responsive: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeoAnalysis {
    meta_tags: MetaTags,
    headings: Headings,
    images: Images,
    links: Links,
    performance: Performance,
}

#[derive(Error, Debug)]
pub enum SeoError {
    #[error("Failed to fetch URL: {0}")]
    FetchError(String),
    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),
    #[error("Failed to analyze content: {0}")]
    AnalysisError(String),
}

pub async fn analyze_url(url: &str) -> Result<SeoAnalysis, SeoError> {
    let start_time = Instant::now();

    // Validate and parse URL
    let parsed_url = Url::parse(url).map_err(|e| SeoError::UrlParseError(e.to_string()))?;

    // Fetch the webpage
    let response = reqwest::get(parsed_url.clone())
        .await
        .map_err(|e| SeoError::FetchError(e.to_string()))?;

    let html = response
        .text()
        .await
        .map_err(|e| SeoError::FetchError(e.to_string()))?;

    let document = Html::parse_document(&html);

    // Analyze meta tags
    let meta_tags = analyze_meta_tags(&document)?;

    // Analyze headings
    let headings = analyze_headings(&document)?;

    // Analyze images
    let images = analyze_images(&document)?;

    // Analyze links
    let links = analyze_links(&document, &parsed_url)?;

    // Calculate performance metrics
    let performance = Performance {
        load_time: format!("{:.2}s", start_time.elapsed().as_secs_f32()),
        mobile_responsive: check_mobile_responsive(&document),
    };

    Ok(SeoAnalysis {
        meta_tags,
        headings,
        images,
        links,
        performance,
    })
}

fn analyze_meta_tags(document: &Html) -> Result<MetaTags, SeoError> {
    let title = document
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(|el| el.inner_html())
        .unwrap_or_default();

    let description = document
        .select(&Selector::parse("meta[name='description']").unwrap())
        .next()
        .and_then(|el| el.value().attr("content"))
        .unwrap_or_default()
        .to_string();

    let keywords = document
        .select(&Selector::parse("meta[name='keywords']").unwrap())
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|k| k.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    Ok(MetaTags {
        title,
        description,
        keywords,
    })
}

fn analyze_headings(document: &Html) -> Result<Headings, SeoError> {
    Ok(Headings {
        h1: count_elements(document, "h1"),
        h2: count_elements(document, "h2"),
        h3: count_elements(document, "h3"),
    })
}

fn analyze_images(document: &Html) -> Result<Images, SeoError> {
    let img_selector = Selector::parse("img").unwrap();
    let images = document.select(&img_selector);

    let mut total = 0;
    let mut with_alt = 0;
    let mut without_alt = 0;

    for img in images {
        total += 1;
        if img.value().attr("alt").is_some() {
            with_alt += 1;
        } else {
            without_alt += 1;
        }
    }

    Ok(Images {
        total,
        with_alt,
        without_alt,
    })
}

fn analyze_links(document: &Html, base_url: &Url) -> Result<Links, SeoError> {
    let link_selector = Selector::parse("a[href]").unwrap();
    let mut internal = 0;
    let mut external = 0;

    for link in document.select(&link_selector) {
        if let Some(href) = link.value().attr("href") {
            match Url::parse(href) {
                Ok(url) => {
                    if url.domain() == base_url.domain() {
                        internal += 1;
                    } else {
                        external += 1;
                    }
                }
                Err(_) => {
                    // Relative URLs are internal
                    internal += 1;
                }
            }
        }
    }

    Ok(Links { internal, external })
}

fn check_mobile_responsive(document: &Html) -> bool {
    // Check for viewport meta tag
    let viewport_selector = Selector::parse("meta[name='viewport']").unwrap();
    document.select(&viewport_selector).next().is_some()
}

fn count_elements(document: &Html, selector: &str) -> i32 {
    document.select(&Selector::parse(selector).unwrap()).count() as i32
}
