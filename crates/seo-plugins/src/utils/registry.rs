use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::plugins::html::HtmlPlugin;

use super::config::{CheckResult, Rule, RuleConfig, SeoPlugin};
use super::page::{Page, PageError};

// Registry to store and provide access to plugins
pub struct PluginRegistry {
    plugins: HashMap<TypeId, Box<dyn SeoPlugin>>,
    config: Option<RuleConfig>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            config: None,
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

    pub fn analyze(&self, page: &Page) -> Vec<CheckResult> {
        let config = self.get_config().unwrap();
        self.plugins
            .values()
            .flat_map(|plugin| plugin.analyze(page, config))
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        let _ = registry.register(HtmlPlugin::new());
        registry
    }
}
