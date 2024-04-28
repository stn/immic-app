use anyhow::{anyhow, Context as _, Result};
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
use tokio::{
    sync::mpsc,
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::{app::event::{emit_event_to_info, ImmicEvent}, plugins::{
    db,
    setting::SettingPlugin,
}};

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
            // get_file_info,
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
    task_handles: Arc<Mutex<Option<Vec<JoinHandle<()>>>>>,
    cancel_token: Arc<Mutex<Option<CancellationToken>>>,
}

impl FilelogPlugin {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            task_handles: Arc::new(Mutex::new(None)),
            cancel_token: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self) -> Result<()> {
        info!("FilelogPlugin start");
        
        // // Check if setting and db are ready
        // let _setting = self.app.state::<SettingPlugin>();
        // let _db = self.app.state::<db::ImmicDb>();

        let watch_pathset = self.watch_pathset()?;
        if watch_pathset.is_empty() {
            debug!("watch_pathset is empty");
            return Ok(());
        }

        let cancel_token = CancellationToken::new();
        self.cancel_token.lock().unwrap().replace(cancel_token.clone());

        let mut task_handles = Vec::new();
        for path in watch_pathset {
            if !path.exists() {
                error!("watch path not found: {:?}", path);
                continue;
            }
            let self_clone = self.clone();
            let token_clone = cancel_token.clone();
            task_handles.push(tokio::spawn(async move {
                tokio::select! {
                    _ = async move {
                        self_clone.async_watch(path).await.unwrap_or_else(|e| {
                            error!("Error on async_watch: {:?}", e);
                        });
                    } => {},
                    _ = token_clone.cancelled() => {}
                }
            }));
        }
        self.task_handles.lock().unwrap().replace(task_handles);

        Ok(())
    }

    async fn async_watch(&self, path: PathBuf) -> Result<()> {
        let (mut debouncer, mut rx) = self.async_watcher()?;

        debouncer.watcher().watch(&path, RecursiveMode::Recursive)?;

        while let Some(infos) = rx.recv().await {
            for mut info in infos {
                debug!("file event info: {:?}", info);

                info.watch_dir = Some(path.clone());

                let file_log = FileLog {
                    id: 0,  // dummy
                    timestamp: Utc::now().timestamp(),
                    date: "".to_string(),  // dummy
                    path: info.path.to_string_lossy().to_string(),
                    kind: Some(info.kind.to_string()),
                    watch_dir: info.watch_dir.map(|p| p.to_string_lossy().to_string()),
                };

                if let Err(e) = self.maybe_insert_file_log_and_emit(file_log).await {
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
                    let infos = debounce_events.into_iter()
                        .map(|de| {
                            let info = FileEventInfo::from(de.event);
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
                    errors.into_iter().for_each(|e| {
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

        Ok(watch_pathset.unwrap_or(Vec::new()))
    }

    pub async fn stop(&self) {
        let cancel_token = self.cancel_token.lock().unwrap().take();
        cancel_token.map(|t| t.cancel());

        let task_handles = self.task_handles.lock().unwrap().take();
        if let Some(task_handles) = task_handles {
            for handle in task_handles.into_iter() {
                handle.await.unwrap_or_else(|e| {
                    error!("Error on stopping filelog task: {:?}", e);
                });
            }
        }

        debug!("FilelogPlugin stopped");
    }

    // async fn maybe_insert_info(&self, info: FileEventInfo) -> Result<Option<i64>> {
    //     if self.check_debounce(&info).await? {
    //         debug!("filelog: debounced!");
    //         return Ok(None);
    //     }
    //     let id = self.insert_info(info).await?;
    //     Ok(Some(id))
    // }

    async fn maybe_insert_file_log_and_emit(&self, log: FileLog) -> Result<Option<i64>> {
        if self.check_debounce(&log).await? {
            debug!("filelog: debounced!");
            return Ok(None);
        }
        let id = self.insert_file_log(&log).await?;

        // send event
        let event = ImmicEvent::File(log);
        emit_event_to_info(&self.app, event).context("Failed to emit event")?;

        Ok(Some(id))
    }

    async fn insert_file_log(&self, log: &FileLog) -> Result<i64> {
        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await?;

        self.insert_file_log_with(&pool, log).await
    }

    // async fn insert_info(&self, info: FileEventInfo) -> Result<i64> {
    //     let timestamp = Utc::now();

    //     let db = self.app.state::<db::ImmicDb>();
    //     let pool = db.pool().await?;

    //     let file_log = FileLog {
    //         id: 0,  // dummy
    //         timestamp: timestamp.timestamp(),
    //         date: "".to_string(),  // dummy
    //         path: info.path.to_string_lossy().to_string(),
    //         kind: Some(info.kind.to_string()),
    //         watch_dir: info.watch_dir.map(|p| p.to_string_lossy().to_string()),
    //     };

    //     self.insert_file_log_with(&pool, file_log).await
    // }

    pub async fn insert_file_log_with(&self, pool: &Pool<Sqlite>, log: &FileLog) -> Result<i64> {
        let timestamp = DateTime::from_timestamp(log.timestamp, 0).context("Invalid timestamp")?;

        let db = self.app.state::<db::ImmicDb>();
        let event_log = db.insert_eventlog_with(pool, &timestamp, KIND).await?;

        // file_info by path
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT id
            FROM file_info
            WHERE path = ?
            "#
        )
        .bind(&log.path)
        .fetch_one(pool)
        .await;
        
        let info_id = match result {
            Ok((id,)) => {
                sqlx::query(
                    r#"
                    UPDATE file_info
                    SET last_update = ?
                    WHERE id = ?
                    "#
                )
                .bind(timestamp.timestamp())
                .bind(id)
                .execute(pool)
                .await?;
                id
            },
            Err(sqlx::Error::RowNotFound) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO file_info (path, last_update)
                    VALUES (?, ?)
                    "#
                )
                .bind(&log.path)
                .bind(timestamp.timestamp())
                .execute(pool)
                .await?;
                result.last_insert_rowid()
            },
            Err(e) => {
                return Err(anyhow!("select from file_info error: {:?}", e));
            }
        };

        // watch_dir
        let watch_dir_id = match &log.watch_dir {
            Some(p) => {
                let result = sqlx::query_as::<_, (i64,)>(
                    r#"
                    SELECT id
                    FROM watch_dir
                    WHERE dir = ?
                    "#
                )
                .bind(p)
                .fetch_one(pool)
                .await;

                match result {
                    Ok((id,)) => Some(id),
                    Err(sqlx::Error::RowNotFound) => {
                        let result = sqlx::query(
                            r#"
                            INSERT INTO watch_dir (dir)
                            VALUES (?)
                            "#
                        )
                        .bind(p)
                        .execute(pool)
                        .await?;
                        Some(result.last_insert_rowid())
                    },
                    Err(e) => {
                        return Err(anyhow!("select from watch_dir error: {:?}", e));
                    }
                }
            },
            None => None,
        };

        // Insert file_log
        let result = sqlx::query(
            r#"
            INSERT INTO file_log (event_id, info_id, kind, watch_dir_id)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(event_log.id)
        .bind(info_id)
        .bind(&log.kind)
        .bind(watch_dir_id)
        .execute(pool)
        .await?;

        let log_id = result.last_insert_rowid();
        Ok(log_id)
    }

    async fn check_debounce(&self, log: &FileLog) -> Result<bool> {
        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await?;

        let result = sqlx::query_as::<_, (Option<i64>,)>(
            r#"
            SELECT last_update
            FROM file_info
            WHERE path = ?
            "#
        )
        .bind(&log.path)
        .fetch_one(&pool)
        .await;

        if let Ok((Some(last_update),)) = result {
            if log.timestamp - last_update < DEBOUNCE_THRESHOLD {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn list_file_logs_on(&self, date: &str) -> Result<Vec<FileLog>> {
        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await?;

        let mut rows = sqlx::query_as::<_, (
            i64,
            i64, Option<String>,
            String,
            Option<String>,
        )>(
            r#"
            SELECT
            e.timestamp,
            f.id, f.kind,
            i.path,
            w.dir
            FROM event_log e
            INNER JOIN file_log f ON e.id = f.event_id
            INNER JOIN file_info i ON f.info_id = i.id
            LEFT JOIN watch_dir w ON f.watch_dir_id = w.id
            WHERE e.kind = ? AND e.date = ?
            ORDER BY e.id
            "#
        )
        .bind(KIND)
        .bind(date)
        .fetch(&pool);

        let mut filelogs = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let (
                timestamp,
                id, kind,
                path,
                watch_dir,
            ) = row;
            filelogs.push(FileLog {
                id,
                timestamp,
                date: date.to_string(),
                path,
                kind,
                watch_dir,
            });
        }
        Ok(filelogs)
    }

    // pub async fn get_file_info(&self, file_id: i64) -> Result<FileInfo> {
    //     debug!("get_file_info: file_id={}", file_id);

    //     let db = self.app.state::<db::ImmicDb>();
    //     let pool = db.pool().await.expect("db pool is not set");

    //     let result = sqlx::query_as::<_, (i64, String, Option<i64>)>(
    //         r#"
    //         SELECT id, path, last_update
    //         FROM file_info
    //         WHERE id = ?
    //         "#
    //     )
    //     .bind(file_id)
    //     .fetch_one(&pool)
    //     .await;

    //     match result {
    //         Ok((id, path, last_update)) => {
    //             Ok(FileInfo {
    //                 id,
    //                 path,
    //                 last_update,
    //             })
    //         },
    //         Err(_) => {
    //             Err(anyhow::anyhow!("Not found"))
    //         }
    //     }
    // }
}

#[derive(Debug,PartialEq)]
struct FileEventInfo {
    kind: FileKind,
    path: PathBuf,
    watch_dir: Option<PathBuf>,
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

impl From<Event> for FileEventInfo {
    fn from(event: Event) -> Self {
        let kind = match event.kind {
            EventKind::Create(_) => FileKind::Create,
            EventKind::Modify(_) => FileKind::Modify,
            EventKind::Remove(_) => FileKind::Remove,
            _ => FileKind::Other,
        };

        // Rename event has two paths, but we only check the first one for now.
        let path = event.paths.into_iter().next().unwrap();

        Self {
            kind,
            path,
            watch_dir: None,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileLog {
    pub id: i64,
    pub timestamp: i64,
    pub date: String,
    pub path: String,
    pub kind: Option<String>,
    pub watch_dir: Option<String>,
}

impl db::Timestamp for FileLog {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

// #[derive(Debug, Serialize)]
// pub struct FileInfo {
//     pub id: i64,
//     pub path: String,
//     pub last_update: Option<i64>,
// }

#[tauri::command]
pub async fn list_file_logs_on(file_log: State<'_, FilelogPlugin>, date: String) -> Result<Vec<FileLog>, String> {
    file_log.list_file_logs_on(&date).await.map_err(|e| e.to_string())
}

// #[tauri::command]
// pub async fn get_file_info(file_log: State<'_, FilelogPlugin>, file_id: i64) -> Result<FileInfo, String> {
//     file_log.get_file_info(file_id).await.map_err(|e| e.to_string())
// }
