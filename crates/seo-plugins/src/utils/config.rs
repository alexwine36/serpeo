// Core plugin traits
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::page::Page;
use super::registry::PluginRegistry;
use super::site::Site;

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
