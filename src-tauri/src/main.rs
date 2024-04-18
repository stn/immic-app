// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Context as _;
use dotenv::dotenv;
use tauri::Manager;
use log::{error,info};

use immic_app::{
    app,
    plugins,
};

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
        // single instance
        // https://github.com/tauri-apps/plugins-workspace/tree/v1/plugins/single-instance
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            // received a message from another instance
            info!("single-instance: {}, {argv:?}, {cwd}", app.package_info().name);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(plugins::setting::init())
        .plugin(plugins::db::init())
        .plugin(plugins::screenshot::init())
        .plugin(plugins::application::init())
        .plugin(plugins::filelog::init())
        .plugin(plugins::browser::init())
        .plugin(plugins::search::init())
        .invoke_handler(tauri::generate_handler![
            app::quit_app,
            app::restart_app,
            app::window::show_main_cmd,
        ])
        .setup(|app| {
            info!("setup");

            let app = app.handle();

            // // Setting plugin
            let setting = app.state::<plugins::setting::SettingPlugin>();
            setting.start().context("Failed to start setting plugin")?;

            tokio::spawn(async move {
                // DB plugin
                let db = app.state::<plugins::db::ImmicDb>();
                db.start().await.unwrap_or_else(|e| {
                    error!("DB error: {}", e);
                    return;
                });

                // Screenshot plugin
                let screen = app.state::<plugins::screenshot::ScreenshotPlugin>();
                screen.start().unwrap_or_else(|e| {
                    error!("Screenshot run error: {}", e);
                });

                // Application plugin
                let application = app.state::<plugins::application::ApplicationPlugin>();
                application.start().unwrap_or_else(|e| {
                    error!("Application start error: {}", e);
                });

                // FileLog plugin
                let filelog = app.state::<plugins::filelog::FilelogPlugin>();
                filelog.start().unwrap_or_else(|e| {
                    error!("Filelog start error: {}", e);
                });

                // Browser plugin
                let app_browser = app.clone();
                std::thread::spawn(move || {
                    plugins::browser::init_server(app_browser).unwrap_or_else(|e| {
                        error!("Browser server error: {}", e);
                    });
                });

                // Search is not necessary to start
                // Search plugin
                // let search = app.state::<plugins::search::SearchPlugin>();
                // search.start().unwrap_or_else(|e| {
                //     error!("Search start error: {}", e);
                // });

                info!("All plugins started");
            });

            Ok(())
        })
        .system_tray(app::tray::generate_system_tray())
        .on_system_tray_event(app::tray::system_tray_event)
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
        // .run(|app, event|
        //     match event {
        //         tauri::RunEvent::ExitRequested { api, .. } => {
        //             debug!("exit requested");
        //             api.prevent_exit();

        //             let app = app.clone();
        //             tokio::spawn(async move {
        //                 let db = app.state::<plugins::db::ImmicDb>();
        //                 db.stop().await;
        //                 debug!("db stopped");

        //                 debug!("exit");
        //                 app.exit(0);
        //             });
        //         },
        //         _ => {}
        //     }
        // );
}
