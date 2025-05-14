use seo_analyzer::{crawl_url, AnalysisProgress, CrawlConfig, CrawlResult};
use seo_storage::entities::{prelude::Site, site};
use seo_storage::SeoStorage;
use specta_typescript::Typescript;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use tauri_specta::{collect_commands, collect_events, Builder};

// #[derive(Default)]
struct AppData {
    config: CrawlConfig,
    storage: SeoStorage,
}

#[tauri::command]
#[specta::specta]
async fn get_config(state: State<'_, Mutex<AppData>>) -> Result<CrawlConfig, String> {
    // let mut app_data = app.state::<Mutex<AppData>>();
    Ok(state.lock().unwrap().config.clone())
}

#[tauri::command]
#[specta::specta]
async fn set_config(state: State<'_, Mutex<AppData>>, config: CrawlConfig) -> Result<(), String> {
    // let app_data = app.state::<Mutex<AppData>>();
    let mut state = state.lock().unwrap();
    state.config = config;
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn analyze_url_seo(app: tauri::AppHandle) -> Result<CrawlResult, String> {
    let app_handle = app.clone();
    let config = app_handle
        .state::<Mutex<AppData>>()
        .lock()
        .unwrap()
        .config
        .clone();
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

// #[tauri::command]
// #[specta::specta]
// async fn analyze_url_seo(app: tauri::AppHandle, url: String) -> Result<CrawlResult, String> {
//     let site = SiteAnalyzer::new_with_default(url);
//     let app_handle = app.clone();
//     site.with_progress_callback(move |progress| {
//         let _ = app_handle.emit("analysis-progress", progress);
//     })
//     .await
//     .crawl()
//     .await
//     .map_err(|e| e.to_string())
// }

fn builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            get_config,
            set_config,
            analyze_url_seo,
            get_sites
        ])
        .events(collect_events![AnalysisProgress])
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
        .setup(|app| {
            tauri::async_runtime::block_on(async move {
                let db_path = app.path().app_data_dir().unwrap().join("serpeo.db");
                let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());
                let storage = SeoStorage::new(&db_url).await;
                storage.migrate_up().await.unwrap();
                app.manage(Mutex::new(AppData {
                    config: CrawlConfig::default(),
                    storage,
                }));
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
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
