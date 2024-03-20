// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod plugins;
mod server;

use std::sync::Mutex;
use tauri::{Manager, State};

use app::db;
use app::tray;
use app::setting::Setting;
use plugins::Plugin;
use plugins::application::ApplicationPlugin;
use plugins::filelog::FilelogPlugin;
use plugins::screen::ScreenshotPlugin;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tokio::main]
async fn main() {
    // Enable gpu hardware acceleration on Windows
    //refer to this issue: https://github.com/tauri-apps/tauri/issues/4891
    std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "--ignore-gpu-blocklist");
    std::env::set_var("RUST_LOG", "info");
    // std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    tauri::async_runtime::set(tokio::runtime::Handle::current());

    db::init().await;

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            plugins::screen::list_dates,
            plugins::screen::list_screens,
        ])
        .register_uri_scheme_protocol(
            "iss",
             move |app, request| {
                plugins::screen::handle_iss_protocol(&app, &request)
            }
        )
        .manage(Mutex::new(ScreenshotPlugin::new()))
        .manage(Mutex::new(ApplicationPlugin::new()))
        .manage(Mutex::new(FilelogPlugin::new()))
        .setup(|app| {
            {
                app.manage(Mutex::new(Setting::new(app)));
            }
            {
                let application: State<Mutex<ApplicationPlugin>> = app.state();
                application.lock().unwrap().start();
            }
            {
                let filelog: State<Mutex<FilelogPlugin>> = app.state();
                filelog.lock().unwrap().start();
            }
            {
                let screenshot: State<Mutex<ScreenshotPlugin>> = app.state();
                screenshot.lock().unwrap().start();
            }
            {
                // server
                let handle = Box::new(app.handle());
                std::thread::spawn(move || {
                    server::init(*handle).unwrap_or_else(|e| {
                        eprintln!("Server error: {}", e);
                    });
                });
            }
            Ok(())
        })
        .system_tray(tray::generate_system_tray())
        .on_system_tray_event(tray::system_tray_event)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
