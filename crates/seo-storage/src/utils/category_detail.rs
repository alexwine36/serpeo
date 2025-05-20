use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    entities::{page_rule_result, plugin_rule, site_page},
    enums::plugin_rule_enums::{DbRuleCategory, DbSeverity},
};

pub fn get_category_detail(
    res: Vec<(
        page_rule_result::Model,
        Option<site_page::Model>,
        Option<plugin_rule::Model>,
    )>,
) -> Result<CategoryDetailResponse, String> {
    let mut category_detail: HashMap<DbRuleCategory, Vec<FlatRuleResult>> = HashMap::new();
    // let mut res_vec: Vec<FlatRuleResult> = Vec::new();
    for (page_rule_result, site_page, plugin_rule) in res {
        let site_page = site_page.unwrap();
        let plugin_rule = plugin_rule.unwrap();
        let category = plugin_rule.category;
        let category_clone = category.clone();
        let message = if page_rule_result.passed {
            plugin_rule.passed_message
        } else {
            plugin_rule.failed_message
        };

        let flat_rule_result = FlatRuleResult {
            rule_id: page_rule_result.rule_id,
            name: plugin_rule.name,
            plugin_name: plugin_rule.plugin_name,
            passed: page_rule_result.passed,
            message,
            severity: plugin_rule.severity,
            category,
            page_url: site_page.url,
        };
        category_detail
            .entry(category_clone)
            .or_default()
            .push(flat_rule_result);
    }
    // Ok(res_vec)
    Ok(CategoryDetailResponse {
        data: category_detail,
    })
}

#[derive(Debug, Serialize, Deserialize, Clone, specta::Type)]
pub struct FlatRuleResult {
    pub rule_id: String,
    pub name: String,
    pub plugin_name: String,
    pub passed: bool,
    pub message: String,
    pub severity: DbSeverity,
    pub category: DbRuleCategory,
    pub page_url: String,
    // link_type: String,
    // found_in: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, specta::Type)]
pub struct CategoryDetailResponse {
    pub data: HashMap<DbRuleCategory, Vec<FlatRuleResult>>,
}

// Test for the category detail response
#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;

    fn get_fake_plugin_rule(category: DbRuleCategory) -> plugin_rule::Model {
        let mut rule = Faker.fake::<plugin_rule::Model>();
        rule.category = category;
        rule
    }

    fn get_test_data() -> Vec<(
        page_rule_result::Model,
        Option<site_page::Model>,
        Option<plugin_rule::Model>,
    )> {
        vec![
            (
                Faker.fake::<page_rule_result::Model>(),
                Some(Faker.fake::<site_page::Model>()),
                Some(get_fake_plugin_rule(DbRuleCategory::Accessibility)),
            ),
            (
                Faker.fake::<page_rule_result::Model>(),
                Some(Faker.fake::<site_page::Model>()),
                Some(get_fake_plugin_rule(DbRuleCategory::Performance)),
            ),
            (
                Faker.fake::<page_rule_result::Model>(),
                Some(Faker.fake::<site_page::Model>()),
                Some(get_fake_plugin_rule(DbRuleCategory::Performance)),
            ),
        ]
    }

    #[test]
    fn test_category_detail() {
        let res = get_test_data();
        let category_detail = get_category_detail(res);
        println!("{:?}", category_detail);
        assert!(category_detail.is_ok());
        let category_detail = category_detail.unwrap();
        assert_eq!(category_detail.data.len(), 2);
        assert_eq!(
            category_detail.data[&DbRuleCategory::Accessibility].len(),
            1
        );
        assert_eq!(category_detail.data[&DbRuleCategory::Performance].len(), 2);
    }
}
