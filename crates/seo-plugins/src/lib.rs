// Core plugin traits
use std::any::{Any, TypeId};
use std::collections::HashMap;

mod plugins;
mod utils;
// Result of an SEO check

// Example plugins

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{
        config::{RuleConfig, SeoPlugin},
        page::Page,
    };
    use plugins::html::HtmlPlugin;
    use utils::registry::PluginRegistry;

    #[tokio::test]
    async fn test_registry() {
        let html = r#"
        <html>
            <head>
                <title>Test Page</title>
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
    "#;
        let page = Page::from_html(html.to_string());

        let mut registry = PluginRegistry::default();

        let mut config = RuleConfig::new();
        config.enable_rule("html.title");
        println!("config: {:?}", config);
        registry.set_config(config);
        let results = registry.analyze(&page);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_id, "html.title");
        assert_eq!(results[0].passed, true);
    }
    #[tokio::test]
    async fn test_static_html() {
        let html = r#"
                                <html>
                                    <head>
                                        
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
                            "#;
        let page = Page::from_html(html.to_string());
        let mut config = RuleConfig::new();
        config.enable_rule("html.title");
        let mut registry = PluginRegistry::default();
        registry.set_config(config);
        let results = registry.analyze(&page);
        println!("results: {:?}", results);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_id, "html.title");
        assert_eq!(results[0].passed, false);
    }
}
