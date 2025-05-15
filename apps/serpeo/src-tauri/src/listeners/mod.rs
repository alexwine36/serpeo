use seo_analyzer::{AnalysisProgress, AnalysisProgressType, CrawlConfig, CrawlResult};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{
    future::Future,
    sync::{Arc, Mutex},
};
use tauri::{Emitter, Listener, Manager, State};

use tauri_specta::{collect_commands, collect_events, Builder, Event};

use crate::AppData;

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct AnalysisStart {
    pub base_url: String,
}

pub fn setup_listeners(app: &tauri::AppHandle) {
    AnalysisStart::listen_any_spawn(app, |data, app| async move {
        // println!("AnalysisStartHandler");
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
    });

    AnalysisProgress::listen_any_spawn(app, |data, app| async move {
        // println!("AnalysisProgressHandler");
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
        match payload.progress_type {
            AnalysisProgressType::AnalyzedPage(page_link) => {
                storage_clone
                    .insert_many_page_rule_results(site_run_id, page_link)
                    .await
                    .unwrap();
            }
            _ => {}
        }
    });
    // setup_start_listener(app);
    // setup_progress_listener(app);
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
