use std::any::Any;
use std::sync::{Arc, Mutex as StdMutex};

use futures::stream::{self, StreamExt};

use super::config::{CheckResult, RuleConfig, RuleResult, SiteRule};
use super::page::Page;

use crate::site_analyzer::SiteAnalyzer;
#[async_trait::async_trait]
pub trait SitePlugin: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
    fn initialize(&mut self, registry: &mut super::registry::PluginRegistry) -> Result<(), String>;
    fn available_rules(&self) -> Vec<SiteRule>;
    fn after_page_hook(
        &mut self,
        _page: Arc<StdMutex<Page>>,
        _results: &Vec<RuleResult>,
    ) -> Result<(), String> {
        Ok(())
    }
    fn check(&self, rule: &SiteRule, site: &SiteAnalyzer) -> CheckResult;
    fn analyze(&self, site: &SiteAnalyzer, config: &RuleConfig) -> Vec<RuleResult> {
        self.available_rules()
            .iter()
            .filter(|rule| config.is_rule_enabled(rule.id))
            .map(|rule| {
                let result = self.check(rule, site);
                RuleResult {
                    rule_id: rule.id.to_string(),
                    name: rule.name.to_string(),
                    plugin_name: self.name().to_string(),
                    passed: result.passed,
                    message: result.message,
                    severity: rule.default_severity.clone(),
                    category: rule.category.clone(),
                }
            })
            .collect()
    }
    async fn async_analyze(&self, site: &SiteAnalyzer, config: &RuleConfig) -> Vec<RuleResult> {
        let available_rules = self.available_rules();
        let rules: Vec<&SiteRule> = available_rules
            .iter()
            .filter(|rule| config.is_rule_enabled(rule.id))
            .collect();

        let results = stream::iter(rules)
            .map(|rule| {
                let result = self.check(rule, site);
                RuleResult {
                    rule_id: rule.id.to_string(),
                    name: rule.name.to_string(),
                    plugin_name: self.name().to_string(),
                    passed: result.passed,
                    message: result.message,
                    severity: rule.default_severity.clone(),
                    category: rule.category.clone(),
                }
            })
            .collect::<Vec<_>>()
            .await;
        results
    }
}
