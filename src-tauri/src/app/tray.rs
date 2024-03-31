use tauri::{
    AppHandle,
    CustomMenuItem,
    SystemTray,
    SystemTrayEvent,
    SystemTrayMenu,
    SystemTrayMenuItem,
};

use crate::app::{
    app::quit_app,
    window::{
        show_main,
        show_preferences,
    },
};

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
                    show_main(&app);
                }
                "preferences" => {
                    show_preferences(&app);
                }
                _ => {}
            }
        }
        _ => {}
    }
}
