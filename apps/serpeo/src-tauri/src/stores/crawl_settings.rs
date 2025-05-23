use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::StoreExt;

use crate::STORE_FILE;

pub const CRAWL_SETTINGS_KEY: &str = "crawl_settings";

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Type)]
pub struct CrawlSettingsStore {
    pub max_concurrent_requests: u32,
    pub request_delay_ms: u32,
}

impl Default for CrawlSettingsStore {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 10,
            request_delay_ms: 100,
        }
    }
}

impl CrawlSettingsStore {
    pub fn get(app: &AppHandle<Wry>) -> Result<Option<Self>, String> {
        match app.store(STORE_FILE).map(|s| s.get(CRAWL_SETTINGS_KEY)) {
            Ok(Some(store)) => {
                // Handle potential deserialization errors gracefully
                match serde_json::from_value(store) {
                    Ok(settings) => Ok(Some(settings)),
                    Err(e) => Err(format!("Failed to deserialize crawl settings store: {e}")),
                }
            }
            _ => Ok(None),
        }
    }

    pub fn get_or_default(app: &AppHandle) -> Result<Self, String> {
        Self::get(app).map(|settings| settings.unwrap_or_default())
    }

    // i don't trust anyone to not overwrite the whole store lols
    pub fn update(app: &AppHandle, update: impl FnOnce(&mut Self)) -> Result<(), String> {
        let Ok(store) = app.store(STORE_FILE) else {
            return Err("Store not found".to_string());
        };

        let mut settings = Self::get(app)?.unwrap_or_default();
        update(&mut settings);
        store.set(CRAWL_SETTINGS_KEY, json!(settings));
        store.save().map_err(|e| e.to_string())
    }

    fn save(&self, app: &AppHandle) -> Result<(), String> {
        let Ok(store) = app.store(STORE_FILE) else {
            return Err("Store not found".to_string());
        };

        store.set(CRAWL_SETTINGS_KEY, json!(self));
        store.save().map_err(|e| e.to_string())
    }
}

pub fn init(app: &AppHandle) {
    println!("Initializing CrawlSettingsStore");

    let store = match CrawlSettingsStore::get(app) {
        Ok(Some(store)) => store,
        Ok(None) => CrawlSettingsStore::default(),
        Err(e) => {
            eprintln!("Failed to get crawl settings store: {e}");
            return;
        }
    };

    store
        .save(app)
        .expect("Failed to save crawl settings store");

    println!("CrawlSettingsState managed");
}
