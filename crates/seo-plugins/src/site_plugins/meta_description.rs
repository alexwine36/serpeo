use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};

// use hashbrown::HashMap;

use crate::site_analyzer::SiteAnalyzer;
use crate::utils::config::{SiteCheckContext, SiteCheckResult};
use crate::utils::{
    config::{CheckResult, RuleCategory, RuleResult, Severity, SiteRule},
    page::Page,
    registry::PluginRegistry,
    site_plugin::SitePlugin,
};

#[derive(Debug)]
struct PageDescription {
    pub url: String,
    pub description: String,
}

#[derive(Clone)]
pub struct MetaDescriptionSitePlugin {
    page_descriptions: Arc<StdMutex<HashMap<String, PageDescription>>>,
}

impl MetaDescriptionSitePlugin {
    pub fn new() -> Self {
        Self {
            page_descriptions: Arc::new(StdMutex::new(HashMap::new())),
        }
    }
}

impl SitePlugin for MetaDescriptionSitePlugin {
    fn name(&self) -> &str {
        "Meta Description Plugin"
    }

    fn description(&self) -> &str {
        "Checks if meta descriptions are unique across pages"
    }
    fn initialize(&mut self, _registry: &mut PluginRegistry) -> Result<(), String> {
        Ok(())
    }
    fn after_page_hook(
        &mut self,
        page: Arc<StdMutex<Page>>,
        _results: &Vec<RuleResult>,
    ) -> Result<(), String> {
        let page = page.lock().unwrap();

        let mut page_descriptions = self.page_descriptions.lock().unwrap();
        let url = page.get_url().to_string();
        let meta_tags = page.extract_meta_tags();

        let description = meta_tags.description.clone();

        if let Some(description) = description {
            page_descriptions.insert(url.clone(), PageDescription { url, description });
        }

        Ok(())
    }

    fn available_rules(&self) -> Vec<SiteRule> {
        vec![SiteRule {
            id: "meta_description_uniqueness",
            name: "Meta Description Uniqueness",
            description: "Checks if meta descriptions are unique 90% of the time",
            default_severity: Severity::Warning,
            category: RuleCategory::SEO,
        }]
    }
    fn check(&self, rule: &SiteRule, _site: &SiteAnalyzer) -> SiteCheckResult {
        match rule.id {
            "meta_description_uniqueness" => {
                let page_descriptions = self.page_descriptions.lock().unwrap();
                let mut found_descriptions = HashMap::new();
                for (_url, page_description) in page_descriptions.iter() {
                    found_descriptions
                        .entry(page_description.description.clone())
                        .or_insert(Vec::new())
                        .push(page_description.url.clone());
                }
                let total_pages = page_descriptions.len();
                let unique_descriptions = found_descriptions.len();
                // println!("unique_descriptions: {:#?}", found_descriptions);
                // println!("page_descriptions: {:#?}", page_descriptions);
                let percentage = (unique_descriptions as f64 / total_pages as f64) * 100.0;
                if percentage < 90.0 {
                    SiteCheckResult {
                        rule_id: rule.id.to_string(),
                        passed: false,
                        message: "Meta description is unique across pages".to_string(),
                        context: SiteCheckContext::Empty,
                    }
                } else {
                    SiteCheckResult {
                        rule_id: rule.id.to_string(),
                        passed: true,
                        message: "Meta description is unique across pages".to_string(),
                        context: SiteCheckContext::Values(found_descriptions),
                    }
                }
            }
            _ => SiteCheckResult {
                rule_id: rule.id.to_string(),
                passed: false,
                message: "Unknown rule".to_string(),
                context: SiteCheckContext::Empty,
            },
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
