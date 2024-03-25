use serde_json::Value;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Error, JsonValue, Store, StoreBuilder};

pub struct Setting {
    store: Store<Wry>
}

impl Setting {
    pub fn new(app: &AppHandle) -> Self {
        let setting_path = app.path_resolver()
            .app_data_dir()
            .unwrap()
            .join("setting.json");
        let store = StoreBuilder::new(app.clone(), setting_path).build();
        Self {
            store,
        }
    }

    pub fn insert(&mut self, key: String, value: JsonValue) -> Result<(), Error> {
        self.store.insert(key, value)
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<&Value> {
        self.store.get(key)
    }

    pub fn has(&self, key: impl AsRef<str>) -> bool {
        self.store.has(key)
    }

    pub fn delete(&mut self, key: impl AsRef<str>) -> Result<bool, Error> {
        self.store.delete(key)
    }
}
