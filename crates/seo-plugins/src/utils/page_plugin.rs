use futures::stream::{self, StreamExt};
use std::any::{Any, TypeId};

use super::config::{Rule, RuleConfig, RuleResult};
use super::page::Page;
use super::registry::PluginRegistry;

// Main plugin trait
#[async_trait::async_trait]
pub trait SeoPlugin: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn as_any(&self) -> &dyn Any;

    // What other plugins this one depends on
    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }

    // Initialize the plugin with access to its dependencies
    fn initialize(&mut self, registry: &PluginRegistry) -> Result<(), String>;

    // Get available rules this plugin can check
    fn available_rules(&self) -> Vec<Rule>;

    async fn analyze_async(&self, page: &Page, config: &RuleConfig) -> Vec<RuleResult> {
        let available_rules = self.available_rules();
        let rules: Vec<&Rule> = available_rules
            .iter()
            .filter(|rule| config.is_rule_enabled(rule.id))
            .collect();

        let results = stream::iter(rules)
            .map(|rule| {
                let result = (rule.check)(page);
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

    // Run enabled rules on the given page
    fn analyze(&self, page: &Page, config: &RuleConfig) -> Vec<RuleResult> {
        self.available_rules()
            .iter()
            .filter(|rule| config.is_rule_enabled(rule.id))
            .map(|rule| {
                let result = (rule.check)(page);
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
}
