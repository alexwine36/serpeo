use html_parser::page_parser::{LinkType, MetaTagInfo, PageParser};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

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
) -> Result<(MetaTagInfo, Headings, Images, Links, bool), SeoError> {
    let document = Html::parse_document(html);
    let mut parser = PageParser::new(base_url.clone()).unwrap();
    parser.set_document(document);
    let meta_tags = parser.extract_meta_tags();

    let headings = parser.extract_headings();

    let headings = Headings {
        h1: headings.iter().filter(|h| h.tag == "h1").count() as i32,
        h2: headings.iter().filter(|h| h.tag == "h2").count() as i32,
        h3: headings.iter().filter(|h| h.tag == "h3").count() as i32,
    };
    // let (total_images, with_alt, without_alt) = analyze_image_stats(&document);
    // let images = Images {
    //     total: total_images,
    //     with_alt,
    //     without_alt,
    // };
    let images = parser.extract_images();
    let images = Images {
        total: images.iter().count() as i32,
        with_alt: images.iter().filter(|img| img.alt.is_some()).count() as i32,
        without_alt: images.iter().filter(|img| img.alt.is_none()).count() as i32,
    };
    let links = parser.extract_links();
    println!("Links: {:?}", links);
    let links = Links {
        internal: links
            .iter()
            .filter(|link| link.link_type == LinkType::Internal)
            .count() as i32,
        external: links
            .iter()
            .filter(|link| link.link_type == LinkType::External)
            .count() as i32,
    };

    let is_mobile_responsive = meta_tags.viewport.is_some();

    Ok((meta_tags, headings, images, links, is_mobile_responsive))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_html_content() {
        let html = r#"
            <html>
                <head>
                    <title>Test Title</title>
                    <meta name="description" content="Test Description">
                    <meta name="viewport" content="width=device-width, initial-scale=1">
                </head>
                <body>
                    <h1>Heading 1</h1>
                    <h2>Heading 2</h2>
                    <h2>Another Heading 2</h2>
                    <h3>Heading 3</h3>
                    <img src="image1.jpg" alt="Image 1">
                    <img src="image2.jpg">
                    <img src="image3.jpg" alt="Image 3">
                    <a href="https://google.com">External Link</a>
                    <a href="/internal">Internal Link</a>
                </body>
            </html>
        "#;

        let base_url = Url::parse("https://example.com").unwrap();
        let result = analyze_html_content(html, &base_url).unwrap();

        let (meta_tags, headings, images, links, is_mobile_responsive) = result;

        assert_eq!(meta_tags.title, Some("Test Title".to_string()));
        assert_eq!(meta_tags.description, Some("Test Description".to_string()));
        assert!(is_mobile_responsive);

        assert_eq!(headings.h1, 1);
        assert_eq!(headings.h2, 2);
        assert_eq!(headings.h3, 1);

        assert_eq!(images.total, 3);
        assert_eq!(images.with_alt, 2);
        assert_eq!(images.without_alt, 1);

        assert_eq!(links.internal, 1);
        assert_eq!(links.external, 1);
    }

    #[test]
    fn test_analyze_html_content_no_meta_tags() {
        let html = r#"
            <html>
                <head></head>
                <body>
                    <h1>Heading 1</h1>
                    <img src="image1.jpg">
                    <a href="/internal">Internal Link</a>
                </body>
            </html>
        "#;

        let base_url = Url::parse("https://example.com").unwrap();
        let result = analyze_html_content(html, &base_url).unwrap();

        let (meta_tags, headings, images, links, is_mobile_responsive) = result;

        assert_eq!(meta_tags.title, None);
        assert_eq!(meta_tags.description, None);
        assert!(!is_mobile_responsive);

        assert_eq!(headings.h1, 1);
        assert_eq!(headings.h2, 0);
        assert_eq!(headings.h3, 0);

        assert_eq!(images.total, 1);
        assert_eq!(images.with_alt, 0);
        assert_eq!(images.without_alt, 1);

        assert_eq!(links.internal, 1);
        assert_eq!(links.external, 0);
    }

    #[test]
    fn test_analyze_html_content_no_images_or_links() {
        let html = r#"
            <html>
                <head>
                    <meta name="viewport" content="width=device-width, initial-scale=1">
                </head>
                <body>
                    <h1>Heading 1</h1>
                    <h2>Heading 2</h2>
                </body>
            </html>
        "#;

        let base_url = Url::parse("https://example.com").unwrap();
        let result = analyze_html_content(html, &base_url).unwrap();

        let (meta_tags, headings, images, links, is_mobile_responsive) = result;

        assert!(is_mobile_responsive);

        assert_eq!(headings.h1, 1);
        assert_eq!(headings.h2, 1);
        assert_eq!(headings.h3, 0);

        assert_eq!(images.total, 0);
        assert_eq!(images.with_alt, 0);
        assert_eq!(images.without_alt, 0);

        assert_eq!(links.internal, 0);
        assert_eq!(links.external, 0);
    }

    #[test]
    fn test_analyze_html_content_invalid_url() {
        let html = r#"
            <html>
                <body>
                    <h1>Heading 1</h1>
                </body>
            </html>
        "#;

        let base_url = Url::parse("invalid-url");
        assert!(base_url.is_err());
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_analyze_meta_tags() {
//         let html = r#"
//             <html>
//                 <head>
//                     <title>Test Title</title>
//                     <meta name="description" content="Test Description">
//                     <meta name="keywords" content="test, rust, seo">
//                 </head>
//             </html>
//         "#;
//         let document = Html::parse_document(html);
//         let meta_tags = analyze_meta_tags(&document).unwrap();

//         assert_eq!(meta_tags.title, "Test Title");
//         assert_eq!(meta_tags.description, "Test Description");
//         assert_eq!(meta_tags.keywords, vec!["test", "rust", "seo"]);
//     }

//     #[test]
//     fn test_analyze_headings() {
//         let html = r#"
//             <html>
//                 <body>
//                     <h1>Heading 1</h1>
//                     <h2>Heading 2</h2>
//                     <h2>Another Heading 2</h2>
//                     <h3>Heading 3</h3>
//                 </body>
//             </html>
//         "#;
//         let document = Html::parse_document(html);
//         let headings = analyze_headings(&document).unwrap();

//         assert_eq!(headings.h1, 1);
//         assert_eq!(headings.h2, 2);
//         assert_eq!(headings.h3, 1);
//     }

//     #[test]
//     fn test_analyze_images() {
//         let html = r#"
//             <html>
//                 <body>
//                     <img src="image1.jpg" alt="Image 1">
//                     <img src="image2.jpg">
//                     <img src="image3.jpg" alt="Image 3">
//                 </body>
//             </html>
//         "#;
//         let document = Html::parse_document(html);
//         let images = analyze_images(&document).unwrap();

//         assert_eq!(images.total, 3);
//         assert_eq!(images.with_alt, 2);
//         assert_eq!(images.without_alt, 1);
//     }

//     #[test]
//     fn test_analyze_links() {
//         let html = r#"
//             <html>
//                 <body>
//                     <a href="https://example.com">External Link</a>
//                     <a href="/internal">Internal Link</a>
//                     <a href="https://example.com/page">Another External Link</a>
//                     <a href="https://cool-web.com/page">Another External Link</a>
//                 </body>
//             </html>
//         "#;
//         let document = Html::parse_document(html);
//         let base_url = Url::parse("https://example.com").unwrap();
//         let links = analyze_links(&document, &base_url).unwrap();

//         assert_eq!(links.internal, 3);
//         assert_eq!(links.external, 1);
//     }

//     #[test]
//     fn test_check_mobile_responsive() {
//         let html_with_viewport = r#"
//             <html>
//                 <head>
//                     <meta name="viewport" content="width=device-width, initial-scale=1">
//                 </head>
//             </html>
//         "#;
//         let document_with_viewport = Html::parse_document(html_with_viewport);
//         assert!(check_mobile_responsive(&document_with_viewport));

//         let html_without_viewport = r#"
//             <html>
//                 <head></head>
//             </html>
//         "#;
//         let document_without_viewport = Html::parse_document(html_without_viewport);
//         assert!(!check_mobile_responsive(&document_without_viewport));
//     }
// }
