use anyhow::{anyhow, Result};
use chrono::{DateTime, Timelike, Utc};
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
use std::{
    collections::HashSet,
    fmt,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
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

const KIND: &str = "file";
const WATCH_PATHSEST_SETTING: &str = "watch-pathset";

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
            list_file_logs,
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
                match self.insert_info(info).await {
                    Ok(id) => {
                        debug!("filelog: inserted id: {:?}", id);
                    },
                    Err(e) => {
                        error!("Error on insert filelog: {:?}", e);
                    },
                }
            }
        }

        Ok(())
    }

    fn async_watcher(&self) -> notify::Result<(Debouncer<RecommendedWatcher, FileIdMap>, mpsc::Receiver<Vec<FileEventInfo>>)> {
        let (tx, rx) = mpsc::channel(32);

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
            .map(|s| s.split(',').map(PathBuf::from).collect());
        debug!("watch_pathset: {:?}", watch_pathset);

        Ok(watch_pathset.unwrap())
    }

    pub fn stop(&self) {
        debug!("filelog stop");
        *self.running.lock().unwrap() = false;
    }

    async fn insert_info(&self, info: &FileEventInfo) -> Result<i64> {
        let timestamp = Utc::now();

        let db = self.app.state::<db::ImmicDb>();
        let event_id = db.insert_eventlog(timestamp, KIND).await?;
        let pool = db.pool().await.expect("db pool is not set");

        let path = info.path.to_string_lossy().to_string();

        // Search file_info by path
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT id
            FROM file_info
            WHERE path = ?
            "#
        )
        .bind(&path)
        .fetch_one(&pool)
        .await;

        let info_id = match result {
            Ok((id,)) => id,
            Err(_) => {
                // Insert file_info for new path
                let result = sqlx::query(
                    r#"
                    INSERT INTO file_info (path)
                    VALUES (?)
                    "#
                )
                .bind(&path)
                .execute(&pool)
                .await?;
                result.last_insert_rowid()
            }
        };

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

        // Update event_log with log_id
        let log_id = result.last_insert_rowid();
        db.update_eventlog_logid(event_id , log_id).await?;

        Ok(log_id)
    }

    pub async fn list_file_logs(&self, timestamp: i64, interval: db::Interval) -> Result<Vec<(String, Vec<FileLog>)>> {
        let dt = DateTime::from_timestamp_millis(timestamp);
        if dt.is_none() {
            error!("Invalid timestamp: {}", timestamp);
            return Err(anyhow!("Invalid timestamp"));
        };
        let dt = dt.unwrap();

        let local_time = dt.with_timezone(&chrono::Local);
        let date = local_time.format("%Y%m%d").to_string();
        // debug!("list_filelogs: date: {}", date);

        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await.expect("db pool is not set");

        let filelogs: Vec<FileLog> = sqlx::query_as::<_,
            (i64, i64, String, String, i64,
            i64, i64, Option<String>)
        >(
            r#"
            SELECT
            e.id, e.timestamp, e.date, e.kind, e.log_id,
            f.id, f.info_id, f.kind
            FROM event_log e
            INNER JOIN file_log f ON e.log_id = f.id
            WHERE e.kind = ? AND e.date = ?
            ORDER BY e.timestamp
            "#
        )
        .bind(KIND)
        .bind(date)
        .fetch_all(&pool)
        .await
        .unwrap_or(Vec::new())
        .iter()
        .map(|row| {
            let (event_id, timestamp, date, _kind, _log_id,
                id, info_id, kind,
            ) = row;
            FileLog {
                id: *id,
                event_id: *event_id,
                timestamp: *timestamp,
                date: date.clone(),
                info_id: *info_id,
                kind: kind.clone(),
            }
        })
        .collect();
        // debug!("list_filelogs: filelogs: {:?}", filelogs);

        match interval {
            db::Interval::Hourly => {
                let mut filelogs_by_hour = Vec::new();
                let mut logs = Vec::new();

                let mut hour = 0;
                let mut ts = local_time.with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().timestamp();
                for log in filelogs {
                    if log.timestamp < ts + 3600 {
                        logs.push(log);
                    } else {
                        if logs.len() > 0 {
                            filelogs_by_hour.push((hour.to_string(), logs));
                            logs = Vec::new();
                        }
                        let dt = DateTime::from_timestamp(log.timestamp, 0).unwrap();
                        let lt = dt.with_timezone(&chrono::Local);
                        hour = lt.hour();
                        ts = local_time.with_hour(hour).unwrap().with_minute(0).unwrap().with_second(0).unwrap().timestamp();
                        logs.push(log);
                    }
                }
                if logs.len() > 0 {
                    filelogs_by_hour.push((hour.to_string(), logs));
                }

                Ok(filelogs_by_hour)
            },
            _ => {
                Err(anyhow!("Not implemented yet"))
            }
        }
    }

    pub async fn get_file_info(&self, file_id: i64) -> Result<FileInfo> {
        debug!("get_file_info: file_id={}", file_id);

        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await.expect("db pool is not set");

        let result = sqlx::query_as::<_, (i64, String)>(
            r#"
            SELECT id, path
            FROM file_info
            WHERE id = ?
            "#
        )
        .bind(file_id)
        .fetch_one(&pool)
        .await;

        match result {
            Ok((id, path)) => {
                Ok(FileInfo {
                    id: id,
                    path: path,
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

#[derive(Debug, serde::Serialize)]
pub struct FileLog {
    pub id: i64,
    pub event_id: i64,
    pub timestamp: i64,
    pub date: String,
    pub info_id: i64,
    pub kind: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct FileInfo {
    pub id: i64,
    pub path: String,
}

#[tauri::command]
pub async fn list_file_logs(file_log: State<'_, FilelogPlugin>, timestamp: i64, interval: db::Interval) -> Result<Vec<(String, Vec<FileLog>)>, String> {
    file_log.list_file_logs(timestamp, interval).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_file_info(file_log: State<'_, FilelogPlugin>, file_id: i64) -> Result<FileInfo, String> {
    file_log.get_file_info(file_id).await.map_err(|e| e.to_string())
}
