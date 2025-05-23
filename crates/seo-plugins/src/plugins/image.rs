use std::any::{Any, TypeId};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::utils::config::{CheckResult, Rule, RuleCategory, Severity};
use crate::utils::page_plugin::SeoPlugin;
use crate::utils::registry::PluginRegistry;

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct ImageData {
    pub src: String,
    pub alt: Option<String>,
    pub srcset: Option<String>,
}

// Image Plugin
pub struct ImagePlugin {}

impl Default for ImagePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ImagePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

const PLUGIN_NAME: &str = "Images";

impl SeoPlugin for ImagePlugin {
    fn name(&self) -> &str {
        PLUGIN_NAME
    }
    fn description(&self) -> &str {
        "Image optimization analysis"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }

    fn available_rules(&self) -> Vec<Rule> {
        vec![
            Rule {
                id: "images.responsive",
                name: "Images are responsive",
                plugin_name: PLUGIN_NAME,
                description: "Checks if images use srcset for responsive design",
                passed_message: "All images use srcset",
                failed_message: "{} images missing srcset",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                check: |page| {
                    let page = page.clone();
                    let images = page.extract_images();
                    let images_without_srcset = images
                        .iter()
                        .filter(|img| img.srcset.is_none())
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "images.responsive".to_string(),
                        passed: images_without_srcset.is_empty(),
                        message: if images_without_srcset.is_empty() {
                            "All images use srcset".to_string()
                        } else {
                            format!("{} images missing srcset", images_without_srcset.len())
                        },
                    }
                },
            },
            // More rules...
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{config::RuleConfig, page::Page, page_plugin::SeoPlugin};
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Response, Server};

    use std::convert::Infallible;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_image_plugin_success() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/success", addr);

        let plugin = ImagePlugin::new();
        let page = Page::from_url(url::Url::parse(&base_url).unwrap())
            .await
            .unwrap();
        let mut config = RuleConfig::new();

        for rule in plugin.available_rules() {
            config.enable_rule(rule.id);
        }
        let results = plugin.analyze(&page, &config);
        for result in results {
            assert!(
                result.passed,
                "Rule {} should have passed, Message: {}",
                result.rule_id, result.message
            );
        }
    }

    #[tokio::test]
    async fn test_image_plugin_failure() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/failure", addr);

        let plugin = ImagePlugin::new();
        let page = Page::from_url(url::Url::parse(&base_url).unwrap())
            .await
            .unwrap();

        let mut config = RuleConfig::new();

        for rule in plugin.available_rules() {
            config.enable_rule(rule.id);
        }
        let results = plugin.analyze(&page, &config);

        for result in results {
            assert!(
                !result.passed,
                "Rule {} should have failed, Message: {}",
                result.rule_id, result.message
            );
        }
    }

    async fn start_test_server() -> SocketAddr {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = TcpListener::bind(addr).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);

        let make_svc = make_service_fn(move |_conn| {
            let base_url = base_url.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let base_url = base_url.clone();
                    async move {
                        match req.uri().path() {
                            "/success" => Ok::<_, Infallible>(Response::new(Body::from(format!(
                                r#"
                                <html>
                                    <head>
                                        <title>Test Page</title>
                                        <meta name="description" content="Test description">
                                        <link rel="canonical" href="{}/success">
                                    </head>
                                    <body>
                                        <img src="https://example.com/image.jpg" alt="Test image" srcset="https://example.com/image.jpg 1x, https://example.com/image@2x.jpg 2x">
                                        <a href="/page1">Page 1</a>
                                        <a href="/page2">Page 2</a>
                                        <a href="/page1?param=value">Page 1 with params</a>
                                        <a href="/page1#section">Page 1 with hash</a>
                                        <a href="/page1?param=value#section">Page 1 with both</a>
                                        <a href="https://external.com">External</a>
                                    </body>
                                </html>
                            "#,
                                base_url
                            )))),
                            "/failure" => Ok::<_, Infallible>(Response::new(Body::from(
                                r#"
                                <html>
                                    <head>

                                    </head>
                                    <body>
                                        <img src="https://example.com/image.jpg" >
                                    </body>
                                </html>
                            "#,
                            ))),
                            _ => Ok(Response::new(Body::from("404"))),
                        }
                    }
                }))
            }
        });

        tokio::spawn(async move {
            Server::from_tcp(listener.into_std().unwrap())
                .unwrap()
                .serve(make_svc)
                .await
                .unwrap();
        });

        addr
    }
}
