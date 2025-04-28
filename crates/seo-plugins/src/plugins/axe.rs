use std::any::Any;

use crate::utils::{
    config::{CheckResult, Rule, RuleCategory, SeoPlugin, Severity},
    registry::PluginRegistry,
};

// Axe Plugin
pub struct AxePlugin {}

impl Default for AxePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl AxePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl SeoPlugin for AxePlugin {
    fn name(&self) -> &str {
        "Axe"
    }
    fn description(&self) -> &str {
        ""
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn initialize(&mut self, _registry: &PluginRegistry) -> Result<(), String> {
        Ok(())
    }

    fn available_rules(&self) -> Vec<Rule> {
        vec![
            Rule {
                id: "axe.html_has_lang",
                name: "HTML has lang attribute",
                description: "Ensures every HTML document has a lang attribute",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    // let document = page.get_document().unwrap();
                    let lang = page.get_element("html").unwrap();
                    let has_lang = lang.attr("lang").is_some();

                    CheckResult {
                        rule_id: "axe.html_has_lang".to_string(),
                        passed: has_lang,
                        message: if has_lang {
                            "HTML has lang attribute".to_string()
                        } else {
                            "HTML is missing lang attribute".to_string()
                        },
                    }
                },
            },
            Rule {
                id: "axe.alt_text",
                name: "Images have alt text (Axe)",
                description: "Ensures <img> elements have alternate text or a role of none or presentation",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let mut page = page.clone();
                    let images = page.extract_images();
                    let images_without_alt = images
                        .iter()
                        .filter(|img| img.alt.is_none())
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "axe.alt_text".to_string(),
                        passed: images_without_alt.is_empty(),
                        message: if images_without_alt.is_empty() {
                            "All images have alt text".to_string()
                        } else {
                            format!("{} images missing alt text", images_without_alt.len())
                        },
                    }
                },
            },
            Rule {
                id: "axe.meta_viewport",
                name: "Meta viewport allows zoom",
                description: "Ensures the meta viewport element does not disable text scaling and zooming",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let mut page = page.clone();
                    let meta_tags = page.extract_meta_tags();

                    let viewport = meta_tags.viewport.unwrap_or_default();
                    let disables_zoom = viewport.contains("user-scalable=no")
                        || viewport.contains("maximum-scale=1.0")
                        || viewport.contains("maximum-scale=1")
                        || viewport.eq("");

                    CheckResult {
                        rule_id: "axe.meta_viewport".to_string(),
                        passed: !disables_zoom,
                        message: if !disables_zoom {
                            "Meta viewport allows zooming".to_string()
                        } else {
                            "Meta viewport disables zooming".to_string()
                        },
                    }
                },
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{
        config::{RuleConfig, SeoPlugin},
        page::Page,
    };
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Response, Server};

    use std::convert::Infallible;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_axe_plugin_success() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/success", addr);

        let plugin = AxePlugin::new();
        let page = Page::from_url(url::Url::parse(&base_url).unwrap())
            .await
            .unwrap();
        let mut config = RuleConfig::new();

        for rule in plugin.available_rules() {
            config.enable_rule(rule.id);
        }
        let results = plugin.analyze(&page, &config);
        for result in results {
            println!("Rule: {}", result.rule_id);
            assert!(result.passed, "Rule {} should have passed", result.rule_id);
        }
    }

    #[tokio::test]
    async fn test_axe_plugin_failure() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/failure", addr);

        let plugin = AxePlugin::new();
        let page = Page::from_url(url::Url::parse(&base_url).unwrap())
            .await
            .unwrap();
        let mut config = RuleConfig::new();

        for rule in plugin.available_rules() {
            config.enable_rule(rule.id);
        }
        let results = plugin.analyze(&page, &config);
        for result in results {
            assert!(!result.passed, "Rule {} should have failed", result.rule_id);
        }
    }

    async fn start_test_server() -> SocketAddr {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = TcpListener::bind(addr).await.unwrap();
        let addr = listener.local_addr().unwrap();
        // let base_url = format!("http://{}", addr);

        let make_svc = make_service_fn(move |_conn| {
            // let base_url = base_url.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    // let base_url = base_url.clone();
                    async move {
                        match req.uri().path() {
                            "/success" => Ok::<_, Infallible>(Response::new(Body::from(
                                r#"
                                <html lang="en">
                                    <head>
                                        <title>Test Page</title>
                                        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                                    </head>
                                    <body>
                                        <a href="/page1">Page 1</a>
                                        <a href="/page2">Page 2</a>
                                        <a href="/page1?param=value">Page 1 with params</a>
                                        <a href="/page1#section">Page 1 with hash</a>
                                        <a href="/page1?param=value#section">Page 1 with both</a>
                                        <a href="https://external.com">External</a>
                                    </body>
                                </html>
                            "#,
                            ))),
                            "/failure" => Ok::<_, Infallible>(Response::new(Body::from(
                                r#"
                                <html>
                                    <head>

                                    </head>
                                    <body>
                                        <img src="test.jpg" />
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
