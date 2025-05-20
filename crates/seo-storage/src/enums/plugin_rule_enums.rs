use sea_orm::entity::prelude::*;
use seo_plugins::utils::config::{RuleCategory, RuleType, Severity};

#[cfg(test)]
use fake::Dummy;

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    serde::Serialize,
    serde::Deserialize,
    specta::Type,
)]
#[cfg_attr(test, derive(Dummy))]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum DbSeverity {
    #[sea_orm(string_value = "Info")]
    Info,
    #[sea_orm(string_value = "Warning")]
    Warning,
    #[sea_orm(string_value = "Error")]
    Error,
    #[sea_orm(string_value = "Critical")]
    Critical,
}

impl From<Severity> for DbSeverity {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Info => DbSeverity::Info,
            Severity::Warning => DbSeverity::Warning,
            Severity::Error => DbSeverity::Error,
            Severity::Critical => DbSeverity::Critical,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    serde::Serialize,
    serde::Deserialize,
    specta::Type,
    Hash,
)]
#[cfg_attr(test, derive(Dummy))]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum DbRuleCategory {
    #[sea_orm(string_value = "Accessibility")]
    Accessibility,
    #[sea_orm(string_value = "Performance")]
    Performance,
    #[sea_orm(string_value = "BestPractices")]
    BestPractices,
    #[sea_orm(string_value = "SEO")]
    SEO,
}

impl From<RuleCategory> for DbRuleCategory {
    fn from(category: RuleCategory) -> Self {
        match category {
            RuleCategory::Accessibility => DbRuleCategory::Accessibility,
            RuleCategory::Performance => DbRuleCategory::Performance,
            RuleCategory::BestPractices => DbRuleCategory::BestPractices,
            RuleCategory::SEO => DbRuleCategory::SEO,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    serde::Serialize,
    serde::Deserialize,
    specta::Type,
)]
#[cfg_attr(test, derive(Dummy))]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum DbRuleType {
    #[sea_orm(string_value = "Page")]
    Page,
    #[sea_orm(string_value = "Site")]
    Site,
}

impl From<RuleType> for DbRuleType {
    fn from(rule_type: RuleType) -> Self {
        match rule_type {
            RuleType::Page => DbRuleType::Page,
            RuleType::Site => DbRuleType::Site,
        }
    }
}
