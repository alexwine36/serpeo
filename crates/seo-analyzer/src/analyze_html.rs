use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct MetaTags {
    title: String,
    description: String,
    keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct Headings {
    h1: i32,
    h2: i32,
    h3: i32,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct Images {
    total: i32,
    with_alt: i32,
    without_alt: i32,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct Links {
    internal: i32,
    external: i32,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct Performance {
    pub load_time: String,
    pub mobile_responsive: bool,
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

pub fn analyze_html_content(
    html: &str,
    base_url: &Url,
) -> Result<(MetaTags, Headings, Images, Links, bool), SeoError> {
    let document = Html::parse_document(html);

    let meta_tags = analyze_meta_tags(&document)?;

    let headings = Headings {
        h1: count_elements(&document, "h1"),
        h2: count_elements(&document, "h2"),
        h3: count_elements(&document, "h3"),
    };

    let (total_images, with_alt, without_alt) = analyze_image_stats(&document);
    let images = Images {
        total: total_images,
        with_alt,
        without_alt,
    };

    let (internal_links, external_links) = analyze_link_stats(&document, base_url);
    let links = Links {
        internal: internal_links,
        external: external_links,
    };

    let is_mobile_responsive = check_mobile_responsive(&document);

    Ok((meta_tags, headings, images, links, is_mobile_responsive))
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

pub fn analyze_headings(document: &Html) -> Result<Headings, SeoError> {
    Ok(Headings {
        h1: count_elements(document, "h1"),
        h2: count_elements(document, "h2"),
        h3: count_elements(document, "h3"),
    })
}

pub fn analyze_images(document: &Html) -> Result<Images, SeoError> {
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

pub fn analyze_links(document: &Html, base_url: &Url) -> Result<Links, SeoError> {
    let link_selector = Selector::parse("a[href]").unwrap();
    let mut internal = 0;
    let mut external = 0;

    for link in document.select(&link_selector) {
        if let Some(href) = link.value().attr("href") {
            println!("{}", href);
            match Url::parse(href) {
                Ok(url) => {
                    if url.domain() == base_url.domain() || href.starts_with("/") {
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

fn analyze_image_stats(document: &Html) -> (i32, i32, i32) {
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

    (total, with_alt, without_alt)
}

fn analyze_link_stats(document: &Html, base_url: &Url) -> (i32, i32) {
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

    (internal, external)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_meta_tags() {
        let html = r#"
            <html>
                <head>
                    <title>Test Title</title>
                    <meta name="description" content="Test Description">
                    <meta name="keywords" content="test, rust, seo">
                </head>
            </html>
        "#;
        let document = Html::parse_document(html);
        let meta_tags = analyze_meta_tags(&document).unwrap();

        assert_eq!(meta_tags.title, "Test Title");
        assert_eq!(meta_tags.description, "Test Description");
        assert_eq!(meta_tags.keywords, vec!["test", "rust", "seo"]);
    }

    #[test]
    fn test_analyze_headings() {
        let html = r#"
            <html>
                <body>
                    <h1>Heading 1</h1>
                    <h2>Heading 2</h2>
                    <h2>Another Heading 2</h2>
                    <h3>Heading 3</h3>
                </body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let headings = analyze_headings(&document).unwrap();

        assert_eq!(headings.h1, 1);
        assert_eq!(headings.h2, 2);
        assert_eq!(headings.h3, 1);
    }

    #[test]
    fn test_analyze_images() {
        let html = r#"
            <html>
                <body>
                    <img src="image1.jpg" alt="Image 1">
                    <img src="image2.jpg">
                    <img src="image3.jpg" alt="Image 3">
                </body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let images = analyze_images(&document).unwrap();

        assert_eq!(images.total, 3);
        assert_eq!(images.with_alt, 2);
        assert_eq!(images.without_alt, 1);
    }

    #[test]
    fn test_analyze_links() {
        let html = r#"
            <html>
                <body>
                    <a href="https://example.com">External Link</a>
                    <a href="/internal">Internal Link</a>
                    <a href="https://example.com/page">Another External Link</a>
                    <a href="https://cool-web.com/page">Another External Link</a>
                </body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let base_url = Url::parse("https://example.com").unwrap();
        let links = analyze_links(&document, &base_url).unwrap();

        assert_eq!(links.internal, 3);
        assert_eq!(links.external, 1);
    }

    #[test]
    fn test_check_mobile_responsive() {
        let html_with_viewport = r#"
            <html>
                <head>
                    <meta name="viewport" content="width=device-width, initial-scale=1">
                </head>
            </html>
        "#;
        let document_with_viewport = Html::parse_document(html_with_viewport);
        assert!(check_mobile_responsive(&document_with_viewport));

        let html_without_viewport = r#"
            <html>
                <head></head>
            </html>
        "#;
        let document_without_viewport = Html::parse_document(html_without_viewport);
        assert!(!check_mobile_responsive(&document_without_viewport));
    }
}
