mod seo_analyzer;
use seo_analyzer::{analyze_url, SeoAnalysis};
use tauri::State;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn analyze_seo(app: tauri::AppHandle, url: String) -> Result<SeoAnalysis, String> {
    analyze_url(app, url).await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![analyze_seo])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
