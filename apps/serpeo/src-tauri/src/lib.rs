use seo_analyzer::{analyze_url, crawl_url, CommandOutput, CrawlResult, SeoAnalysis, ShellCommand};
use specta_typescript::Typescript;
use tauri_plugin_shell::ShellExt;
use tauri_specta::{collect_commands, Builder};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
async fn crawl_seo(url: String) -> Result<CrawlResult, String> {
    crawl_url(&url).await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![analyze_seo, crawl_seo]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
