use tauri::{App, AppHandle, Wry};
use tauri_plugin_store::{Store, StoreBuilder};

pub struct Setting {
    app: AppHandle,
    store: Store<Wry>
}

impl Setting {
    pub fn new(app: &App) -> Self {
        let setting_path = app.path_resolver()
            .app_data_dir()
            .unwrap()
            .join("setting.json");
        let store = StoreBuilder::new(app.handle().clone(), setting_path).build();
        Self {
            app: app.handle().clone(),
            store,
        }
    }

    // pub fn get(key: &str) -> Option<Value> {
    //
    // }
}
