use std::any::Any;
use std::collections::HashMap;

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::utils::{
    config::{CheckResult, Rule, RuleCategory, RuleConfig, SeoPlugin, Severity},
    page::Page,
    registry::PluginRegistry,
};

// HTML Plugin
pub struct HtmlPlugin {
    // html_content: Option<String>,
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
                    let meta_tags = page.extract_meta_tags(&page.get_document().unwrap());
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
                        severity: Severity::Error,
                        details: None,
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
                    let meta_tags = page.extract_meta_tags(&page.get_document().unwrap());
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
                        severity: Severity::Warning,
                        details: None,
                    }
                },
            },
            // More rules...
        ]
    }
}
