use anyhow::{Context as _, Result};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use log::{debug, error, info};
use notify_debouncer_full::{
    notify::{self, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher},
    new_debouncer,
    Debouncer,
    DebounceEventResult,
    FileIdMap,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use sqlx::{
    Pool,
    sqlite::Sqlite,
};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, State, Wry,
};
use tokio::sync::mpsc;

use crate::plugins::{
    db,
    setting::SettingPlugin,
};

pub const KIND: &str = "file";
const WATCH_PATHSEST_SETTING: &str = "watch-pathset";
const DEBOUNCE_THRESHOLD: i64 = 60;

static IGNORE_EXTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    vec![
        "pyc",
        "pyo",
        "swp",
        "swx",
        "db",
        "db-shm",
        "db-wal",
        "lock",
        "log",
        "sln",
    ].iter().cloned().collect()
});

#[cfg(target_os = "windows")]
static IGNORE_PAT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\\\.(git|hg|svn|idea|vscode)\\)|(\\(dist|dist-ssr|node_modules|target)\\)").unwrap()
});

#[cfg(not(target_os = "windows"))]
static IGNORE_PAT: Lazy<Regex> = Lazy::new(|| {
    use std::path::MAIN_SEPARATOR;
    Regex::new(&format!("({MAIN_SEPARATOR}\\.(git|hg|svn|idea|vscode){MAIN_SEPARATOR})|({MAIN_SEPARATOR}(dist|dist-ssr|node_modules|target){MAIN_SEPARATOR})")).unwrap()
});

pub fn init() -> TauriPlugin<Wry> {
    plugin::Builder::new("filelog")
        .invoke_handler(tauri::generate_handler![
            list_file_logs_on,
            get_file_info,
        ])
        .setup(|app_handle| {
            debug!("filelog plugin setup");
            let filelog = FilelogPlugin::new(app_handle.clone());
            app_handle.manage(filelog);
            Ok(())
        })
        .build()
}

#[derive(Clone)]
pub struct FilelogPlugin {
    app: AppHandle,
    running: Arc<Mutex<bool>>,
}

impl FilelogPlugin {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&self) -> Result<()> {
        info!("starting filelog");
        
        // Check if setting and db are ready
        let _setting = self.app.state::<SettingPlugin>();
        let _db = self.app.state::<db::ImmicDb>();

        let watch_pathset = self.watch_pathset()?;
        if watch_pathset.is_empty() {
            debug!("watch_pathset is empty");
            return Ok(());
        }

        *self.running.lock().unwrap() = true;

        let self_clone = self.clone();
        tokio::spawn(async move {
            self_clone.async_watch(&watch_pathset).await.unwrap();
        });

        Ok(())
    }

    async fn async_watch(&self, pathset: &Vec<PathBuf>) -> Result<()> {
        let (mut debouncer, mut rx) = self.async_watcher()?;

        // Add each path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        for path in pathset.iter() {
            debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
        }

        while let Some(infos) = rx.recv().await {
            // Check if the watcher has been stopped
            if !*self.running.lock().unwrap() {
                // 実際には、上のrecvがブロックされているので、ここには来ない。
                // watcherをdropする必要がある。
                break;
            }

            for info in infos.iter() {
                debug!("file event info: {:?}", info);
                if let Err(e) = self.maybe_insert_info(info).await {
                    error!("Error on maybe_insert_info: {:?}", e);
                }
            }
        }

        Ok(())
    }

    fn async_watcher(&self) -> notify::Result<(Debouncer<RecommendedWatcher, FileIdMap>, mpsc::Receiver<Vec<FileEventInfo>>)> {
        let (tx, rx) = mpsc::channel(32);

        // durationを大きくしすぎるとイベントの即時性が失われる。別途、logと比較して保存するかをチェックする。
        let debouncer = new_debouncer(Duration::from_secs(2), None, move |result: DebounceEventResult| {
            match result {
                Ok(debounce_events) => {
                    let infos = debounce_events.iter()
                        .map(|de| {
                            let info = FileEventInfo::from(&de.event);
                            if check_ignore(&info) {
                                debug!("ignore: {:?}", info);
                                None
                            } else {
                                Some(info)
                            }
                        })
                        .filter(|info| info.is_some())
                        .map(|info| info.unwrap())
                        .collect::<Vec<_>>();
                    tx.blocking_send(infos).unwrap();
                },
                Err(errors) => {
                    errors.iter().for_each(|e| {
                        error!("watch error: {:?}", e);
                    });
                },
            }
        }).unwrap();

        Ok((debouncer, rx))

        // let watcher = RecommendedWatcher::new(
        //     move |res| {
        //         match res {
        //             Ok(event) => {
        //                 let info = FileEventInfo::from(event);
        //                 if check_ignore(&info) {
        //                     debug!("ignore: {:?}", info);
        //                     return;
        //                 }
        //                 tx.blocking_send(info).unwrap();
        //             },
        //             Err(e) => {
        //                 error!("watch error: {:?}", e);
        //             }
        //         }
        //     },
        //     Config::default(),
        // )?;

        // Ok((watcher, rx))
    }

    fn watch_pathset(&self) -> Result<Vec<PathBuf>> {
        let setting = self.app.state::<SettingPlugin>();
        let watch_pathset = setting.get(WATCH_PATHSEST_SETTING)?
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .map(|s| s.split('|').map(PathBuf::from).collect());
        debug!("watch_pathset: {:?}", watch_pathset);

        Ok(watch_pathset.unwrap())
    }

    pub fn stop(&self) {
        debug!("filelog stop");
        *self.running.lock().unwrap() = false;
    }

    async fn maybe_insert_info(&self, info: &FileEventInfo) -> Result<Option<i64>> {
        if self.check_debounce(info).await? {
            debug!("filelog: debounced!");
            return Ok(None);
        }
        let id = self.insert_info(info).await?;
        Ok(Some(id))
    }

    async fn insert_info(&self, info: &FileEventInfo) -> Result<i64> {
        let timestamp = Utc::now();

        let db = self.app.state::<db::ImmicDb>();
        let event_id = db.insert_eventlog(timestamp, KIND).await?;
        let pool = db.pool().await.expect("db pool is not set");

        let path = info.path.to_string_lossy().to_string();

        // Upsert file_info by path
        let result = sqlx::query(
            r#"
            INSERT OR REPLACE INTO file_info (id, path, last_update)
            VALUES (
                (SELECT id FROM file_info WHERE path = ?),
                ?,
                ?
            );
            "#
        )
        .bind(&path)
        .bind(&path)
        .bind(timestamp.timestamp())
        .execute(&pool)
        .await?;

        let info_id = result.last_insert_rowid();

        // Insert file_log
        let result = sqlx::query(
            r#"
            INSERT INTO file_log (event_id, info_id, kind)
            VALUES (?, ?, ?)
            "#
        )
        .bind(event_id)
        .bind(info_id)
        .bind(&info.kind.to_string())
        .execute(&pool)
        .await?;

        let log_id = result.last_insert_rowid();
        Ok(log_id)
    }

    pub async fn insert_file_log_with(&self, pool: &Pool<Sqlite>, log: &FileLog) -> Result<i64> {
        let timestamp = DateTime::from_timestamp(log.timestamp, 0).context("Invalid timestamp")?;

        let db = self.app.state::<db::ImmicDb>();
        let event_id = db.insert_eventlog_with(pool, timestamp, KIND).await?;

        // Upsert file_info by path
        let result = sqlx::query(
            r#"
            INSERT OR REPLACE INTO file_info (id, path, last_update)
            VALUES (
                (SELECT id FROM file_info WHERE path = ?),
                ?,
                ?
            );
            "#
        )
        .bind(&log.path)
        .bind(&log.path)
        .bind(timestamp.timestamp())
        .execute(pool)
        .await?;

        let info_id = result.last_insert_rowid();

        // Insert file_log
        let result = sqlx::query(
            r#"
            INSERT INTO file_log (event_id, info_id, kind)
            VALUES (?, ?, ?)
            "#
        )
        .bind(event_id)
        .bind(info_id)
        .bind(&log.kind)
        .execute(pool)
        .await?;

        let log_id = result.last_insert_rowid();
        Ok(log_id)
    }

    async fn check_debounce(&self, info: &FileEventInfo) -> Result<bool> {
        let timestamp = Utc::now().timestamp();

        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await.context("db pool is not set")?;

        let result = sqlx::query_as::<_, (Option<i64>,)>(
            r#"
            SELECT last_update
            FROM file_info
            WHERE path = ?
            "#
        )
        .bind(&info.path.to_string_lossy().to_string())
        .fetch_one(&pool)
        .await;

        if let Ok((last_update,)) = result {
            if last_update.is_none() {
                return Ok(false);
            }

            let last_update = last_update.unwrap();
            if timestamp - last_update < DEBOUNCE_THRESHOLD {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn list_file_logs_on(&self, date: String) -> Result<Vec<FileLog>> {
        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await.context("db pool is not set")?;

        let mut rows = sqlx::query_as::<_, (
            i64, i64,
            i64, Option<String>,
            i64, String,
        )>(
            r#"
            SELECT
            e.id, e.timestamp,
            f.id, f.kind,
            i.id, i.path
            FROM event_log e
            INNER JOIN file_log f ON e.id = f.event_id
            INNER JOIN file_info i ON f.info_id = i.id
            WHERE e.kind = ? AND e.date = ?
            ORDER BY e.id
            "#
        )
        .bind(KIND)
        .bind(&date)
        .fetch(&pool);

        let mut filelogs = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let (
                event_id, timestamp,
                id, kind,
                info_id, path,
            ) = row;
            filelogs.push(FileLog {
                id,
                event_id,
                timestamp,
                date: date.clone(),
                info_id,
                path,
                kind,
            });
        }
        Ok(filelogs)
    }

    pub async fn get_file_info(&self, file_id: i64) -> Result<FileInfo> {
        debug!("get_file_info: file_id={}", file_id);

        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await.expect("db pool is not set");

        let result = sqlx::query_as::<_, (i64, String, Option<i64>)>(
            r#"
            SELECT id, path, last_update
            FROM file_info
            WHERE id = ?
            "#
        )
        .bind(file_id)
        .fetch_one(&pool)
        .await;

        match result {
            Ok((id, path, last_update)) => {
                Ok(FileInfo {
                    id,
                    path,
                    last_update,
                })
            },
            Err(_) => {
                Err(anyhow::anyhow!("Not found"))
            }
        }
    }
}

#[derive(Debug,PartialEq)]
struct FileEventInfo {
    kind: FileKind,
    path: PathBuf,
}

#[derive(Debug,PartialEq)]
enum FileKind {
    Create,
    Modify,
    Remove,
    Other,
}

impl fmt::Display for FileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileKind::Create => write!(f, "create"),
            FileKind::Modify => write!(f, "modify"),
            FileKind::Remove => write!(f, "remove"),
            FileKind::Other => write!(f, "other"),
        }
    }
}

impl From<&Event> for FileEventInfo {
    fn from(event: &Event) -> Self {
        let kind = match event.kind {
            EventKind::Create(_) => FileKind::Create,
            EventKind::Modify(_) => FileKind::Modify,
            EventKind::Remove(_) => FileKind::Remove,
            _ => FileKind::Other,
        };

        // Rename event has two paths, but we only check the first one for now.
        let path = event.paths.iter().next().unwrap().clone();

        Self {
            kind,
            path,
        }
    }
}

fn check_ignore(info: &FileEventInfo) -> bool {
    // Ignore other events than create, modify, remove
    if info.kind == FileKind::Other {
        return true;
    }

    let name = info.path.file_name();
    if name.is_none() {
        return true;
    }

    let ext = info.path.extension();
    if ext.is_some() {
        let ext = ext.unwrap().to_string_lossy().to_string();
        if IGNORE_EXTS.contains(&ext.as_str()) {
            return true;
        }
    }

    let name = name.unwrap().to_string_lossy().to_string();

    // Ignore hidden files
    if name.starts_with(".") {
        return true;
    }

    // Ignore temporary files
    if name.ends_with("~") {
        return true;
    }

    // Ignore files in ignore list
    if IGNORE_PAT.is_match(&info.path.to_string_lossy().to_string()) {
        return true;
    }

    false
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileLog {
    pub id: i64,
    pub event_id: i64,
    pub timestamp: i64,
    pub date: String,
    pub info_id: i64,
    pub path: String,
    pub kind: Option<String>,
}

impl db::Timestamp for FileLog {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub id: i64,
    pub path: String,
    pub last_update: Option<i64>,
}

#[tauri::command]
pub async fn list_file_logs_on(file_log: State<'_, FilelogPlugin>, date: String) -> Result<Vec<FileLog>, String> {
    file_log.list_file_logs_on(date).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_file_info(file_log: State<'_, FilelogPlugin>, file_id: i64) -> Result<FileInfo, String> {
    file_log.get_file_info(file_id).await.map_err(|e| e.to_string())
}
