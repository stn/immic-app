// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod plugins;

use app::tray;

use tauri::{Manager, State};
use crate::plugins::screen::ScreenshotPlugin;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    // Enable gpu hardware acceleration on Windows
    //refer to this issue: https://github.com/tauri-apps/tauri/issues/4891
    std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "--ignore-gpu-blocklist");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .manage(ScreenshotPlugin::new())
        .setup(|app| {
            let sched: State<ScreenshotPlugin> = app.state();
            sched.start();
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
