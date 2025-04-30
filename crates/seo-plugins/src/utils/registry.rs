use crate::plugins::axe::AxePlugin;
use crate::plugins::image::ImagePlugin;
use crate::plugins::request::RequestPlugin;
use crate::plugins::seo_basic::SeoBasicPlugin;
use futures::stream::{self, StreamExt};
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;

use super::config::{Rule, RuleConfig, RuleResult};
use super::page::{Page, PageError};
use super::page_plugin::SeoPlugin;
use super::site::Site;
use super::site_plugin::SitePlugin;

#[derive(Clone)]
pub struct PluginRegistry {
    plugins: HashMap<TypeId, Box<dyn SeoPlugin>>,
    site_plugins: Vec<Box<dyn SitePlugin>>,
    config: Option<RuleConfig>,
    before_page_hooks: Vec<Box<dyn FnMut(&Page) -> Result<(), PageError> + Send + Sync>>,
    after_page_hooks:
        Vec<Box<dyn FnMut(&Page, &Vec<RuleResult>) -> Result<(), PageError> + Send + Sync>>,
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
            plugins: HashMap::new(),
            site_plugins: Vec::new(),
            config: None,
            before_page_hooks: Vec::new(),
            after_page_hooks: Vec::new(),
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
        self.plugins.insert(type_id, Box::new(plugin));
        Ok(())
    }

    pub fn register_site_plugin<P: SitePlugin + 'static>(
        &mut self,
        mut plugin: P,
    ) -> Result<(), String> {
        plugin.initialize(self)?;
        self.site_plugins.push(Box::new(plugin));
        Ok(())
    }

    pub fn add_before_page_hook<F>(&mut self, hook: F)
    where
        F: Fn(&Page) -> Result<(), PageError> + Send + Sync + 'static,
    {
        self.before_page_hooks.push(Box::new(hook));
    }

    pub fn add_after_page_hook<F>(&mut self, hook: F)
    where
        F: Fn(&Page, &Vec<RuleResult>) -> Result<(), PageError> + Send + Sync + 'static,
    {
        self.after_page_hooks.push(Box::new(hook));
    }

    pub fn get<P: SeoPlugin + 'static>(&self) -> Option<&P> {
        self.plugins
            .get(&TypeId::of::<P>())
            .and_then(|plugin| plugin.as_any().downcast_ref::<P>())
    }

    pub fn get_plugins(&self) -> Vec<&dyn SeoPlugin> {
        self.plugins.values().map(|boxed| boxed.as_ref()).collect()
    }

    pub fn get_available_rules(&self) -> Vec<Rule> {
        self.plugins
            .values()
            .flat_map(|plugin| plugin.available_rules())
            .collect()
    }

    pub async fn analyze_async(&mut self, page: &Page) -> Vec<RuleResult> {
        // Run before page hooks
        for hook in &mut self.before_page_hooks {
            if let Err(e) = hook(page) {
                eprintln!("Error in before page hook: {}", e);
            }
        }

        let config = self.get_config().unwrap();
        let futures = self
            .plugins
            .values()
            .map(|plugin| plugin.analyze_async(page, config));

        let results = futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .collect();

        // Run after page hooks
        for hook in &mut self.after_page_hooks {
            if let Err(e) = hook(page, &results) {
                eprintln!("Error in after page hook: {}", e);
            }
        }

        results
    }

    pub fn analyze(&mut self, page: &Page) -> Vec<RuleResult> {
        // Run before page hooks
        for hook in &mut self.before_page_hooks {
            if let Err(e) = hook(page) {
                eprintln!("Error in before page hook: {}", e);
            }
        }

        let config = self.get_config().unwrap();
        let results = self
            .plugins
            .values()
            .flat_map(|plugin| plugin.analyze(page, config))
            .collect();

        // Run site plugins
        for plugin in &mut self.site_plugins {
            plugin.after_page_hook(&page, &results);
        }

        // Run after page hooks
        for hook in &mut self.after_page_hooks {
            if let Err(e) = hook(page, &results) {
                eprintln!("Error in after page hook: {}", e);
            }
        }

        results
    }

    pub fn analyze_site(&self, site: &Site) -> Vec<RuleResult> {
        let config = self.get_config().unwrap();
        self.site_plugins
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
        registry
    }
}
