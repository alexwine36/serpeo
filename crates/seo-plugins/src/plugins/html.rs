use std::any::Any;

use crate::utils::{
    config::{CheckResult, Rule, RuleCategory, SeoPlugin, Severity},
    registry::PluginRegistry,
};

// HTML Plugin
pub struct HtmlPlugin {}

impl Default for HtmlPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl SeoPlugin for HtmlPlugin {
    fn name(&self) -> &str {
        "HTML"
    }
    fn description(&self) -> &str {
        "Basic HTML structure analysis"
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
                id: "html.title",
                name: "Page has title tag",
                description: "Checks if the page has a proper title tag",
                default_severity: Severity::Error,
                category: RuleCategory::SEO,
                check: |page| {
                    let mut page = page.clone();
                    let meta_tags = page.extract_meta_tags();
                    let has_title = meta_tags.title.is_some();

                    CheckResult {
                        rule_id: "html.title".to_string(),
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
                id: "html.title_length",
                name: "Title length is less than 60 characters",
                description: "Checks if the title length is less than 60 characters",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                check: |page| {
                    let mut page = page.clone();
                    let meta_tags = page.extract_meta_tags();
                    let title = meta_tags.title.unwrap_or_default();
                    let title_length = title.len();
                    let passed = title_length < 60;
                    CheckResult {
                        rule_id: "html.title_length".to_string(),
                        passed,
                        message: format!("Title length is {} characters", title_length),
                    }
                },
            },
            Rule {
                id: "html.meta_description",
                name: "Page has meta description",
                description: "Checks if the page has a meta description",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                check: |page| {
                    let mut page = page.clone();
                    let meta_tags = page.extract_meta_tags();
                    let has_description = meta_tags.description.is_some();

                    CheckResult {
                        rule_id: "html.meta_description".to_string(),
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
                id: "html.meta_description_length",
                name: "Meta description length is less than 155 characters",
                description: "Checks if the meta description length is less than 155 characters",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                check: |page| {
                    let mut page = page.clone();
                    let meta_tags = page.extract_meta_tags();
                    let description = meta_tags.description.unwrap_or_default();
                    let description_length = description.len();
                    let passed = description_length < 155;
                    CheckResult {
                        rule_id: "html.meta_description_length".to_string(),
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
    use crate::utils::config::RuleConfig;
    use crate::utils::page::Page;

    #[test]
    fn test_html_plugin() {
        let plugin = HtmlPlugin::new();
        let page = Page::from_html(
            "<html><head><title>Test Page</title></head><body><h1>Test Page</h1></body></html>"
                .to_string(),
        );
        let mut config = RuleConfig::new();

        for rule in plugin.available_rules() {
            config.enable_rule(rule.id);
        }
        let results = plugin.analyze(&page, &config);
        assert_eq!(results.len(), 4);
        assert_eq!(results[0].rule_id, "html.title");
        assert_eq!(results[0].passed, true);
        assert_eq!(results[1].rule_id, "html.title_length");
        assert_eq!(results[1].passed, true);
    }
}
