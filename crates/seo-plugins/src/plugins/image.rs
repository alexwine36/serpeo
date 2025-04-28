use std::any::{Any, TypeId};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::plugins::html::HtmlPlugin;
use crate::utils::config::{CheckResult, Rule, RuleCategory, RuleConfig, SeoPlugin, Severity};
use crate::utils::page::Page;
use crate::utils::registry::PluginRegistry;

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct ImageData {
    pub src: String,
    pub alt: Option<String>,
    pub srcset: Option<String>,
}

// Image Plugin
pub struct ImagePlugin {}

impl ImagePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl SeoPlugin for ImagePlugin {
    fn name(&self) -> &str {
        "Images"
    }
    fn description(&self) -> &str {
        "Image optimization analysis"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<HtmlPlugin>()]
    }

    fn initialize(&mut self, _registry: &PluginRegistry) -> Result<(), String> {
        Ok(())
    }

    fn available_rules(&self) -> Vec<Rule> {
        vec![
            Rule {
                id: "images.alt_text",
                name: "Images have alt text",
                description: "Checks if all images have alt text attributes",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                check: |page| {
                    let images = page.extract_images(&page.get_document().unwrap());
                    let images_without_alt = images
                        .iter()
                        .filter(|img| img.alt.is_none())
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "images.alt_text".to_string(),
                        passed: images_without_alt.is_empty(),
                        message: if images_without_alt.is_empty() {
                            "All images have alt text".to_string()
                        } else {
                            format!("{} images missing alt text", images_without_alt.len())
                        },
                        severity: Severity::Warning,
                        details: None,
                    }
                },
            },
            Rule {
                id: "images.responsive",
                name: "Images are responsive",
                description: "Checks if images use srcset for responsive design",
                default_severity: Severity::Warning,
                category: RuleCategory::SEO,
                check: |page| {
                    let images = page.extract_images(&page.get_document().unwrap());
                    let images_without_srcset = images
                        .iter()
                        .filter(|img| img.srcset.is_none())
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "images.responsive".to_string(),
                        passed: images_without_srcset.is_empty(),
                        message: if images_without_srcset.is_empty() {
                            "All images use srcset".to_string()
                        } else {
                            format!("{} images missing srcset", images_without_srcset.len())
                        },
                        severity: Severity::Warning,
                        details: None,
                    }
                },
            },
            // More rules...
        ]
    }

    // fn analyze(&self, page: &Page, config: &RuleConfig) -> Vec<CheckResult> {
    //     let mut results = Vec::new();
    //     let images = page.extract_images(&page.get_document().unwrap());
    //     // Check for alt text
    //     if config.is_rule_enabled("images.alt_text") {
    //         let images_without_alt = images
    //             .iter()
    //             .filter(|img| img.alt.is_none())
    //             .collect::<Vec<_>>();

    //         results.push(CheckResult {
    //             rule_id: "images.alt_text".to_string(),
    //             passed: images_without_alt.is_empty(),
    //             message: if images_without_alt.is_empty() {
    //                 "All images have alt text".to_string()
    //             } else {
    //                 format!("{} images missing alt text", images_without_alt.len())
    //             },
    //             severity: Severity::Warning,
    //             details: None,
    //         });
    //     }

    //     // More checks...

    //     results
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::page::Page;

    #[test]
    fn test_image_plugin() {
        let html = "<html><body><img src='test.jpg' alt='Test Image' /></body></html>";
        let plugin = ImagePlugin::new();
        let page = Page::from_html(html.to_string());
        let mut config = RuleConfig::new();
        config.enable_rule("images.alt_text");
        config.enable_rule("images.responsive");
        let results = plugin.analyze(&page, &config);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].rule_id, "images.alt_text");
        assert_eq!(results[0].passed, true);
        assert_eq!(results[1].rule_id, "images.responsive");
        assert_eq!(results[1].passed, false);
    }
}
