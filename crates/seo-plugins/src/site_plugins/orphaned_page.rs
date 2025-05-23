use std::sync::{Arc, Mutex as StdMutex};

use crate::site_analyzer::{LinkSourceType, SiteAnalyzer};

use crate::utils::config::{SiteCheckContext, SiteCheckResult};
use crate::utils::{
    config::{RuleCategory, RuleResult, Severity, SiteRule},
    page::Page,
    registry::PluginRegistry,
    site_plugin::SitePlugin,
};

#[derive(Clone)]
pub struct OrphanedPagePlugin {}

impl Default for OrphanedPagePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl OrphanedPagePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

const PLUGIN_NAME: &str = "OrphanedPage Plugin";

impl SitePlugin for OrphanedPagePlugin {
    fn name(&self) -> &str {
        PLUGIN_NAME
    }

    fn description(&self) -> &str {
        "Check if pages are found only in sitemap but not in links"
    }

    fn after_page_hook(
        &self,
        _page: Arc<StdMutex<Page>>,
        _results: &Vec<RuleResult>,
    ) -> Result<(), String> {
        Ok(())
    }

    fn available_rules(&self) -> Vec<SiteRule> {
        vec![SiteRule {
            id: "orphaned_page.check",
            name: "Orphaned Page",
            plugin_name: PLUGIN_NAME,
            description: "Check if pages are found only in sitemap but not in links",
            default_severity: Severity::Warning,
            category: RuleCategory::SEO,
            passed_message: "No orphaned pages found",
            failed_message: "Orphaned pages found",
        }]
    }

    // async fn pre_analyze(&self, site: &SiteAnalyzer<'_>) -> Result<(), String> {
    //     let site = site.clone();
    //     let links = site.links.lock().await;
    //     // links.iter().for_each(|(url, link)| {
    //     //     if link.link_type == LinkType::External {
    //     //         println!("External link: {}", url);
    //     //     }
    //     // });
    //     Ok(())
    // }
    fn check(&self, rule: &SiteRule, site: &SiteAnalyzer) -> SiteCheckResult {
        let links = site.get_links();
        let orphaned_pages = links
            .iter()
            .filter(|(_, link)| {
                let found_in = link
                    .found_in
                    .iter()
                    .any(|found_in| found_in.link_source_type != LinkSourceType::Sitemap);
                !found_in
            })
            .map(|(url, _)| url.clone())
            .collect::<Vec<_>>();
        // let orphaned_pages = futures::executor::block_on(async {
        //     let links = site.links.lock().await;
        //     let orphaned_pages = links
        //         .iter()
        //         .filter(|(_, link)| {
        //             let found_in = link
        //                 .found_in
        //                 .iter()
        //                 .any(|found_in| found_in.link_source_type != LinkSourceType::Sitemap);
        //             !found_in
        //         })
        //         .map(|(url, _)| url.clone())
        //         .collect::<Vec<_>>();
        //     orphaned_pages
        // });

        let orphaned_pages_count = orphaned_pages.len();
        // println!("Orphaned pages: {:#?}", orphaned_pages);

        match rule.id {
            "orphaned_page.check" => SiteCheckResult {
                rule_id: rule.id.to_string(),
                passed: orphaned_pages_count == 0,
                message: format!("Orphaned pages: {}", orphaned_pages_count),
                context: SiteCheckContext::Urls(orphaned_pages),
            },
            _ => SiteCheckResult {
                rule_id: rule.id.to_string(),
                passed: false,
                message: "Unknown rule".to_string(),
                context: SiteCheckContext::Empty,
            },
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
