use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use markup5ever::QualName;
use reqwest::Client;
use scraper::{
    ElementRef, Html, Selector,
    node::{Attributes, Element},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::Instant;
use url::Url;

use super::{
    page::{FromUrl, Page, PageError},
    sitemap_parser::SitemapParser,
};

#[derive(Debug, Error)]
pub enum SiteError {
    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),
    #[error("Page error: {0}")]
    PageError(#[from] PageError),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum LinkSourceType {
    Sitemap,
    Link,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct PageLinkSource {
    pub link_source_type: LinkSourceType,
    pub url: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageLink {
    pub url: Url,
    pub found_in: HashSet<PageLinkSource>,
}

pub struct Site {
    url: Url,
    links: Arc<Mutex<HashMap<String, PageLink>>>,
}

impl Site {
    pub fn new<T: FromUrl>(url: T) -> Result<Self, SiteError> {
        let url = url
            .to_url()
            .map_err(|e| SiteError::UrlParseError(e.to_string()))?;

        Ok(Self {
            url,
            links: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    async fn fetch_sitemap(&self) -> Result<HashSet<String>, SiteError> {
        let sitemap_parser = SitemapParser::new(self.url.clone()).unwrap();
        let sitemap_urls = sitemap_parser.get_sitemap().await.unwrap();
        Ok(sitemap_urls)
    }

    async fn add_link<T: FromUrl>(
        &mut self,
        url: T,
        page_link_source: PageLinkSource,
    ) -> Result<(), SiteError> {
        let url = url
            .to_url()
            .map_err(|e| SiteError::UrlParseError(e.to_string()))?;
        let mut found_in = HashSet::new();
        found_in.insert(page_link_source);
        self.links
            .lock()
            .await
            .insert(url.to_string(), PageLink { url, found_in });
        Ok(())
    }

    pub async fn crawl(&mut self) -> Result<(), SiteError> {
        let sitemap_urls = self.fetch_sitemap().await?;
        for sitemap_url in sitemap_urls {
            self.add_link(
                sitemap_url,
                PageLinkSource {
                    link_source_type: LinkSourceType::Sitemap,
                    url: self.url.join("/sitemap.xml").unwrap(),
                },
            )
            .await?;
        }
        Ok(())
    }

    async fn process_page(&mut self, url: Url) -> Result<(), SiteError> {
        let mut page = Page::from_url(url.clone())
            .await
            .map_err(|e| SiteError::PageError(e))?;

        let links = page.extract_links();

        for link in links {
            self.add_link(
                link.href,
                PageLinkSource {
                    link_source_type: LinkSourceType::Link,
                    url: url.clone(),
                },
            )
            .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_url_test() {
        let base_url = Url::parse("https://example.com/something").unwrap();
        let test_url = base_url.join("/test").unwrap();
        let test_url_2 = base_url.join("test/test").unwrap();
        assert_eq!(test_url.to_string(), "https://example.com/test");
        assert_eq!(test_url_2.to_string(), "https://example.com/test/test");
    }
}
