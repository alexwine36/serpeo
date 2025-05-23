use std::any::Any;

use crate::utils::{
    config::{CheckResult, Rule, RuleCategory, Severity},
    page_plugin::SeoPlugin,
};

// MetaDescription Plugin
pub struct MetaDescriptionPlugin {}

impl Default for MetaDescriptionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaDescriptionPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

const PLUGIN_NAME: &str = "MetaDescription";

impl SeoPlugin for MetaDescriptionPlugin {
    fn name(&self) -> &str {
        PLUGIN_NAME
    }
    fn description(&self) -> &str {
        "Meta descriptions provide concise explanations of the contents of web pages. They are commonly used on search engine result pages to display preview snippets for a given page."
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn available_rules(&self) -> Vec<Rule> {
        vec![
            Rule {
                id: "meta_description.has_meta_description",
                name: "Page has meta description",
                plugin_name: PLUGIN_NAME,
                description: "Checks if the page has a meta description",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                passed_message: "Page has a meta description",
                failed_message: "Page is missing a meta description",
                check: |page| {
                    let page = page.clone();
                    let meta_tags = page.extract_meta_tags();
                    let has_description = meta_tags.description.is_some();

                    CheckResult {
                        rule_id: "meta_description.has_meta_description".to_string(),
                        passed: has_description,
                        message: if has_description {
                            "Page has a meta description"
                        } else {
                            "Page is missing a meta description"
                        }
                        .to_string(),
                    }
                },
            },
            Rule {
                id: "meta_description.description_length",
                name: "Meta description length is less than 155 characters",
                plugin_name: PLUGIN_NAME,
                description: "Checks if the meta description length is less than 155 characters",
                default_severity: Severity::Warning,
                passed_message: "Meta description length is less than 155 characters",
                failed_message: "Meta description length is greater than 155 characters",
                category: RuleCategory::SEO,
                check: |page| {
                    let page = page.clone();
                    let meta_tags = page.extract_meta_tags();
                    let description = meta_tags.description.unwrap_or_default();
                    let description_length = description.len();
                    let passed = description_length < 155 && description_length > 0;
                    CheckResult {
                        rule_id: "meta_description.description_length".to_string(),
                        passed,
                        message: format!(
                            "Meta description length is {} characters",
                            description_length
                        ),
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
    async fn test_meta_description_plugin_success() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/success", addr);

        let plugin = MetaDescriptionPlugin::new();
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
    async fn test_meta_description_plugin_failure() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/failure", addr);

        let plugin = MetaDescriptionPlugin::new();
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
                                        <meta name="description" content="This is a test page with a meta description">
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
