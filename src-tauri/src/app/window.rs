use tauri::{
    AppHandle,
    Manager,
};

pub fn show_main(app: &AppHandle) {
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
            app,
            "main".to_string(),
            tauri::WindowUrl::App("index.html".into()),
        ).build().unwrap();
    }
}

#[tauri::command]
pub fn show_main_cmd(app: AppHandle) {
    show_main(&app);
}
