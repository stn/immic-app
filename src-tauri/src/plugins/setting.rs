use anyhow::{Context as _, Result};
use log::debug;
use std::{
    path::PathBuf,
    sync::RwLock,
};
use tauri::{
    AppHandle, Manager, State, Wry,
    plugin::{self, TauriPlugin},
};
use tauri_plugin_store::{
    Error, JsonValue, Store, StoreBuilder,
};

const SETTING_FILE: &str = "settings.dat";


pub fn init() -> TauriPlugin<Wry> {
    plugin::Builder::new("setting")
        .invoke_handler(tauri::generate_handler![set, get, has, delete, load, save])
        .setup(|app| {
            debug!("setting plugin setup");
            let setting = SettingPlugin::new(app.clone());
            app.manage(setting);
            Ok(())
        })
        .build()
}

pub struct SettingPlugin {
    store: RwLock<Store<Wry>>,
}

impl SettingPlugin {
    fn new(app: AppHandle) -> Self {
        let path = path(&app);
        let store = StoreBuilder::new(app, path).build();
        Self {
            store: RwLock::new(store),
        }
    }

    pub fn start(&self) -> Result<()> {
        debug!("setting plugin start");
        self.load().ok();
        self.save().context("failed to start setting plugin")?;
        Ok(())
    }

    pub fn set(&self, key: String, value: JsonValue) -> Result<(), Error> {
        self.store.write().unwrap().insert(key, value)
    }

    pub fn get(&self, key: &str) -> Result<Option<JsonValue>, Error> {
        Ok(self.store.read().unwrap().get(key).cloned())
    }

    pub fn has(&self, key: &str) -> Result<bool, Error> {
        Ok(self.store.read().unwrap().has(key))
    }

    pub fn delete(&self, key: &str) -> Result<bool, Error> {
        self.store.write().unwrap().delete(key)
    }

    pub fn load(&self) -> Result<(), Error> {
        self.store.write().unwrap().load()
    }

    pub fn save(&self) -> Result<(), Error> {
        self.store.read().unwrap().save()
    }
}

fn path(app: &AppHandle) -> PathBuf {
    app.path_resolver()
        .app_data_dir()
        .unwrap()
        .join(SETTING_FILE)
}

#[tauri::command]
pub fn set(setting: State<SettingPlugin>, key: String, value: JsonValue) -> Result<(), Error> {
    setting.set(key, value)
}

#[tauri::command]
pub fn get(setting: State<SettingPlugin>, key: String) -> Result<Option<JsonValue>, Error> {
    setting.get(&key)
}

#[tauri::command]
pub fn has(setting: State<SettingPlugin>, key: String) -> Result<bool, Error> {
    setting.has(&key)
}

#[tauri::command]
pub fn delete(setting: State<SettingPlugin>, key: String) -> Result<bool, Error> {
    setting.delete(&key)
}

#[tauri::command]
pub fn load(setting: State<SettingPlugin>) -> Result<(), Error> {
    setting.load()
}

#[tauri::command]
pub fn save(setting: State<SettingPlugin>) -> Result<(), Error> {
    setting.save()
}
