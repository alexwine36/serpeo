use crate::plugins::axe::AxePlugin;
use crate::plugins::image::ImagePlugin;
use crate::plugins::request::RequestPlugin;
use crate::plugins::seo_basic::SeoBasicPlugin;
use crate::plugins::title::TitlePlugin;
use crate::site_analyzer::SiteAnalyzer;
use crate::site_plugins::MetaDescriptionSitePlugin;
use crate::site_plugins::orphaned_page::OrphanedPagePlugin;
use futures::stream::StreamExt;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;

use super::config::{RuleConfig, RuleDisplay, RuleResult};
use super::page::{Page, PageError};
use super::page_plugin::SeoPlugin;
use super::site_plugin::SitePlugin;

#[derive(Clone)]
pub struct PluginRegistry {
    plugins: Arc<Mutex<HashMap<TypeId, Box<dyn SeoPlugin>>>>,
    site_plugins: Arc<Mutex<Vec<Box<dyn SitePlugin>>>>,
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
            plugins: Arc::new(Mutex::new(HashMap::new())),
            site_plugins: Arc::new(Mutex::new(Vec::new())),
            config: None,
        }
    }

    pub fn set_config(&mut self, config: RuleConfig) {
        self.config = Some(config);
    }

    pub fn get_config(&self) -> Result<&RuleConfig, PageError> {
        self.config.as_ref().ok_or(PageError::ConfigNotSet)
    }

    pub async fn register<P: SeoPlugin + 'static>(&mut self, mut plugin: P) -> Result<(), String> {
        let type_id = TypeId::of::<P>();
        plugin.initialize(self)?;
        self.plugins.lock().await.insert(type_id, Box::new(plugin));
        Ok(())
    }

    pub async fn register_site_plugin<P: SitePlugin + 'static>(
        &mut self,
        mut plugin: P,
    ) -> Result<(), String> {
        plugin.initialize(self)?;
        self.site_plugins.lock().await.push(Box::new(plugin));
        Ok(())
    }

    async fn get_available_rules_async(&self) -> Vec<RuleDisplay> {
        let plugins = self.plugins.lock().await;
        let page_rules: Vec<RuleDisplay> = plugins
            .values()
            .flat_map(|plugin| plugin.available_rules())
            .collect();
        let site_rules = self
            .site_plugins
            .lock()
            .await
            .iter()
            .flat_map(|plugin| plugin.available_rules())
            .collect();

        [page_rules, site_rules].concat()
    }

    pub fn get_available_rules(&self) -> Vec<RuleDisplay> {
        futures::executor::block_on(self.get_available_rules_async())
    }

    pub async fn analyze_async(&self, page: &Page) -> Vec<RuleResult> {
        let config = self.get_config().unwrap();
        let plugins = self.plugins.lock().await;
        let futures = plugins
            .values()
            .map(|plugin| plugin.analyze_async(page, config));
        let results = futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .collect();

        // Run site plugins
        for plugin in self.site_plugins.lock().await.iter_mut() {
            let _ = plugin.after_page_hook(Arc::new(StdMutex::new(page.clone())), &results);
        }
        results
    }

    pub fn analyze(&self, page: &Page) -> Vec<RuleResult> {
        let config = self.get_config().unwrap();
        let results = futures::executor::block_on(async {
            let r = self
                .plugins
                .lock()
                .await
                .values()
                .flat_map(|plugin| plugin.analyze(page, config))
                .collect();

            // Run site plugins
            for plugin in self.site_plugins.lock().await.iter_mut() {
                let _ = plugin.after_page_hook(Arc::new(StdMutex::new(page.clone())), &r);
            }

            r
        });

        results
    }

    pub async fn analyze_site(&self, site: &SiteAnalyzer) -> Vec<RuleResult> {
        let config = self.get_config().unwrap();
        self.site_plugins
            .lock()
            .await
            .iter()
            .flat_map(|plugin| plugin.analyze(site, config))
            .collect()
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
        let mut registry = Self::new();
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
