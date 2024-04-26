use anyhow::Result;
use log::warn;
use tauri::{
    AppHandle,
    Manager,
};

pub fn show_window(app: &AppHandle, label: &str, url: &str) -> Result<()> {
    if let Some(window) = app.get_window(label) {
        if window.is_minimized()? {
            window.unminimize()?;
        } else if window.is_visible()? {
            window.set_focus()?;
        } else {
            window.show()?;
        }
    } else {
        // ここにはこないはず
        warn!("Window not found: {}", label);

        tauri::WindowBuilder::new(
            app,
            label.to_string(),
            tauri::WindowUrl::App(url.into()),
        ).build()?;
    }

    Ok(())
}

pub fn show_main(app: &AppHandle) -> Result<()> {
    show_window(app, "main", "index.html")
}

pub fn show_info(app: &AppHandle) -> Result<()> {
    show_window(app, "info", "info.html")
}

#[tauri::command]
pub fn show_main_cmd(app: AppHandle) -> Result<(), String> {
    show_main(&app).map_err(|e| e.to_string())
}
