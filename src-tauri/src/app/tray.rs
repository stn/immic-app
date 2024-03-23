use std::sync::Mutex;

use tauri::{
    AppHandle,
    CustomMenuItem,
    Manager,
    State,
    SystemTray,
    SystemTrayEvent,
    SystemTrayMenu,
    SystemTrayMenuItem,
};

use crate::app::db;
use crate::plugins::Plugin;
use crate::plugins::application::ApplicationPlugin;
use crate::plugins::filelog::FilelogPlugin;
use crate::plugins::screen::ScreenshotPlugin;


pub fn generate_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let preferences = CustomMenuItem::new("Preferences".to_string(), "Preferences");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
        .add_item(preferences)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    SystemTray::new().with_menu(tray_menu)
}

pub fn system_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "quit" => {
                    let app = app.clone();
                    // Plugins
                    let application: State<Mutex<ApplicationPlugin>> = app.state();
                    application.lock().unwrap().stop();
                    let filelog: State<Mutex<FilelogPlugin>> = app.state();
                    filelog.lock().unwrap().stop();
                    let screenshot: State<Mutex<ScreenshotPlugin>> = app.state();
                    screenshot.lock().unwrap().stop();
                    tokio::spawn(async move {
                        // DB
                        db::close().await;

                        // https://github.com/tauri-apps/tauri/discussions/3273
                        tauri::api::process::kill_children();

                        app.exit(0);
                    });
                }
                "show" => {
                    if let Some(window) = app.get_window("main") {
                        if window.is_minimized().unwrap() {
                            window.unminimize().unwrap();
                        } else if window.is_visible().unwrap() {
                            window.set_focus().unwrap();
                        } else {
                            window.show().unwrap();
                        }
                    } else {
                        let window = tauri::WindowBuilder::new(
                            app,
                            "main".to_string(),
                            tauri::WindowUrl::App("index.html".into()),
                        ).build().unwrap();
                    }
                }
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                }
                _ => {}
            }
        }
        _ => {}
    }
}
