use std::any::Any;

use crate::utils::{
    config::{CheckResult, Rule, RuleCategory, Severity},
    page_plugin::SeoPlugin,
    registry::PluginRegistry,
};

// Request Plugin
pub struct RequestPlugin {}

impl Default for RequestPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl SeoPlugin for RequestPlugin {
    fn name(&self) -> &str {
        "Request"
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
                id: "request.size",
                name: "Page size",
                description: "Checks if the page size is within the recommended limits",
                default_severity: Severity::Error,
                category: RuleCategory::Performance,
                check: |page| {
                    let page = page.clone();
                    let content_length = page.get_content_length();
                    let is_within_limits = content_length.unwrap_or(0) <= 1000000;

                    CheckResult {
                        rule_id: "request.size".to_string(),
                        passed: is_within_limits,
                        message: if is_within_limits {
                            "Page size is within the recommended limits".to_string()
                        } else {
                            format!(
                                "Page size is too large: {} bytes",
                                content_length.unwrap_or(0)
                            )
                        },
                    }
                },
            },
            Rule {
                id: "request.redirects",
                name: "Redirects",
                description: "Checks if the page has redirects",
                default_severity: Severity::Error,
                category: RuleCategory::Performance,
                check: |page| {
                    let page = page.clone();
                    let redirected = page.get_redirected();
                    let is_redirected = redirected.unwrap_or(false);

                    CheckResult {
                        rule_id: "request.redirects".to_string(),
                        passed: !is_redirected,
                        message: if is_redirected {
                            "Page has redirects".to_string()
                        } else {
                            "Page does not have redirects".to_string()
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
    use crate::utils::{config::RuleConfig, page::Page, page_plugin::SeoPlugin};
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Response, Server};

    use std::convert::Infallible;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_request_plugin_success() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/success", addr);

        let plugin = RequestPlugin::new();
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

    // #[tokio::test]
    // async fn test_request_plugin_failure() {
    //     let addr = start_test_server().await;
    //     let base_url = format!("http://{}/failure", addr);

    //     let plugin = RequestPlugin::new();
    //     let page = Page::from_url(url::Url::parse(&base_url).unwrap())
    //         .await
    //         .unwrap();
    //     let mut config = RuleConfig::new();

    //     for rule in plugin.available_rules() {
    //         config.enable_rule(rule.id);
    //     }
    //     let results = plugin.analyze(&page, &config);
    //     for result in results {
    //         assert!(!result.passed, "Rule {} should have failed", result.rule_id);
    //     }
    // }

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
