use crate::plugins::axe::AxePlugin;
use crate::plugins::image::ImagePlugin;
use crate::plugins::request::RequestPlugin;
use crate::plugins::seo_basic::SeoBasicPlugin;
use crate::site_plugins::MetaDescriptionPlugin;
use futures::stream::{self, StreamExt};
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use super::config::{Rule, RuleConfig, RuleDisplay, RuleResult};
use super::page::{Page, PageError};
use super::page_plugin::SeoPlugin;
use super::site::Site;
use super::site_plugin::SitePlugin;

#[derive(Clone)]
pub struct PluginRegistry {
    plugins: Arc<Mutex<HashMap<TypeId, Box<dyn SeoPlugin>>>>,
    site_plugins: Arc<Mutex<Vec<Box<dyn SitePlugin>>>>,
    config: Option<RuleConfig>,
    // before_page_hooks: Vec<Box<dyn FnMut(&Page) -> Result<(), PageError> + Send + Sync>>,
    // after_page_hooks:
    //     Vec<Box<dyn FnMut(&Page, &Vec<RuleResult>) -> Result<(), PageError> + Send + Sync>>,
    // before_page_hooks: Vec<Box<dyn for<'a> Fn(&'a Page) -> Result<(), PageError> + Send + Sync>>,
    // after_page_hooks: Vec<
    //     Box<dyn for<'a> Fn(&'a Page, &'a Vec<RuleResult>) -> Result<(), PageError> + Send + Sync>,
    // >,
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
            // before_page_hooks: Vec::new(),
            // after_page_hooks: Vec::new(),
        }
    }

    pub fn set_config(&mut self, config: RuleConfig) {
        self.config = Some(config);
    }

    pub fn get_config(&self) -> Result<&RuleConfig, PageError> {
        self.config.as_ref().ok_or(PageError::ConfigNotSet)
    }

    pub fn register<P: SeoPlugin + 'static>(&mut self, mut plugin: P) -> Result<(), String> {
        let type_id = TypeId::of::<P>();
        plugin.initialize(self)?;
        self.plugins
            .lock()
            .unwrap()
            .insert(type_id, Box::new(plugin));
        Ok(())
    }

    pub fn register_site_plugin<P: SitePlugin + 'static>(
        &mut self,
        mut plugin: P,
    ) -> Result<(), String> {
        plugin.initialize(self)?;
        self.site_plugins.lock().unwrap().push(Box::new(plugin));
        Ok(())
    }

    pub fn get_available_rules(&self) -> Vec<RuleDisplay> {
        let plugins = self.plugins.lock().unwrap();
        let page_rules: Vec<RuleDisplay> = plugins
            .values()
            .flat_map(|plugin| plugin.available_rules())
            .collect();
        let site_rules = self
            .site_plugins
            .lock()
            .unwrap()
            .iter()
            .flat_map(|plugin| plugin.available_rules())
            .collect();
        [page_rules, site_rules].concat()
    }

    pub async fn analyze_async(&self, page: &Page) -> Vec<RuleResult> {
        let config = self.get_config().unwrap();
        let plugins = self.plugins.lock().unwrap();
        let futures = plugins
            .values()
            .map(|plugin| plugin.analyze_async(page, config));
        let results = futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .collect();

        // Run site plugins
        for plugin in self.site_plugins.lock().unwrap().iter_mut() {
            let _ = plugin.after_page_hook(&page, &results);
        }
        results
    }

    pub fn analyze(&self, page: &Page) -> Vec<RuleResult> {
        let config = self.get_config().unwrap();
        let results = self
            .plugins
            .lock()
            .unwrap()
            .values()
            .flat_map(|plugin| plugin.analyze(page, config))
            .collect();

        // Run site plugins
        for plugin in self.site_plugins.lock().unwrap().iter_mut() {
            let _ = plugin.after_page_hook(&page, &results);
        }

        results
    }

    pub fn analyze_site(&self, site: &Site) -> Vec<RuleResult> {
        let config = self.get_config().unwrap();
        self.site_plugins
            .lock()
            .unwrap()
            .iter()
            .flat_map(|plugin| plugin.analyze(site, config))
            .collect()
    }

    pub fn default_with_config() -> Self {
        let mut config = RuleConfig::new();
        let mut registry = Self::default();
        let rules = registry.get_available_rules();
        for rule in rules {
            if !rule.id.starts_with("axe") {
                config.enable_rule(rule.id);
            }
        }
        registry.set_config(config);
        registry
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        let _ = registry.register(ImagePlugin::new());
        let _ = registry.register(SeoBasicPlugin::new());
        let _ = registry.register(AxePlugin::new());
        let _ = registry.register(RequestPlugin::new());
        let _ = registry.register_site_plugin(MetaDescriptionPlugin::new());
        registry
    }
}
