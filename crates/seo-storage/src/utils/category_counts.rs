use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    entities::{page_rule_result, plugin_rule},
    enums::plugin_rule_enums::DbRuleCategory,
};

pub fn get_category_counts(
    data: Vec<(plugin_rule::Model, Vec<page_rule_result::Model>)>,
) -> HashMap<DbRuleCategory, CategoryResult> {
    let mut category_counts: HashMap<DbRuleCategory, CategoryResult> = HashMap::new();

    for (rule, results) in data {
        let category = rule.category;
        let count = results.len();
        let passed = results.iter().filter(|r| r.passed).count() as i32;
        let failed = count as i32 - passed;
        let Some(existing) = category_counts.get_mut(&category) else {
            category_counts.insert(
                category,
                CategoryResult {
                    total: count as i32,
                    passed,
                    failed,
                },
            );
            continue;
        };
        existing.total += count as i32;
        existing.passed += results.iter().filter(|r| r.passed).count() as i32;
        existing.failed += results.iter().filter(|r| !r.passed).count() as i32;
    }

    category_counts
}

pub fn get_category_result_display(
    data: HashMap<DbRuleCategory, CategoryResult>,
) -> CategoryResultDisplay {
    let total = data.values().map(|r| r.total).sum();
    let passed = data.values().map(|r| r.passed).sum();
    let failed = data.values().map(|r| r.failed).sum();
    CategoryResultDisplay {
        data,
        total,
        passed,
        failed,
    }
}

#[derive(Debug, Serialize, Deserialize, specta::Type, Clone)]
pub struct CategoryResult {
    pub total: i32,
    pub passed: i32,
    pub failed: i32,
}

#[derive(Debug, Serialize, Deserialize, specta::Type, Clone)]
pub struct CategoryResultDisplay {
    pub data: HashMap<DbRuleCategory, CategoryResult>,
    pub total: i32,
    pub passed: i32,
    pub failed: i32,
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;

    fn create_test_data() -> Vec<(plugin_rule::Model, Vec<page_rule_result::Model>)> {
        let mut data = Vec::new();

        let categories = vec![
            DbRuleCategory::Accessibility,
            DbRuleCategory::Performance,
            DbRuleCategory::BestPractices,
            DbRuleCategory::SEO,
        ];

        for category in categories {
            match category {
                DbRuleCategory::Accessibility => {
                    let mut rule = Faker.fake::<plugin_rule::Model>();
                    rule.category = category;
                    let mut rule_result = Faker.fake::<page_rule_result::Model>();
                    rule_result.passed = true;
                    let results = vec![rule_result];
                    data.push((rule, results));
                }
                DbRuleCategory::Performance => {
                    for _ in 0..10 {
                        let mut rule = Faker.fake::<plugin_rule::Model>();
                        rule.category = DbRuleCategory::Performance;
                        for _ in 0..10 {
                            let mut rule_result = Faker.fake::<page_rule_result::Model>();
                            rule_result.passed = true;
                            let results = vec![rule_result];
                            data.push((rule.clone(), results));
                        }
                    }
                }
                DbRuleCategory::BestPractices => {
                    for _ in 0..10 {
                        let mut rule = Faker.fake::<plugin_rule::Model>();
                        rule.category = DbRuleCategory::BestPractices;
                        for i in 0..10 {
                            let mut rule_result = Faker.fake::<page_rule_result::Model>();
                            rule_result.passed = i % 2 == 0;
                            let results = vec![rule_result];
                            data.push((rule.clone(), results));
                        }
                    }
                }
                _ => {}
            }
        }
        data
    }

    #[test]
    fn test_get_category_counts() {
        let data = create_test_data();
        let result = get_category_counts(data);
        println!("result: {:?}", result);
        // assert_eq!(result.len(), 1);
        assert_eq!(result[&DbRuleCategory::Accessibility].total, 1);
        assert_eq!(result[&DbRuleCategory::Accessibility].passed, 1);
        assert_eq!(result[&DbRuleCategory::Accessibility].failed, 0);

        assert_eq!(result[&DbRuleCategory::Performance].total, 100);
        assert_eq!(result[&DbRuleCategory::Performance].passed, 100);
        assert_eq!(result[&DbRuleCategory::Performance].failed, 0);

        assert_eq!(result[&DbRuleCategory::BestPractices].total, 100);
        assert_eq!(result[&DbRuleCategory::BestPractices].passed, 50);
        assert_eq!(result[&DbRuleCategory::BestPractices].failed, 50);
    }
}
