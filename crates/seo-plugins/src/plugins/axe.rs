use std::any::Any;

use crate::utils::{
    config::{CheckResult, Rule, RuleCategory, Severity},
    page_plugin::SeoPlugin,
    registry::PluginRegistry,
};
use scraper::Selector;

// Axe Plugin
pub struct AxePlugin {}

impl Default for AxePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl AxePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl SeoPlugin for AxePlugin {
    fn name(&self) -> &str {
        "Axe"
    }
    fn description(&self) -> &str {
        "Accessibility testing using Axe rules"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn initialize(&mut self, _registry: &PluginRegistry) -> Result<(), String> {
        Ok(())
    }

    fn available_rules(&self) -> Vec<Rule> {
        vec![
            Rule {
                id: "axe.html_has_lang",
                name: "HTML has lang attribute",
                description: "Ensures every HTML document has a lang attribute",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    let lang = page.get_element("html").unwrap();
                    let has_lang = lang.attr("lang").is_some();

                    CheckResult {
                        rule_id: "axe.html_has_lang".to_string(),
                        passed: has_lang,
                        message: if has_lang {
                            "HTML has lang attribute".to_string()
                        } else {
                            "HTML is missing lang attribute".to_string()
                        },
                    }
                },
            },
            Rule {
                id: "axe.image_alt",
                name: "Images have alt text",
                description: "Ensures <img> elements have alternate text or a role of none or presentation",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    let images = page.extract_images();
                    let images_without_alt = images
                        .iter()
                        .filter(|img| img.alt.is_none())
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "axe.image_alt".to_string(),
                        passed: images_without_alt.is_empty(),
                        message: if images_without_alt.is_empty() {
                            "All images have alt text".to_string()
                        } else {
                            format!("{} images missing alt text", images_without_alt.len())
                        },
                    }
                },
            },
            Rule {
                id: "axe.meta_viewport",
                name: "Meta viewport allows zoom",
                description: "Ensures the meta viewport element does not disable text scaling and zooming",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    let meta_tags = page.extract_meta_tags();

                    let viewport = meta_tags.viewport.unwrap_or_default();
                    let disables_zoom = viewport.contains("user-scalable=no")
                        || viewport.contains("maximum-scale=1.0")
                        || viewport.contains("maximum-scale=1")
                        || viewport.is_empty();

                    CheckResult {
                        rule_id: "axe.meta_viewport".to_string(),
                        passed: !disables_zoom,
                        message: if !disables_zoom {
                            "Meta viewport allows zooming".to_string()
                        } else {
                            "Meta viewport disables zooming".to_string()
                        },
                    }
                },
            },
            Rule {
                id: "axe.document_title",
                name: "Document has title",
                description: "Ensures the document has a title element",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    let has_title = page.get_element("title").is_ok();

                    CheckResult {
                        rule_id: "axe.document_title".to_string(),
                        passed: has_title,
                        message: if has_title {
                            "Document has title".to_string()
                        } else {
                            "Document is missing title".to_string()
                        },
                    }
                },
            },
            Rule {
                id: "axe.button_name",
                name: "Buttons have accessible name",
                description: "Ensures buttons have an accessible name",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    let selector = Selector::parse("button").unwrap();
                    let document = page.get_document().unwrap();
                    let buttons = document.select(&selector);
                    let buttons_without_name = buttons
                        .filter(|button| {
                            button.text().collect::<String>().trim().is_empty()
                                && button.attr("aria-label").is_none()
                                && button.attr("aria-labelledby").is_none()
                        })
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "axe.button_name".to_string(),
                        passed: buttons_without_name.is_empty(),
                        message: if buttons_without_name.is_empty() {
                            "All buttons have accessible names".to_string()
                        } else {
                            format!(
                                "{} buttons missing accessible names",
                                buttons_without_name.len()
                            )
                        },
                    }
                },
            },
            Rule {
                id: "axe.link_name",
                name: "Links have accessible name",
                description: "Ensures links have an accessible name",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    let selector = Selector::parse("a").unwrap();
                    let document = page.get_document().unwrap();
                    let links = document.select(&selector);
                    // let links_length = links.count();
                    let links_without_name = links
                        .filter(|link| {
                            link.text().collect::<String>().trim().is_empty()
                                && link.attr("aria-label").is_none()
                                && link.attr("aria-labelledby").is_none()
                        })
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "axe.link_name".to_string(),
                        passed: links_without_name.is_empty(),
                        message: if links_without_name.is_empty() {
                            "All links have accessible names".to_string()
                        } else {
                            format!(
                                "{} links missing accessible names",
                                links_without_name.len()
                            )
                        },
                    }
                },
            },
            Rule {
                id: "axe.form_field_labels",
                name: "Form fields have labels",
                description: "Ensures form fields have associated labels",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    let selector = Selector::parse("input, select, textarea").unwrap();
                    let document = page.get_document().unwrap();
                    let form_fields = document.select(&selector);
                    let fields_without_labels = form_fields
                        .filter(|field| {
                            let id = field.attr("id");
                            if let Some(id) = id {
                                page.get_element(&format!("label[for={}]", id)).is_ok()
                            } else {
                                true
                            }
                        })
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "axe.form_field_labels".to_string(),
                        passed: fields_without_labels.is_empty(),
                        message: if fields_without_labels.is_empty() {
                            "All form fields have labels".to_string()
                        } else {
                            format!("{} form fields missing labels", fields_without_labels.len())
                        },
                    }
                },
            },
            Rule {
                id: "axe.aria_valid_attr",
                name: "ARIA attributes are valid",
                description: "Ensures ARIA attributes are valid",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    let selector = Selector::parse("[aria-]").unwrap();
                    let document = page.get_document().unwrap();
                    let elements = document.select(&selector);
                    let invalid_attrs = elements
                        .filter_map(|element| {
                            let attrs = element.value().attrs.clone();
                            let invalid: Vec<_> = attrs
                                .iter()
                                .filter(|(name, _)| name.local.to_string().starts_with("aria-"))
                                .filter(|(name, _)| !is_valid_aria_attribute(name.local.as_ref()))
                                .cloned()
                                .collect();
                            if invalid.is_empty() {
                                None
                            } else {
                                Some(invalid)
                            }
                        })
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "axe.aria_valid_attr".to_string(),
                        passed: invalid_attrs.is_empty(),
                        message: if invalid_attrs.is_empty() {
                            "All ARIA attributes are valid".to_string()
                        } else {
                            format!(
                                "Found {} elements with invalid ARIA attributes",
                                invalid_attrs.len()
                            )
                        },
                    }
                },
            },
            Rule {
                id: "axe.aria_required_attr",
                name: "ARIA required attributes",
                description: "Ensures elements with ARIA roles have required attributes",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    let selector = Selector::parse("[role]").unwrap();
                    let document = page.get_document().unwrap();
                    let elements = document.select(&selector);
                    let missing_attrs = elements
                        .filter_map(|element| {
                            if let Some(role) = element.attr("role") {
                                let required_attrs = get_required_aria_attributes(role).to_vec();
                                let missing = required_attrs
                                    .iter()
                                    .filter(|attr| element.attr(attr).is_none())
                                    .cloned()
                                    .collect::<Vec<_>>();
                                if missing.is_empty() {
                                    None
                                } else {
                                    Some(missing)
                                }
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "axe.aria_required_attr".to_string(),
                        passed: missing_attrs.is_empty(),
                        message: if missing_attrs.is_empty() {
                            "All ARIA roles have required attributes".to_string()
                        } else {
                            format!(
                                "Found {} elements missing required ARIA attributes",
                                missing_attrs.len()
                            )
                        },
                    }
                },
            },
            Rule {
                id: "axe.duplicate_id",
                name: "No duplicate IDs",
                description: "Ensures IDs are unique",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    let selector = Selector::parse("[id]").unwrap();
                    let document = page.get_document().unwrap();
                    let elements = document.select(&selector);
                    let mut id_counts = std::collections::HashMap::new();
                    for element in elements {
                        if let Some(id) = element.attr("id") {
                            *id_counts.entry(id.to_string()).or_insert(0) += 1;
                        }
                    }
                    let duplicate_ids = id_counts
                        .iter()
                        .filter(|(_, count)| **count > 1)
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "axe.duplicate_id".to_string(),
                        passed: duplicate_ids.is_empty(),
                        message: if duplicate_ids.is_empty() {
                            "No duplicate IDs found".to_string()
                        } else {
                            format!("Found {} duplicate IDs", duplicate_ids.len())
                        },
                    }
                },
            },
            Rule {
                id: "axe.frame_title",
                name: "Frames have title",
                description: "Ensures frames have title attributes",
                default_severity: Severity::Error,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    let selector = Selector::parse("frame, iframe").unwrap();
                    let document = page.get_document().unwrap();
                    let frames = document.select(&selector);
                    let frames_without_title = frames
                        .filter(|frame| frame.attr("title").is_none())
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "axe.frame_title".to_string(),
                        passed: frames_without_title.is_empty(),
                        message: if frames_without_title.is_empty() {
                            "All frames have title attributes".to_string()
                        } else {
                            format!(
                                "{} frames missing title attributes",
                                frames_without_title.len()
                            )
                        },
                    }
                },
            },
            // Rule {
            //     id: "axe.skip_link",
            //     name: "Skip link present",
            //     description: "Ensures skip link is present for keyboard navigation",
            //     default_severity: Severity::Warning,
            //     category: RuleCategory::Accessibility,
            //     check: |page| {
            //         let page = page.clone();
            //         let selector = Selector::parse("a[href^='#']").unwrap();
            //         let document = page.get_document().unwrap();
            //         let mut links = document.select(&selector);
            //         let has_skip_link = links.any(|link| {
            //             link.text()
            //                 .collect::<String>()
            //                 .to_lowercase()
            //                 .contains("skip")
            //         });

            //         CheckResult {
            //             rule_id: "axe.skip_link".to_string(),
            //             passed: has_skip_link,
            //             message: if has_skip_link {
            //                 "Skip link found".to_string()
            //             } else {
            //                 "Skip link not found".to_string()
            //             },
            //         }
            //     },
            // },
            Rule {
                id: "axe.tabindex",
                name: "Valid tabindex values",
                description: "Ensures tabindex values are valid",
                default_severity: Severity::Warning,
                category: RuleCategory::Accessibility,
                check: |page| {
                    let page = page.clone();
                    let selector = Selector::parse("[tabindex]").unwrap();
                    let document = page.get_document().unwrap();
                    let elements = document.select(&selector);
                    let invalid_tabindex = elements
                        .filter(|element| {
                            if let Some(tabindex) = element.attr("tabindex") {
                                tabindex.parse::<i32>().unwrap_or(0) > 0
                            } else {
                                false
                            }
                        })
                        .collect::<Vec<_>>();

                    CheckResult {
                        rule_id: "axe.tabindex".to_string(),
                        passed: invalid_tabindex.is_empty(),
                        message: if invalid_tabindex.is_empty() {
                            "No invalid tabindex values found".to_string()
                        } else {
                            format!(
                                "Found {} elements with invalid tabindex values",
                                invalid_tabindex.len()
                            )
                        },
                    }
                },
            },
        ]
    }
}

fn is_valid_aria_attribute(attr: &str) -> bool {
    let valid_attrs = [
        "aria-activedescendant",
        "aria-atomic",
        "aria-autocomplete",
        "aria-busy",
        "aria-checked",
        "aria-colcount",
        "aria-colindex",
        "aria-colspan",
        "aria-controls",
        "aria-current",
        "aria-describedby",
        "aria-details",
        "aria-disabled",
        "aria-dropeffect",
        "aria-errormessage",
        "aria-expanded",
        "aria-flowto",
        "aria-grabbed",
        "aria-haspopup",
        "aria-hidden",
        "aria-invalid",
        "aria-keyshortcuts",
        "aria-label",
        "aria-labelledby",
        "aria-level",
        "aria-live",
        "aria-modal",
        "aria-multiline",
        "aria-multiselectable",
        "aria-orientation",
        "aria-owns",
        "aria-placeholder",
        "aria-posinset",
        "aria-pressed",
        "aria-readonly",
        "aria-relevant",
        "aria-required",
        "aria-roledescription",
        "aria-rowcount",
        "aria-rowindex",
        "aria-rowspan",
        "aria-selected",
        "aria-setsize",
        "aria-sort",
        "aria-valuemax",
        "aria-valuemin",
        "aria-valuenow",
        "aria-valuetext",
    ];
    valid_attrs.contains(&attr)
}

fn get_required_aria_attributes(role: &str) -> Vec<&'static str> {
    match role {
        "checkbox" => vec!["aria-checked"],
        "combobox" => vec!["aria-expanded"],
        "slider" => vec!["aria-valuenow", "aria-valuemin", "aria-valuemax"],
        "spinbutton" => vec!["aria-valuenow", "aria-valuemin", "aria-valuemax"],
        "tablist" => vec!["aria-orientation"],
        "tab" => vec!["aria-selected"],
        "treeitem" => vec!["aria-expanded"],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{config::RuleConfig, page::Page, page_plugin::SeoPlugin};
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Response, Server};

    use std::convert::Infallible;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_axe_plugin_success() {
        let addr = start_test_server().await;
        let base_url = format!("http://{}/success", addr);

        let plugin = AxePlugin::new();
        let page = Page::from_url(url::Url::parse(&base_url).unwrap())
            .await
            .unwrap();
        let mut config = RuleConfig::new();

        for rule in plugin.available_rules() {
            config.enable_rule(rule.id);
        }
        let results = plugin.analyze(&page, &config);
        for result in results {
            println!("Rule: {}", result.rule_id);
            assert!(result.passed, "Rule {} should have passed", result.rule_id);
        }
    }

    // #[tokio::test]
    // async fn test_axe_plugin_failure() {
    //     let addr = start_test_server().await;
    //     let base_url = format!("http://{}/failure", addr);

    //     let plugin = AxePlugin::new();
    //     let page = Page::from_url(url::Url::parse(&base_url).unwrap())
    //         .await
    //         .unwrap();
    //     let mut config = RuleConfig::new();

    //     for rule in plugin.available_rules() {
    //         config.enable_rule(rule.id);
    //     }
    //     let results = plugin.analyze(&page, &config);
    //     for result in results {
    //         assert!(!result.passed, "Rule {} should have failed", result.rule_id);
    //     }
    // }

    async fn start_test_server() -> SocketAddr {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = TcpListener::bind(addr).await.unwrap();
        let addr = listener.local_addr().unwrap();

        let make_svc = make_service_fn(move |_conn| async move {
            Ok::<_, Infallible>(service_fn(move |req| async move {
                match req.uri().path() {
                    "/success" => Ok::<_, Infallible>(Response::new(Body::from(
                        r#"
                                <html lang="en">
                                    <head>
                                        <title>Test Page</title>
                                        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                                    </head>
                                    <body>
                                        <a href="/page1">Page 1</a>
                                        <a href="/page2">Page 2</a>
                                        <a href="/page1?param=value">Page 1 with params</a>
                                        <a href="/page1#section">Page 1 with hash</a>
                                        <a href="/page1?param=value#section">Page 1 with both</a>
                                        <a href="https://external.com">External</a>
                                    </body>
                                </html>
                            "#,
                    ))),
                    "/failure" => Ok::<_, Infallible>(Response::new(Body::from(
                        r#"
                                <html>
                                    <head>

                                    </head>
                                    <body>
                                        <img src="test.jpg" />
                                        <button><img src="test.jpg" /></button>
                                        <a><img src="test.jpg" /></a>
                                    </body>
                                </html>
                            "#,
                    ))),
                    _ => Ok(Response::new(Body::from("404"))),
                }
            }))
        });

        tokio::spawn(async move {
            Server::from_tcp(listener.into_std().unwrap())
                .unwrap()
                .serve(make_svc)
                .await
                .unwrap();
        });

        addr
    }
}
