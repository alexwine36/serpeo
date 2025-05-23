use std::any::Any;

use crate::utils::{
    config::{CheckResult, Rule, RuleCategory, Severity},
    page_plugin::SeoPlugin,
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

const PLUGIN_NAME: &str = "Request";

impl SeoPlugin for RequestPlugin {
    fn name(&self) -> &str {
        PLUGIN_NAME
    }
    fn description(&self) -> &str {
        "Check the status of network requests"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn available_rules(&self) -> Vec<Rule> {
        vec![Rule {
            id: "request.redirects",
            name: "Redirects",
            plugin_name: PLUGIN_NAME,
            description: "Checks if the page has redirects",
            passed_message: "Page does not have redirects",
            failed_message: "Page has redirects",
            default_severity: Severity::Error,
            category: RuleCategory::Performance,
            check: |page| {
                let page = page.clone();
                let redirected = page.get_redirected();

                CheckResult {
                    rule_id: "request.redirects".to_string(),
                    passed: !redirected,
                    message: if redirected {
                        "Page has redirects".to_string()
                    } else {
                        "Page does not have redirects".to_string()
                    },
                }
            },
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{config::RuleConfig, page::Page, page_plugin::SeoPlugin};
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Response, Server, header};

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

    #[tokio::test]
    async fn test_request_plugin_failure() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/redirect", addr);

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
                            // Page with redirect
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
                            "/redirect" => Ok::<_, Infallible>(
                                Response::builder()
                                    .status(301)
                                    .header(header::LOCATION, "/success")
                                    .body(Body::from("Redirecting..."))
                                    .unwrap(),
                            ),
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
