use seo_analyzer::{AnalysisProgress, AnalysisProgressType, CrawlResult};
use seo_storage::enums::site_run_status::SiteRunStatus;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{future::Future, sync::Mutex};
use tauri::{Listener, Manager};

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
            .unwrap()
            .storage
            .clone();
        let site_run_id = storage_clone
            .create_site_run(&payload.base_url)
            .await
            .unwrap();
        app_handle
            .state::<Mutex<AppData>>()
            .lock()
            .unwrap()
            .site_run_id = Some(site_run_id);
        SiteRunIdSet { site_run_id }.emit(&app_handle).unwrap();
    });

    AnalysisProgress::listen_any_spawn(app, |data, app| async move {
        let payload = data;
        let app_handle = app;
        let storage_clone = app_handle
            .state::<Mutex<AppData>>()
            .lock()
            .unwrap()
            .storage
            .clone();
        let site_run_id = app_handle
            .state::<Mutex<AppData>>()
            .lock()
            .unwrap()
            .site_run_id
            .unwrap();
        if let AnalysisProgressType::AnalyzedPage(page_link) = payload.progress_type {
            storage_clone
                .insert_many_page_rule_results(site_run_id, page_link)
                .await
                .unwrap();
        }
    });
    AnalysisFinished::listen_any_spawn(app, |data, app| async move {
        let payload = data;
        let result = payload.result;
        let app_handle = app;
        let storage_clone = app_handle
            .state::<Mutex<AppData>>()
            .lock()
            .unwrap()
            .storage
            .clone();
        storage_clone
            .update_site_run_status(payload.site_run_id, SiteRunStatus::Finished)
            .await
            .unwrap();

        storage_clone
            .handle_crawl_result(payload.site_run_id, result)
            .await
            .unwrap();
    });
}

trait EventExt: tauri_specta::Event {
    fn listen_any_spawn<Fut>(
        app: &tauri::AppHandle,
        handler: impl Fn(Self, tauri::AppHandle) -> Fut + Send + 'static + Clone,
    ) -> tauri::EventId
    where
        Fut: Future + Send,
        Self: serde::de::DeserializeOwned + Send + 'static,
    {
        let app = app.clone();
        Self::listen_any(&app.clone(), move |e| {
            let app = app.clone();
            let handler = handler.clone();
            tokio::spawn(async move {
                (handler)(e.payload, app).await;
            });
        })
    }
}

impl<T: tauri_specta::Event> EventExt for T {}
