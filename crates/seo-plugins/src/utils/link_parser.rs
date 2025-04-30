use std::{collections::HashMap, num::NonZeroU16, time::Duration};

use markup5ever::QualName;
use reqwest::Client;
use scraper::{
    ElementRef, Html, Selector,
    node::{Attributes, Element},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use tokio::time::Instant;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
pub enum LinkType {
    Internal,
    External,
    Mailto,
    Tel,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct Link {
    pub href: String,
    pub path: String,
    pub link_type: LinkType,
}

#[derive(Debug, Error)]
pub enum LinkParseError {
    #[error("Failed to parse link: {0}")]
    LinkParseError(String),
    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),
}

pub fn parse_link<W: FromUrl>(link: &str, base_url: W) -> Result<Link, LinkParseError> {
    let base_url = base_url
        .to_url()
        .map_err(|e| LinkParseError::LinkParseError(e.to_string()))?;
    let link = if link.starts_with("http") {
        Url::parse(link).unwrap()
    } else {
        base_url.join(link).unwrap_or_else(|_| base_url.clone())
    };

    let link_type = if link.host_str() == Some(base_url.host_str().unwrap()) {
        LinkType::Internal
    } else if link.scheme() == "mailto" {
        LinkType::Mailto
    } else if link.scheme() == "tel" {
        LinkType::Tel
    } else {
        LinkType::External
    };
    Ok(Link {
        href: link.to_string(),
        path: link.path().to_string(),
        link_type,
    })
}

pub trait FromUrl: Clone {
    fn to_url(self) -> Result<Url, LinkParseError>;
}

impl FromUrl for Url {
    fn to_url(self) -> Result<Url, LinkParseError> {
        Ok(self)
    }
}

impl FromUrl for String {
    fn to_url(self) -> Result<Url, LinkParseError> {
        Url::parse(&self).map_err(|e| LinkParseError::UrlParseError(e.to_string()))
    }
}

impl FromUrl for &String {
    fn to_url(self) -> Result<Url, LinkParseError> {
        Url::parse(self).map_err(|e| LinkParseError::UrlParseError(e.to_string()))
    }
}

impl FromUrl for &str {
    fn to_url(self) -> Result<Url, LinkParseError> {
        Url::parse(self).map_err(|e| LinkParseError::UrlParseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_link() {
        let link = parse_link("https://www.google.com", "https://www.google.com").unwrap();
        assert_eq!(link.href, "https://www.google.com/");
        assert_eq!(link.path, "/");
        assert_eq!(link.link_type, LinkType::Internal);

        let link = parse_link("/sample", "http://localhost:3000").unwrap();
        assert_eq!(link.href, "http://localhost:3000/sample");
        assert_eq!(link.path, "/sample");
        assert_eq!(link.link_type, LinkType::Internal);
    }

    #[test]
    fn test_parse_link_external() {
        let link = parse_link("https://www.google.com", "http://localhost:3000").unwrap();
        assert_eq!(link.href, "https://www.google.com/");
        assert_eq!(link.path, "/");
        assert_eq!(link.link_type, LinkType::External);
    }

    #[test]
    fn test_parse_link_mailto() {
        let link = parse_link("mailto:test@example.com", "http://localhost:3000").unwrap();
        assert_eq!(link.href, "mailto:test@example.com");

        assert_eq!(link.link_type, LinkType::Mailto);
    }

    #[test]
    fn test_parse_link_tel() {
        let link = parse_link("tel:1234567890", "http://localhost:3000").unwrap();
        assert_eq!(link.href, "tel:1234567890");
        assert_eq!(link.link_type, LinkType::Tel);
    }
}
