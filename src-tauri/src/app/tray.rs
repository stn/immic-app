use tauri::{
    AppHandle,
    CustomMenuItem,
    SystemTray,
    SystemTrayEvent,
    SystemTrayMenu,
    SystemTrayMenuItem,
};

use crate::app::{
    quit_app,
    window::{
        show_main,
        show_info,
    },
};

pub fn generate_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show");
    let info = CustomMenuItem::new("info".to_string(), "Info");
    // let preferences = CustomMenuItem::new("settings".to_string(), "Settings");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(info)
        // .add_item(preferences)
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
                    show_main(&app).err().map(|e| log::error!("{}", e));
                }
                "info" => {
                    show_info(&app).err().map(|e| log::error!("{}", e));
                }
                _ => {}
            }
        }
        _ => {}
    }
}
