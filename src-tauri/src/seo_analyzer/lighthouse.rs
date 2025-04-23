use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tempfile::TempDir;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct LighthouseMetrics {
    pub performance_score: f64,
    pub accessibility_score: f64,
    pub best_practices_score: f64,
    pub seo_score: f64,
    pub pwa_score: f64,
    pub first_contentful_paint: f64,
    pub speed_index: f64,
    pub largest_contentful_paint: f64,
    pub time_to_interactive: f64,
    pub total_blocking_time: f64,
    pub cumulative_layout_shift: f64,
}

#[derive(Error, Debug)]
pub enum LighthouseError {
    #[error("Failed to run Lighthouse: {0}")]
    ExecutionError(String),
    #[error("Failed to parse Lighthouse results: {0}")]
    ParseError(String),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
}

pub async fn run_lighthouse_analysis(
    app_handle: AppHandle,
    url: String,
) -> Result<LighthouseMetrics, LighthouseError> {
    let shell = app_handle.shell();
    // Create a temporary directory for the report
    let temp_dir = TempDir::new()?;
    let report_path = temp_dir.path().join("lighthouse-report.json");
    let report_path_str = report_path.to_str().unwrap().to_string();

    // Run Lighthouse using Node.js
    let output = shell
        .command("lighthouse")
        .args([
            &url,
            "--output=json",
            "--output-path",
            &report_path_str,
            "--chrome-flags=--headless",
            "--only-categories=performance,accessibility,best-practices,seo,pwa",
        ])
        .output()
        .await
        .map_err(|e| LighthouseError::ExecutionError(e.to_string()))?;

    if !output.status.success() {
        return Err(LighthouseError::ExecutionError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    // Read and parse the report
    let report_content = tokio::fs::read_to_string(&report_path)
        .await
        .map_err(|e| LighthouseError::IoError(e))?;

    let report: serde_json::Value = serde_json::from_str(&report_content)
        .map_err(|e| LighthouseError::ParseError(e.to_string()))?;

    println!("{:?}", report);

    // Extract metrics from the report
    let categories = &report["categories"];
    let audits = &report["audits"];

    let metrics = LighthouseMetrics {
        performance_score: get_category_score(categories, "performance"),
        accessibility_score: get_category_score(categories, "accessibility"),
        best_practices_score: get_category_score(categories, "best-practices"),
        seo_score: get_category_score(categories, "seo"),
        pwa_score: get_category_score(categories, "pwa"),
        first_contentful_paint: get_audit_value(audits, "first-contentful-paint"),
        speed_index: get_audit_value(audits, "speed-index"),
        largest_contentful_paint: get_audit_value(audits, "largest-contentful-paint"),
        time_to_interactive: get_audit_value(audits, "interactive"),
        total_blocking_time: get_audit_value(audits, "total-blocking-time"),
        cumulative_layout_shift: get_audit_value(audits, "cumulative-layout-shift"),
    };
    println!("{:?}", metrics);

    Ok(metrics)
}

fn get_category_score(categories: &serde_json::Value, category: &str) -> f64 {
    categories[category]["score"].as_f64().unwrap_or(0.0) * 100.0
}

fn get_audit_value(audits: &serde_json::Value, audit_name: &str) -> f64 {
    audits[audit_name]["numericValue"].as_f64().unwrap_or(0.0)
}
