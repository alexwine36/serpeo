use crate::plugins::axe::AxePlugin;
use crate::plugins::image::ImagePlugin;
use crate::plugins::request::RequestPlugin;
use crate::plugins::seo_basic::SeoBasicPlugin;
use crate::plugins::title::TitlePlugin;
use crate::site_analyzer::SiteAnalyzer;
use crate::site_plugins::MetaDescriptionSitePlugin;
use crate::site_plugins::orphaned_page::OrphanedPagePlugin;
use parking_lot::RwLock;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex as StdMutex};

use super::config::{RuleConfig, RuleDisplay, RuleResult};
use super::page::{Page, PageError};
use super::page_plugin::SeoPlugin;
use super::site_plugin::SitePlugin;

#[derive(Clone)]
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<TypeId, Box<dyn SeoPlugin>>>>,
    site_plugins: Arc<RwLock<Vec<Box<dyn SitePlugin>>>>,
    config: Option<RuleConfig>,
}

impl fmt::Debug for PluginRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PluginRegistry")
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            site_plugins: Arc::new(RwLock::new(Vec::new())),
            config: None,
        }
    }

    pub fn set_config(&mut self, config: RuleConfig) {
        self.config = Some(config);
    }

    pub fn get_config(&self) -> Result<&RuleConfig, PageError> {
        self.config.as_ref().ok_or(PageError::ConfigNotSet)
    }

    pub async fn register<P: SeoPlugin + 'static>(&self, plugin: P) -> Result<(), String> {
        let type_id = TypeId::of::<P>();
        plugin.initialize(self)?;
        self.plugins.write().insert(type_id, Box::new(plugin));
        Ok(())
    }

    pub async fn register_site_plugin<P: SitePlugin + 'static>(
        &self,
        plugin: P,
    ) -> Result<(), String> {
        plugin.initialize(self)?;
        self.site_plugins.write().push(Box::new(plugin));
        Ok(())
    }

    pub fn get_available_rules(&self) -> Vec<RuleDisplay> {
        let plugins = self.plugins.read();
        let page_rules: Vec<RuleDisplay> = plugins
            .values()
            .flat_map(|plugin| plugin.available_rules())
            .collect();
        let site_rules = self
            .site_plugins
            .read()
            .iter()
            .flat_map(|plugin| plugin.available_rules())
            .collect();

        [page_rules, site_rules].concat()
    }

    pub async fn analyze_async(&self, page: &Page) -> Result<Vec<RuleResult>, PageError> {
        let config = self.get_config()?;
        let plugins = self.plugins.read();
        let futures = plugins
            .values()
            .map(|plugin| plugin.analyze_async(page, config))
            .collect::<Vec<_>>();
        // drop(plugins); // Drop the lock before awaiting

        let results = futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .collect();

        // Run site plugins
        for plugin in self.site_plugins.read().iter() {
            let _ = plugin.after_page_hook(Arc::new(StdMutex::new(page.clone())), &results);
        }
        Ok(results)
    }

    pub fn analyze(&self, page: &Page) -> Result<Vec<RuleResult>, PageError> {
        let config = self.get_config()?;
        let results = futures::executor::block_on(async {
            let r = self
                .plugins
                .read()
                .values()
                .flat_map(|plugin| plugin.analyze(page, config))
                .collect();

            // Run site plugins
            for plugin in self.site_plugins.read().iter() {
                let _ = plugin.after_page_hook(Arc::new(StdMutex::new(page.clone())), &r);
            }

            r
        });

        Ok(results)
    }

    pub async fn analyze_site(&self, site: &SiteAnalyzer) -> Result<Vec<RuleResult>, PageError> {
        let config = self.get_config()?;
        let results = self
            .site_plugins
            .read()
            .iter()
            .flat_map(|plugin| plugin.analyze(site, config))
            .collect();

        Ok(results)
    }

    pub fn default_with_config() -> Self {
        let mut config = RuleConfig::new();
        let mut registry = Self::default();
        let rules = registry.get_available_rules();

        for rule in rules {
            // if !rule.id.starts_with("axe") {
            config.enable_rule(rule.id);
            // }
        }
        registry.set_config(config);
        registry
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        let registry = Self::new();
        futures::executor::block_on(async {
            let _ = registry.register(ImagePlugin::new()).await;
            let _ = registry.register(SeoBasicPlugin::new()).await;
            let _ = registry.register(TitlePlugin::new()).await;
            let _ = registry.register(AxePlugin::new()).await;
            let _ = registry.register(RequestPlugin::new()).await;
            let _ = registry
                .register(crate::plugins::meta_description::MetaDescriptionPlugin::new())
                .await;
            let _ = registry
                .register_site_plugin(MetaDescriptionSitePlugin::new())
                .await;
            let _ = registry
                .register_site_plugin(OrphanedPagePlugin::new())
                .await;
        });

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ids_are_unique() {
        let registry = PluginRegistry::default_with_config();
        let ids = registry
            .get_available_rules()
            .iter()
            .map(|rule| rule.id.clone())
            .collect::<Vec<String>>();
        let mut unique_ids = ids.clone();
        unique_ids.sort();
        unique_ids.dedup();
        assert_eq!(ids.len(), unique_ids.len());
    }
}
