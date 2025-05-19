use listeners::{setup_listeners, AnalysisStart};
use seo_analyzer::{crawl_url, AnalysisProgress, CrawlConfig, CrawlResult};
use seo_storage::SeoStorage;
use seo_storage::{entities::site, utils::category_counts::CategoryResultDisplay};
use specta_typescript::Typescript;
use std::sync::Mutex;
use stores::crawl_settings::{self, CrawlSettingsStore, CRAWL_SETTINGS_KEY};
use tauri::{Emitter, Manager};
use tauri_specta::{collect_commands, collect_events, Builder, Event};

mod listeners;
mod stores;

pub const STORE_FILE: &str = "store.json";
// #[derive(Default)]
struct AppData {
    storage: SeoStorage,
    site_run_id: Option<i32>,
}

// #[tauri::command]
// #[specta::specta]
// async fn get_config(state: State<'_, Mutex<AppData>>) -> Result<CrawlConfig, String> {
//     // let mut app_data = app.state::<Mutex<AppData>>();
//     Ok(state.lock().unwrap().config.clone())
// }

// #[tauri::command]
// #[specta::specta]
// async fn set_config(state: State<'_, Mutex<AppData>>, config: CrawlConfig) -> Result<(), String> {
//     // let app_data = app.state::<Mutex<AppData>>();
//     let mut state = state.lock().unwrap();
//     state.config = config;
//     Ok(())
// }

#[tauri::command]
#[specta::specta]
async fn analyze_url_seo(app: tauri::AppHandle, url: String) -> Result<CrawlResult, String> {
    let app_handle = app.clone();
    let base_url = url.clone();

    let crawl_settings = CrawlSettingsStore::get_or_default(&app_handle).unwrap();

    let config = CrawlConfig {
        base_url: url,
        max_concurrent_requests: crawl_settings.max_concurrent_requests,
        request_delay_ms: crawl_settings.request_delay_ms,
    };

    {
        AnalysisStart { base_url }.emit(&app_handle).unwrap();
    }
    let progress_callback = Box::new(move |progress| {
        let _ = app_handle.emit("analysis-progress", progress);
    });
    crawl_url(&config, progress_callback)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
async fn get_sites(app: tauri::AppHandle) -> Result<Vec<site::Model>, String> {
    let app_handle = app.clone();
    let storage = app_handle
        .state::<Mutex<AppData>>()
        .lock()
        .unwrap()
        .storage
        .clone();
    let sites = storage.get_sites().await.map_err(|e| e.to_string())?;
    Ok(sites)
}

#[tauri::command]
#[specta::specta]
async fn get_category_result(
    app: tauri::AppHandle,
    site_run_id: i32,
) -> Result<CategoryResultDisplay, String> {
    let app_handle = app.clone();
    let storage = app_handle
        .state::<Mutex<AppData>>()
        .lock()
        .unwrap()
        .storage
        .clone();
    let category_result = storage
        .get_category_result(&site_run_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(category_result)
}

fn builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            analyze_url_seo,
            get_sites,
            get_category_result
        ])
        .events(collect_events![AnalysisProgress, AnalysisStart])
        .constant("STORE_FILE", STORE_FILE)
        .constant("CRAWL_SETTINGS_KEY", CRAWL_SETTINGS_KEY)
        .typ::<crawl_settings::CrawlSettingsStore>()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = builder();
    #[cfg(debug_assertions)] // <- Only export on non-release builds
    #[cfg(not(target_os = "ios"))]
    builder
        .export(Typescript::default(), "../src/generated/bindings.ts")
        .expect("Failed to export typescript bindings");

    // Create the tauri app
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            let app_clone = app.handle().clone();

            builder.mount_events(&app_clone);
            stores::crawl_settings::init(&app_clone);

            tauri::async_runtime::block_on(async move {
                let db_path = app.path().app_data_dir().unwrap().join("serpeo.db");
                let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
                let storage = SeoStorage::new(&db_url).await;

                storage.migrate_up().await.unwrap();
                app.manage(Mutex::new(AppData {
                    storage,
                    site_run_id: None,
                }));
                setup_listeners(app.handle());
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_types() {
        builder()
            .export(Typescript::default(), "../src/generated/bindings.ts")
            .expect("Failed to export typescript bindings");
    }
}

// NOTE: keeping in case we need to use ShellCommand again
// #[tauri::command]
// #[specta::specta]
// async fn analyze_seo(app: tauri::AppHandle, url: String) -> Result<SeoAnalysis, String> {
//     let shell = TauriShell(app);
//     analyze_url(&shell, url).await.map_err(|e| e.to_string())
// }
