use anyhow::Result;
use seo_analyzer::{AnalysisProgress, AnalysisProgressType, CrawlResult};
use seo_storage::enums::site_run_status::SiteRunStatus;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{future::Future, sync::Mutex};
use tauri::{ipc::private::ResultKind, Listener, Manager};

use tauri_specta::Event;

use crate::AppData;

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct AnalysisStart {
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct SiteRunIdSet {
    pub site_run_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct AnalysisFinished {
    pub site_run_id: i32,
    pub result: CrawlResult,
}

pub fn setup_listeners(app: &tauri::AppHandle) {
    AnalysisStart::listen_any_spawn(app, |data, app| async move {
        let payload = data;
        let app_handle = app;
        let storage_clone = app_handle
            .state::<Mutex<AppData>>()
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex poisoned: {}", e))?
            .storage
            .clone();
        let site_run_id = storage_clone
            .create_site_run(&payload.base_url)
            .await
            .map_err(|e| anyhow::anyhow!("Error creating site run: {}", e))?;
        app_handle
            .state::<Mutex<AppData>>()
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex poisoned: {}", e))?
            .site_run_id = Some(site_run_id);
        SiteRunIdSet { site_run_id }
            .emit(&app_handle)
            .map_err(|e| anyhow::anyhow!("Error emitting site run id: {}", e))?;
        Ok(())
    });

    AnalysisProgress::listen_any_spawn(app, |data, app| async move {
        let payload = data;
        let app_handle = app;
        let storage_clone = app_handle
            .state::<Mutex<AppData>>()
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex poisoned: {}", e))?
            .storage
            .clone();
        let site_run_id = app_handle
            .state::<Mutex<AppData>>()
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex poisoned: {}", e))?
            .site_run_id
            .expect("Site run id is not set");
        match payload.progress_type {
            AnalysisProgressType::AnalyzedPage(page_link) => {
                storage_clone
                    .insert_many_page_rule_results(site_run_id, page_link)
                    .await?;
            }
            AnalysisProgressType::AnalyzedSite(site_result) => {
                storage_clone
                    .insert_many_site_rule_results(site_run_id, site_result)
                    .await?;
            }
            _ => {}
        }

        Ok(())
    });
    AnalysisFinished::listen_any_spawn(app, |data, app| async move {
        let payload = data;
        let result = payload.result;
        let app_handle = app;
        let storage_clone = app_handle
            .state::<Mutex<AppData>>()
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex poisoned: {}", e))?
            .storage
            .clone();
        storage_clone
            .update_site_run_status(payload.site_run_id, SiteRunStatus::Finished)
            .await
            .map_err(|e| anyhow::anyhow!("Error updating site run status: {}", e))?;

        storage_clone
            .handle_crawl_result(payload.site_run_id, result)
            .await?;
        Ok(())
    });
}

trait EventExt: tauri_specta::Event {
    fn listen_any_spawn<Fut>(
        app: &tauri::AppHandle,
        handler: impl Fn(Self, tauri::AppHandle) -> Fut + Send + 'static + Clone,
    ) -> tauri::EventId
    where
        Fut: Future<Output = Result<()>> + Send,
        Self: serde::de::DeserializeOwned + Send + 'static,
    {
        let app = app.clone();
        Self::listen_any(&app.clone(), move |e| {
            let app = app.clone();
            let handler = handler.clone();
            tokio::spawn(async move {
                if let Err(e) = (handler)(e.payload, app).await {
                    eprintln!("Error in event handler: {}", e);
                }
            });
        })
    }
}

impl<T: tauri_specta::Event> EventExt for T {}
