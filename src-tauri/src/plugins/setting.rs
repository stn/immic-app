use std::path::PathBuf;
use tauri::{
    AppHandle, Manager, RunEvent, Wry,
    plugin::{self, TauriPlugin},
};
use tauri_plugin_store::{Error, JsonValue, Store, StoreCollection, with_store};


pub fn init() -> TauriPlugin<Wry> {
    plugin::Builder::new("setting")
        .invoke_handler(tauri::generate_handler![set, get, has, delete, load, save])
        .setup(move |app| {
            app.manage(SettingPlugin::new(app.clone()));
            Ok(())
        })
        .on_event(|app, event| {
            match event {
                RunEvent::Ready => {
                    let plugin = app.state::<SettingPlugin>();
                    plugin.start();
                },
                RunEvent::Exit => {
                    let plugin = app.state::<SettingPlugin>();
                    plugin.stop();
                },
                _ => (),
            }
        })
        .build()
}

#[derive(Clone)]
pub struct SettingPlugin {
    app: AppHandle,
    path: PathBuf,
}

impl SettingPlugin {
    fn new(app: AppHandle) -> Self {
        let path = path(&app);
        Self {
            app,
            path,
        }
    }

    fn start(&self) {
        let app1 = self.app.clone();
        let stores = app1.state::<StoreCollection<Wry>>();
        with_store(self.app.clone(), stores, &self.path, |store| {
            store.load().ok();
            store.save()
        }).expect("failed to start setting plugin")
    }

    fn stop(&self) {
        let app1 = self.app.clone();
        let stores = app1.state::<StoreCollection<Wry>>();
        with_store(self.app.clone(), stores, &self.path, |store| {
            store.save()
        }).unwrap_or_else(|e| {
            eprintln!("failed to stop setting plugin: {}", e);
        })
    }

    pub fn set(&self, key: String, value: JsonValue) -> Result<(), Error> {
        let app1 = self.app.clone();
        let stores = app1.state::<StoreCollection<Wry>>();
        with_store(self.app.clone(), stores, &self.path, |store| {
            store.insert(key, value)
        })
    }

    pub fn get(&self, key: String) -> Result<Option<JsonValue>, Error> {
        let app1 = self.app.clone();
        let stores = app1.state::<StoreCollection<Wry>>();
        with_store(self.app.clone(), stores, &self.path, |store| {
            Ok(store.get(key).cloned())
        })
    }

    pub fn has(&self, key: String) -> Result<bool, Error> {
        let app1 = self.app.clone();
        let stores = app1.state::<StoreCollection<Wry>>();
        with_store(self.app.clone(), stores, &self.path, |store| {
            Ok(store.has(key))
        })
    }

    pub fn delete(&self, key: String) -> Result<bool, Error> {
        let app1 = self.app.clone();
        let stores = app1.state::<StoreCollection<Wry>>();
        with_store(self.app.clone(), stores, &self.path, |store| {
            store.delete(key)
        })
    }

    pub fn load(&self) -> Result<(), Error> {
        let app1 = self.app.clone();
        let stores = app1.state::<StoreCollection<Wry>>();
        with_store(self.app.clone(), stores, &self.path, |store| {
            store.load()
        })
    }

    pub fn save(&self) -> Result<(), Error> {
        let app1 = self.app.clone();
        let stores = app1.state::<StoreCollection<Wry>>();
        with_store(self.app.clone(), stores, &self.path, |store| {
            store.save()
        })
    }
}

pub fn with_setting<T, F: FnOnce(&mut Store<Wry>) -> Result<T, Error>>(
    app: AppHandle,
    f: F,
) -> Result<T, Error> {
    let setting = app.state::<SettingPlugin>();
    let stores = app.state::<StoreCollection<Wry>>();
    with_store(app.clone(), stores, &setting.path, f)
}

fn path(app: &AppHandle) -> PathBuf {
    app.path_resolver()
        .app_data_dir()
        .unwrap()
        .join("settings.dat")
}

#[tauri::command]
pub fn set(app: AppHandle, key: String, value: JsonValue) -> Result<(), Error> {
    with_setting(app, |store| {
        store.insert(key, value)
    })
}

#[tauri::command]
pub fn get(app: AppHandle, key: String) -> Result<Option<JsonValue>, Error> {
    with_setting(app, |store| {
        Ok(store.get(key).cloned())
    })
}

#[tauri::command]
pub fn has(app: AppHandle, key: String) -> Result<bool, Error> {
    with_setting(app, |store| {
        Ok(store.has(key))
    })
}

#[tauri::command]
pub fn delete(app: AppHandle, key: String) -> Result<bool, Error> {
    with_setting(app, |store| {
        store.delete(key)
    })
}

#[tauri::command]
pub fn load(app: AppHandle) -> Result<(), Error> {
    with_setting(app, |store| {
        store.load()
    })
}

#[tauri::command]
pub fn save(app: AppHandle) -> Result<(), Error> {
    with_setting(app, |store| {
        store.save()
    })
}
