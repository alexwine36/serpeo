use std::any::Any;

use crate::utils::{
    config::{CheckResult, Rule, RuleCategory, Severity},
    page_plugin::SeoPlugin,
    registry::PluginRegistry,
};

// Title Plugin
pub struct TitlePlugin {}

impl Default for TitlePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl TitlePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl SeoPlugin for TitlePlugin {
    fn name(&self) -> &str {
        "Title"
    }
    fn description(&self) -> &str {
        "The title tag of a web page is meant to be an accurate and concise description of
 a page's content. It is critical to both user experience and SEO."
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
                id: "title.has_title",
                name: "Page has title tag",
                description: "Checks if the page has a proper title tag",
                default_severity: Severity::Critical,
                category: RuleCategory::SEO,
                check: |page| {
                    let page = page.clone();
                    let meta_tags = page.extract_meta_tags();
                    let has_title = meta_tags.title.is_some();

                    CheckResult {
                        rule_id: "title.has_title".to_string(),
                        passed: has_title,
                        message: if has_title {
                            "Page has a title tag"
                        } else {
                            "Page is missing a title tag"
                        }
                        .to_string(),
                    }
                },
            },
            Rule {
                id: "title.title_length",
                name: "Title length is less than 60 characters",
                description: "Checks if the title length is less than 60 characters",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                check: |page| {
                    let page = page.clone();
                    let meta_tags = page.extract_meta_tags();
                    let title = meta_tags.title.unwrap_or_default();
                    let title_length = title.len();
                    let passed = title_length < 60 && title_length > 0;
                    CheckResult {
                        rule_id: "title.title_length".to_string(),
                        passed,
                        message: format!("Title length is {} characters", title_length),
                    }
                },
            },
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
    async fn test_title_plugin_success() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/success", addr);

        let plugin = TitlePlugin::new();
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
    async fn test_title_plugin_failure() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/failure", addr);

        let plugin = TitlePlugin::new();
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
                                <html>
                                    <head>
                                        <title>Test Page</title>
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
