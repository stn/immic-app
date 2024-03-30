use tauri::{
    AppHandle,
    CustomMenuItem,
    Manager,
    SystemTray,
    SystemTrayEvent,
    SystemTrayMenu,
    SystemTrayMenuItem,
};

use crate::plugins::db::ImmicDb;
use crate::plugins::setting::SettingPlugin;
use crate::plugins::screenshot::ScreenshotPlugin;
use crate::plugins::application::ApplicationPlugin;
use crate::plugins::filelog::FilelogPlugin;

pub fn generate_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show");
    let preferences = CustomMenuItem::new("preferences".to_string(), "Preferences");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(preferences)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    SystemTray::new().with_menu(tray_menu)
}

pub fn system_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    let app = app.clone();
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "quit" => {
                    quit_app(app);
                }
                "show" => {
                    show_main(app);
                }
                "preferences" => {
                    show_preferences(app);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    tokio::spawn(async move {
        // Filelog
        let filelog = app.state::<FilelogPlugin>();
        filelog.stop();

        // Screenshot
        let screenshot = app.state::<ScreenshotPlugin>();
        screenshot.stop();

        // Application
        let application = app.state::<ApplicationPlugin>();
        application.stop();

        // DB
        let db = app.state::<ImmicDb>();
        db.stop().await.ok();

        // Setting
        let setting = app.state::<SettingPlugin>();
        setting.stop();

        app.exit(0);
    });
}

#[tauri::command]
pub fn show_main(app: AppHandle) {
    if let Some(window) = app.get_window("main") {
        if window.is_minimized().unwrap() {
            window.unminimize().unwrap();
        } else if window.is_visible().unwrap() {
            window.set_focus().unwrap();
        } else {
            window.show().unwrap();
        }
    } else {
        tauri::WindowBuilder::new(
            &app,
            "main".to_string(),
            tauri::WindowUrl::App("index.html".into()),
        ).build().unwrap();
    }
}

#[tauri::command]
pub fn show_preferences(app: AppHandle) {
    if let Some(window) = app.get_window("preferences") {
        if window.is_minimized().unwrap() {
            window.unminimize().unwrap();
        } else if window.is_visible().unwrap() {
            window.set_focus().unwrap();
        } else {
            window.show().unwrap();
        }
    } else {
        tauri::WindowBuilder::new(
            &app,
            "preferences".to_string(),
            tauri::WindowUrl::App("preferences.html".into()),
        ).build().unwrap();
    }
}
