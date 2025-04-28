// Core plugin traits
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::page::Page;
use super::registry::PluginRegistry;

#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct CheckResult {
    pub rule_id: String,
    pub passed: bool,
    pub message: String,
    // pub severity: Option<Severity>,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct RuleResult {
    pub rule_id: String,
    pub passed: bool,
    pub message: String,
    pub severity: Severity,
    pub category: RuleCategory,
}

// Severity level of an SEO issue
#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

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
                    passed: result.passed,
                    message: result.message,
                    severity: rule.default_severity.clone(),
                    category: rule.category.clone(),
                }
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub enum RuleCategory {
    Accessibility,
    Performance,
    BestPractices,
    SEO,
}

// Rule definition
#[derive(Clone)]
pub struct Rule {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub default_severity: Severity,
    pub check: fn(&Page) -> CheckResult,
    pub category: RuleCategory,
}

// Configuration for which rules to run
#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct RuleConfig {
    enabled_rules: HashMap<String, bool>,
    rule_severities: HashMap<String, Severity>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleConfig {
    pub fn new() -> Self {
        Self {
            enabled_rules: HashMap::new(),
            rule_severities: HashMap::new(),
        }
    }

    pub fn enable_rule(&mut self, rule_id: &str) {
        self.enabled_rules.insert(rule_id.to_string(), true);
    }

    pub fn disable_rule(&mut self, rule_id: &str) {
        self.enabled_rules.insert(rule_id.to_string(), false);
    }

    pub fn set_severity(&mut self, rule_id: &str, severity: Severity) {
        self.rule_severities.insert(rule_id.to_string(), severity);
    }

    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        *self.enabled_rules.get(rule_id).unwrap_or(&false)
    }

    pub fn get_severity(&self, rule_id: &str, default: Severity) -> Severity {
        self.rule_severities
            .get(rule_id)
            .cloned()
            .unwrap_or(default)
    }
}
