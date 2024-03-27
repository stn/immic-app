// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod plugins;

use std::sync::Mutex;
use app::setting;
use dotenv::dotenv;
use tauri::{Manager, State};
use log::error;

use app::db;
use app::tray;
use app::server;
use plugins::Plugin;
use plugins::application::ApplicationPlugin;
use plugins::filelog::FilelogPlugin;
use plugins::screen::ScreenshotPlugin;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Enable gpu hardware acceleration on Windows
    //refer to this issue: https://github.com/tauri-apps/tauri/issues/4891
    std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "--ignore-gpu-blocklist");
    
    // Enable logging
    env_logger::init();

    tauri::async_runtime::set(tokio::runtime::Handle::current());

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            app::setting::setting_set,
            app::setting::setting_get,
            app::setting::setting_has,
            app::setting::setting_delete,
            app::setting::setting_load,
            app::setting::setting_save,
            app::tray::quit_app,
            app::tray::show_main,
            app::tray::show_preferences,
            db::list_eventlog_dates,
            db::list_eventlog_on,
            plugins::application::list_application_logs,
            plugins::application::get_application_info,
            plugins::browser::list_browser_logs,
            plugins::browser::get_browser_info,
            plugins::filelog::list_file_logs,
            plugins::filelog::get_file_info,
            plugins::screen::list_screenshots,
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
            let app = app.handle();

            // Initialize settings
            setting::init(app.clone())?;

            tokio::spawn(async move {
                db::init(&app).await;

                let application: State<Mutex<ApplicationPlugin>> = app.state();
                application.lock().unwrap().start();

                let filelog: State<Mutex<FilelogPlugin>> = app.state();
                filelog.lock().unwrap().start();

                let screenshot: State<Mutex<ScreenshotPlugin>> = app.state();
                screenshot.lock().unwrap().start();

                // server::init will block the thread
                std::thread::spawn(move || {
                    server::init(app).unwrap_or_else(|e| {
                        error!("Server error: {}", e);
                    });
                });
            });
            Ok(())
        })
        .system_tray(tray::generate_system_tray())
        .on_system_tray_event(tray::system_tray_event)
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
        // .build(tauri::generate_context!())
        // .expect("error while running tauri application")
        // .run(|_app_handle, event| match event {
        //     tauri::RunEvent::ExitRequested { api, .. } => {
        //         api.prevent_exit();
        //     }
        //     _ => {}
        // });
}
