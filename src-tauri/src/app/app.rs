use tauri::{
    AppHandle,
    Manager,
};

use crate::plugins::{
    db::ImmicDb,
    setting::SettingPlugin,
    screenshot::ScreenshotPlugin,
    application::ApplicationPlugin,
    filelog::FilelogPlugin,
};

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
