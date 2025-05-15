use futures::executor::block_on;
use seo_analyzer::{crawl_url, AnalysisProgress, AnalysisProgressType, CrawlConfig, CrawlResult};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{
    future::Future,
    sync::{Arc, Mutex},
};
use tauri::{Emitter, Listener, Manager, State};

use tauri_specta::{collect_commands, collect_events, Builder, Event};

use crate::AppData;

// struct AppEvent<T: Event> {
//     name: &'static str,
//     handler: Box<dyn Fn(Arc<Mutex<tauri::AppHandle>>, T) + Send + Sync>,
// }

// impl<T: Event> AppEvent<T> {
//     fn new<F>(name: &'static str, handler: F) -> Self
//     where
//         F: Fn(Arc<Mutex<tauri::AppHandle>>, T) + Send + Sync + 'static,
//     {
//         Self {
//             name,
//             handler: Box::new(handler),
//         }
//     }

//     // fn emit(&self, app: &tauri::AppHandle, payload: T) {
//     //     let _ = app.emit(self.name, payload);
//     // }
// }

pub trait EventHandler: Clone + Send + Sync + 'static {
    fn name(&self) -> &'static str;
    async fn process(&self, app: Arc<Mutex<tauri::AppHandle>>, event: tauri::Event);
    fn handle(&self, app: Arc<Mutex<tauri::AppHandle>>, event: tauri::Event) -> Result<(), String> {
        let result = futures::executor::block_on(self.process(app, event));
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct AnalysisStart {
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct AnalysisStartHandler;

impl EventHandler for AnalysisStartHandler {
    fn name(&self) -> &'static str {
        "analysis-start"
    }

    async fn process(&self, app: Arc<Mutex<tauri::AppHandle>>, event: tauri::Event) {
        println!("AnalysisStartHandler");
        if let Ok(payload) = serde_json::from_str::<AnalysisStart>(event.payload()) {
            let app_handle = app.lock().unwrap();
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
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
pub struct AnalysisProgressHandler;

impl EventHandler for AnalysisProgressHandler {
    fn name(&self) -> &'static str {
        "analysis-progress"
    }

    async fn process(&self, app: Arc<Mutex<tauri::AppHandle>>, event: tauri::Event) {
        println!("AnalysisProgressHandler");
        if let Ok(payload) = serde_json::from_str::<AnalysisProgress>(event.payload()) {
            let app_handle = app.lock().unwrap();
            let storage_clone = app_handle
                .state::<Mutex<AppData>>()
                .lock()
                .unwrap()
                .storage
                .clone();
            // TODO: get actual run instead of hardcoding
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
            // Ok(())
        }
    }
}

// pub fn get_event_handlers() -> Vec<Box<dyn EventHandler>> {
//     vec![
//         Box::new(AnalysisStartHandler),
//         Box::new(AnalysisProgressHandler),
//     ]
// }

pub fn setup_progress_listener(app: &tauri::AppHandle) {
    let app_handle = Arc::new(Mutex::new(app.clone()));
    app.listen(AnalysisProgressHandler.name(), move |event| {
        let app_handle = Arc::clone(&app_handle);
        let handler = AnalysisProgressHandler;
        let _ = handler.handle(app_handle, event);
    });
}

pub fn setup_start_listener(app: &tauri::AppHandle) {
    println!("setup start listener");
    let app_handle = Arc::new(Mutex::new(app.clone()));
    app.listen(AnalysisStartHandler.name(), move |event| {
        let app_handle = Arc::clone(&app_handle);
        let handler = AnalysisProgressHandler;
        let _ = handler.handle(app_handle, event);
    });
}

pub fn setup_listeners(app: &tauri::AppHandle) {
    AnalysisStart::listen_any_spawn(app, |data, app| async move {
        println!("AnalysisStartHandler");
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
        println!("AnalysisProgressHandler");
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
