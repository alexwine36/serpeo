// Core plugin traits
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;

use super::page::Page;

#[derive(Debug, Serialize, Deserialize, specta::Type, Clone)]
pub enum SiteCheckContext {
    Urls(Vec<String>),
    Values(HashMap<String, Vec<String>>),
    Empty,
}
#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct SiteCheckResult {
    pub rule_id: String,
    pub passed: bool,
    pub message: String,
    pub context: SiteCheckContext,
    // pub severity: Option<Severity>,
}

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
    pub name: String,
    pub plugin_name: String,
    pub passed: bool,
    pub message: String,
    pub severity: Severity,
    pub category: RuleCategory,
    pub context: SiteCheckContext,
}

// Severity level of an SEO issue
#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
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

// Rule definition
#[derive(Clone)]
pub struct SiteRule {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub default_severity: Severity,
    // pub check: fn(&Site) -> CheckResult,
    pub category: RuleCategory,
}
#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub enum RuleType {
    Page,
    Site,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct RuleDisplay {
    pub id: String,
    pub name: String,
    pub plugin_name: String,
    pub description: String,
    pub severity: Severity,
    pub category: RuleCategory,
    pub rule_type: RuleType,
}

impl Rule {
    pub fn to_display(&self) -> RuleDisplay {
        RuleDisplay::from(self)
    }
}

impl SiteRule {
    pub fn to_display(&self) -> RuleDisplay {
        RuleDisplay::from(self)
    }
}

impl From<Rule> for RuleDisplay {
    fn from(rule: Rule) -> Self {
        RuleDisplay {
            id: rule.id.to_string(),
            name: rule.name.to_string(),
            plugin_name: "".to_string(),
            description: rule.description.to_string(),
            severity: rule.default_severity,
            category: rule.category,
            rule_type: RuleType::Page,
        }
    }
}

impl From<&Rule> for RuleDisplay {
    fn from(rule: &Rule) -> Self {
        let rule = rule.clone();
        RuleDisplay::from(rule)
    }
}
impl From<SiteRule> for RuleDisplay {
    fn from(rule: SiteRule) -> Self {
        RuleDisplay {
            id: rule.id.to_string(),
            name: rule.name.to_string(),
            plugin_name: "".to_string(),
            description: rule.description.to_string(),
            severity: rule.default_severity,
            category: rule.category,
            rule_type: RuleType::Site,
        }
    }
}

impl From<&SiteRule> for RuleDisplay {
    fn from(rule: &SiteRule) -> Self {
        let rule = rule.clone();
        RuleDisplay::from(rule)
    }
}

impl FromIterator<SiteRule> for Vec<RuleDisplay> {
    fn from_iter<T: IntoIterator<Item = SiteRule>>(iter: T) -> Self {
        iter.into_iter().map(|rule| rule.to_display()).collect()
    }
}

impl FromIterator<Rule> for Vec<RuleDisplay> {
    fn from_iter<T: IntoIterator<Item = Rule>>(iter: T) -> Self {
        iter.into_iter().map(|rule| rule.to_display()).collect()
    }
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

    pub fn enable_rule<S: AsRef<str>>(&mut self, rule_id: S) {
        self.enabled_rules
            .insert(rule_id.as_ref().to_string(), true);
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
