use std::any::Any;

use crate::utils::{
    config::{CheckResult, Rule, RuleCategory, SeoPlugin, Severity},
    registry::PluginRegistry,
};

// SeoBasic Plugin
pub struct SeoBasicPlugin {}

impl Default for SeoBasicPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl SeoBasicPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl SeoPlugin for SeoBasicPlugin {
    fn name(&self) -> &str {
        "SeoBasic"
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
                id: "seo_basic.title",
                name: "Page has title tag",
                description: "Checks if the page has a proper title tag",
                default_severity: Severity::Critical,
                category: RuleCategory::SEO,
                check: |page| {
                    let mut page = page.clone();
                    let meta_tags = page.extract_meta_tags();
                    let has_title = meta_tags.title.is_some();

                    CheckResult {
                        rule_id: "seo_basic.title".to_string(),
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
                id: "seo_basic.title_length",
                name: "Title length is less than 60 characters",
                description: "Checks if the title length is less than 60 characters",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                check: |page| {
                    let mut page = page.clone();
                    let meta_tags = page.extract_meta_tags();
                    let title = meta_tags.title.unwrap_or_default();
                    let title_length = title.len();
                    let passed = title_length < 60 && title_length > 0;
                    CheckResult {
                        rule_id: "seo_basic.title_length".to_string(),
                        passed,
                        message: format!("Title length is {} characters", title_length),
                    }
                },
            },
            Rule {
                id: "seo_basic.meta_description",
                name: "Page has meta description",
                description: "Checks if the page has a meta description",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                check: |page| {
                    let mut page = page.clone();
                    let meta_tags = page.extract_meta_tags();
                    let has_description = meta_tags.description.is_some();

                    CheckResult {
                        rule_id: "seo_basic.meta_description".to_string(),
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
                id: "seo_basic.meta_description_length",
                name: "Meta description length is less than 155 characters",
                description: "Checks if the meta description length is less than 155 characters",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                check: |page| {
                    let mut page = page.clone();
                    let meta_tags = page.extract_meta_tags();
                    let description = meta_tags.description.unwrap_or_default();
                    let description_length = description.len();
                    let passed = description_length < 155 && description_length > 0;
                    CheckResult {
                        rule_id: "seo_basic.meta_description_length".to_string(),
                        passed,
                        message: format!(
                            "Meta description length is {} characters",
                            description_length
                        ),
                    }
                },
            },
            Rule {
                id: "seo_basic.has_canonical_url",
                name: "Page has canonical url",
                description: "Checks if the page has a canonical url",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                check: |page| {
                    let mut page = page.clone();
                    let canonical_url = page.extract_meta_tags().canonical;
                    let has_canonical_url = canonical_url.is_some();
                    CheckResult {
                        rule_id: "seo_basic.has_canonical_url".to_string(),
                        passed: has_canonical_url,
                        message: if has_canonical_url {
                            "Page has a canonical url"
                        } else {
                            "Page is missing a canonical url"
                        }
                        .to_string(),
                    }
                },
            },
            Rule {
                id: "seo_basic.canonical_url_matches_site",
                name: "Canonical url matches site",
                description: "Checks if the canonical url matches the site",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                check: |page| {
                    let mut page = page.clone();
                    let canonical_url = page.extract_meta_tags().canonical;
                    let canonical_url_matches_site = canonical_url.is_some()
                        && canonical_url
                            .unwrap()
                            .starts_with(page.get_url().unwrap().as_str());
                    CheckResult {
                        rule_id: "seo_basic.canonical_url_matches_site".to_string(),
                        passed: canonical_url_matches_site,
                        message: if canonical_url_matches_site {
                            "Canonical url matches site"
                        } else {
                            "Canonical url does not match site"
                        }
                        .to_string(),
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
    async fn test_seo_basic_plugin_success() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/success", addr);

        let plugin = SeoBasicPlugin::new();
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
    async fn test_seo_basic_plugin_failure() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/failure", addr);

        let plugin = SeoBasicPlugin::new();
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
