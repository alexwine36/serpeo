use seo_analyzer::{
    analyze_url, config::Config, crawl_url, crawler::CrawlResult, AnalysisProgress, CommandOutput,
    PageAnalysis, SeoAnalysis, ShellCommand,
};
use specta_typescript::Typescript;
use std::{collections::HashMap, sync::Mutex};
use tauri::{Emitter, Manager, State};
use tauri_plugin_shell::ShellExt;
use tauri_specta::{collect_commands, collect_events, Builder};

#[derive(Default)]
struct AppData {
    config: Config,
}

struct TauriShell(tauri::AppHandle);

#[async_trait::async_trait]
impl ShellCommand for TauriShell {
    async fn run_command(
        &self,
        command: &str,
        args: &[&str],
    ) -> Result<CommandOutput, std::io::Error> {
        let output = self
            .0
            .shell()
            .command(command)
            .args(args)
            .output()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        Ok(CommandOutput {
            status: output.status.success(),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

#[tauri::command]
#[specta::specta]
async fn analyze_seo(app: tauri::AppHandle, url: String) -> Result<SeoAnalysis, String> {
    let shell = TauriShell(app);
    analyze_url(&shell, url).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
async fn crawl_seo(state: State<'_, Mutex<AppData>>) -> Result<CrawlResult, String> {
    let config = state.lock().unwrap().config.clone();
    println!("Crawling with config: {:?}", config);
    crawl_url(&config).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
async fn get_config(state: State<'_, Mutex<AppData>>) -> Result<Config, String> {
    // let mut app_data = app.state::<Mutex<AppData>>();
    Ok(state.lock().unwrap().config.clone())
}

#[tauri::command]
#[specta::specta]
async fn set_config(state: State<'_, Mutex<AppData>>, config: Config) -> Result<(), String> {
    // let app_data = app.state::<Mutex<AppData>>();
    let mut state = state.lock().unwrap();
    state.config = config;
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn analyze_crawl_seo(
    app: tauri::AppHandle,
    // url: String,
    crawl_result: CrawlResult,
    lighthouse_enabled: bool,
) -> Result<HashMap<String, PageAnalysis>, String> {
    let state = app.state::<Mutex<AppData>>();
    let config = state.lock().unwrap().config.clone();
    let analyzer =
        seo_analyzer::Analyzer::new(&config, lighthouse_enabled).map_err(|e| e.to_string())?;
    let app_handle = app.clone();

    analyzer
        .analyze_crawl_result(crawl_result, move |progress| {
            let _ = app_handle.emit("analysis-progress", progress);
        })
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            analyze_seo,
            crawl_seo,
            analyze_crawl_seo,
            get_config,
            set_config,
        ])
        .events(collect_events![AnalysisProgress]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../src/generated/bindings.ts")
        .expect("Failed to export typescript bindings");

    // Create the tauri app
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppData::default()));
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
