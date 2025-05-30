use seo_storage::entities::site_run;
use seo_storage::utils::category_counts::CategoryResultHistory;
use seo_storage::utils::category_detail::CategoryDetailResponse;
use seo_storage::utils::sites_with_site_runs::SiteWithSiteRuns;
use seo_storage::SitePageLinkCount;
use seo_storage::{entities::site, utils::category_counts::CategoryResultDisplay};
use std::sync::Mutex;

use tauri::Manager;

use crate::AppData;

#[tauri::command]
#[specta::specta]
pub async fn get_sites(app: tauri::AppHandle) -> Result<Vec<SiteWithSiteRuns>, String> {
    let app_handle = app.clone();
    let storage = app_handle
        .state::<Mutex<AppData>>()
        .lock()
        .map_err(|e| e.to_string())?
        .storage
        .clone();
    let sites = storage.get_sites().await.map_err(|e| e.to_string())?;
    Ok(sites)
}

#[tauri::command]
#[specta::specta]
pub async fn get_site_category_history(
    app: tauri::AppHandle,
    site_id: i32,
) -> Result<Vec<CategoryResultHistory>, String> {
    let app_handle = app.clone();
    let storage = app_handle
        .state::<Mutex<AppData>>()
        .lock()
        .expect("Failed to lock app data")
        .storage
        .clone();
    let category_result_history = storage
        .get_site_category_history(&site_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(category_result_history)
}

#[tauri::command]
#[specta::specta]
pub async fn get_site_by_id(app: tauri::AppHandle, id: i32) -> Result<site::Model, String> {
    let app_handle = app.clone();
    let storage = app_handle
        .state::<Mutex<AppData>>()
        .lock()
        .map_err(|e| e.to_string())?
        .storage
        .clone();
    let site = storage
        .get_site_by_id(id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(site)
}

#[tauri::command]
#[specta::specta]
pub async fn get_site_run_by_id(
    app: tauri::AppHandle,
    site_run_id: i32,
) -> Result<site_run::Model, String> {
    let app_handle = app.clone();
    let storage = app_handle
        .state::<Mutex<AppData>>()
        .lock()
        .expect("Failed to lock app data")
        .storage
        .clone();
    let site_run = storage
        .get_site_run_by_id(site_run_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(site_run)
}

#[tauri::command]
#[specta::specta]
pub async fn get_category_result(
    app: tauri::AppHandle,
    site_run_id: i32,
) -> Result<CategoryResultDisplay, String> {
    let app_handle = app.clone();
    let storage = app_handle
        .state::<Mutex<AppData>>()
        .lock()
        .map_err(|e| e.to_string())?
        .storage
        .clone();
    let category_result = storage
        .get_category_result(&site_run_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(category_result)
}

#[tauri::command]
#[specta::specta]
pub async fn get_category_result_detail(
    app: tauri::AppHandle,
    site_run_id: i32,
    passed: Option<bool>,
) -> Result<CategoryDetailResponse, String> {
    let app_handle = app.clone();
    let storage = app_handle
        .state::<Mutex<AppData>>()
        .lock()
        .map_err(|e| e.to_string())?
        .storage
        .clone();
    let category_result = storage
        .get_category_result_detail(&site_run_id, passed)
        .await
        .map_err(|e| e.to_string())?;
    Ok(category_result)
}

#[tauri::command]
#[specta::specta]
pub async fn get_site_run_link_counts(
    app: tauri::AppHandle,
    site_run_id: i32,
) -> Result<Vec<SitePageLinkCount>, String> {
    let app_handle = app.clone();
    let storage = app_handle
        .state::<Mutex<AppData>>()
        .lock()
        .map_err(|e| e.to_string())?
        .storage
        .clone();
    let site_run_link_counts = storage
        .get_site_run_link_counts(site_run_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(site_run_link_counts)
}
