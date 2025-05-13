use seo_analyzer::{crawl_url, AnalysisProgress, CrawlConfig, CrawlResult};
use seo_storage::{entities::Site, SeoStorage};
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Typescript;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Listener, Manager, State};
use tauri_plugin_sql::{Migration, MigrationKind};
use tauri_specta::{collect_commands, collect_events, Builder, Event};

const DB_PATH: &str = "seo-storage.db";

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
struct AnalysisStart {
    base_url: String,
}

// #[derive(Default)]
struct AppData {
    config: CrawlConfig,
    // pool: SqlitePool,
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
    let base_url = config.base_url.clone();
    {
        let app_handle = app.clone();
        let _ = app_handle.emit("analysis-start", AnalysisStart { base_url });
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
async fn get_sites(app: tauri::AppHandle) -> Result<Vec<Site>, String> {
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
        .events(collect_events![AnalysisProgress, AnalysisStart])
}

fn setup_listeners(app: Arc<Mutex<tauri::AppHandle>>) {
    let root_app = Arc::clone(&app);
    let root_app = root_app.lock().unwrap().clone();

    root_app.listen("analysis-start", move |event| {
        let app_handle = Arc::clone(&app);
        let app_handle = app_handle.lock().unwrap().clone();
        futures::executor::block_on(async move {
            if let Ok(payload) = serde_json::from_str::<AnalysisStart>(&event.payload()) {
                let storage = app_handle
                    .state::<Mutex<AppData>>()
                    .lock()
                    .unwrap()
                    .storage
                    .clone();
                let site_id = storage.create_run(&payload.base_url).await.unwrap();
                println!("site_id: {}", site_id);
            }
        });
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = builder();
    #[cfg(debug_assertions)] // <- Only export on non-release builds
    #[cfg(not(target_os = "ios"))]
    builder
        .export(Typescript::default(), "../src/generated/bindings.ts")
        .expect("Failed to export typescript bindings");

    let migrations = seo_storage::migrations::get_migrations()
        .into_iter()
        .map(|m| Migration {
            version: m.version,
            description: Box::leak(m.description.clone().into_boxed_str()),
            sql: Box::leak(m.schema.clone().into_boxed_str()),
            kind: MigrationKind::Up,
        })
        .collect();

    // Create the tauri app
    tauri::Builder::default()
        // TODO: Add migrations
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(format!("sqlite://{}", DB_PATH).as_str(), migrations)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            tauri::async_runtime::spawn(async move {});
            tauri::async_runtime::block_on(async move {
                let db_path = app.path().app_data_dir().unwrap().join(DB_PATH);
                let db_url = format!("sqlite://{}", db_path.to_string_lossy());
                println!("db_url: {}", db_url);
                let storage = SeoStorage::new(&db_url).await.unwrap();

                app.manage(Mutex::new(AppData {
                    config: CrawlConfig::default(),
                    storage,
                }));
                let app_handle = app.app_handle().clone();
                setup_listeners(Arc::new(Mutex::new(app_handle)));
                // app.manage(Mutex::new(app_handle));
            });
            Ok(())
        })
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
