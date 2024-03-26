use std::path::PathBuf;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_store::{Error, JsonValue, Store, StoreCollection, with_store};

pub fn init(app: AppHandle) -> Result<(), Error> {
    with_setting(app.clone(), |store| {
        store.load().ok();
        store.save()
    })
}

fn path(app: &AppHandle) -> PathBuf {
    app.path_resolver()
        .app_data_dir()
        .unwrap()
        .join("settings.dat")
}

pub fn with_setting<T, F: FnOnce(&mut Store<Wry>) -> Result<T, Error>>(
    app: AppHandle,
    f: F,
) -> Result<T, Error> {
    let stores = app.state::<StoreCollection<Wry>>();
    let path = path(&app);
    with_store(app.clone(), stores, path, f)
}

#[tauri::command]
pub fn setting_set(app: AppHandle, key: String, value: JsonValue) -> Result<(), Error> {
    with_setting(app, |store| {
        store.insert(key, value)
    })
}

#[tauri::command]
pub fn setting_get(app: AppHandle, key: String) -> Result<Option<JsonValue>, Error> {
    with_setting(app, |store| {
        Ok(store.get(key).cloned())
    })
}

#[tauri::command]
pub fn setting_has(app: AppHandle, key: String) -> Result<bool, Error> {
    with_setting(app, |store| {
        Ok(store.has(key))
    })
}

#[tauri::command]
pub fn setting_delete(app: AppHandle, key: String) -> Result<bool, Error> {
    with_setting(app, |store| {
        store.delete(key)
    })
}

#[tauri::command]
pub fn setting_load(app: AppHandle) -> Result<(), Error> {
    with_setting(app, |store| {
        store.load()
    })
}

#[tauri::command]
pub fn setting_save(app: AppHandle) -> Result<(), Error> {
    with_setting(app, |store| {
        store.save()
    })
}
