use anyhow::{anyhow, Result};
use log::{debug, error, info};
use std::{
    fmt,
    sync::{ Arc, Mutex},
};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, State, Wry,
};
use tokio::sync::mpsc;
use watchexec::Watchexec;
use watchexec_signals::Signal; 

use crate::plugins::{
    db,
    setting::SettingPlugin,
};

const KIND: &str = "file";
const WATCH_PATHSEST_SETTING: &str = "watch-pathset";

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
        
        let watch_pathset = self.watch_pathset()?;
        if watch_pathset.is_empty() {
            debug!("watch_pathset is empty");
            return Ok(());
        }

        let (tx, mut rx) = mpsc::channel::<Vec<FileEventInfo>>(64);

        let self_clone = self.clone();
        let _manager = tokio::spawn(async move {
            while let Some(infos) = rx.recv().await {
                debug!("Received file_event_infos: {:?}", infos);
                for info in infos.iter() {
                    debug!("filelog: {:?}", info);
                    match self_clone.insert_info(info).await {
                        Ok(id) => {
                            debug!("filelog: inserted id: {:?}", id);
                        },
                        Err(e) => {
                            error!("Error on insert filelog: {:?}", e);
                        },
                    }
                }
            }
        });

        *self.running.lock().unwrap() = true;
        // let running = Arc::clone(&self.running);
        let self_clone = self.clone();
        let wx = Watchexec::new(move |mut action| {
            if !*self_clone.running.lock().unwrap() {
                action.quit();
                return action;
            }

            // if Ctrl-C is received, quit
            if action.signals().any(|sig| sig == Signal::Interrupt) {
                self_clone.stop();
                action.quit();
                return action;
            }

            let infos: Vec<FileEventInfo> = action.events.iter().filter_map(event_to_file_event_info).collect();
            tx.try_send(infos).unwrap();

            action
        }).unwrap();

        // watch the current directory
        wx.config.pathset(watch_pathset);

        tokio::spawn(async move {
            wx.main().await.unwrap().unwrap();
        });

        Ok(())
    }

    pub fn stop(&self) {
        debug!("filelog stop");
        *self.running.lock().unwrap() = false;
    }

    fn watch_pathset(&self) -> Result<Vec<String>> {
        // debug!("db_path");
        let setting = self.app.state::<SettingPlugin>();
        let watch_pathset = setting.get(WATCH_PATHSEST_SETTING)?
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .map(|s| s.split(',').map(|s| s.to_string()).collect::<Vec<String>>());
        if watch_pathset.is_none() {
            return Err(anyhow!("{} is not set", WATCH_PATHSEST_SETTING));
        }
        debug!("watch_pathset: {:?}", watch_pathset);

        Ok(watch_pathset.unwrap())
    }

    async fn insert_info(&self, info: &FileEventInfo) -> Result<i64> {
        let timestamp = chrono::Utc::now();

        let db = self.app.state::<db::ImmicDb>();
        let event_id = db.insert_eventlog(timestamp, KIND).await?;
        let pool = db.pool().await.expect("db pool is not set");

        // Search file_info by path
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT id
            FROM file_info
            WHERE path = ?
            "#
        )
        .bind(&info.path)
        .fetch_one(&pool)
        .await;

        let info_id = match result {
            Ok((id,)) => id,
            Err(_) => {
                // Insert file_info for new path
                let result = sqlx::query(
                    r#"
                    INSERT INTO file_info (path, file_type)
                    VALUES (?, ?)
                    "#
                )
                .bind(&info.path)
                .bind(&info.file_type.to_string())
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

    pub async fn list_file_logs(&self, date: &str) -> Result<Vec<FileLog>> {
        debug!("list_filelogs: date: {}", date);

        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await.expect("db pool is not set");

        let filelogs = sqlx::query_as::<_,
            (i64, i64, String, String, i64,
            i64, i64, Option<String>)
        >(
            r#"
            SELECT
            e.id, e.timestamp, e.date, e.kind, e.log_id,
            f.id, f.indo_id, f.kind
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

        debug!("list_filelogs: filelogs: {:?}", filelogs);
        
        Ok(filelogs)
    }

    pub async fn get_file_info(&self, file_id: i64) -> Result<FileInfo> {
        debug!("get_file_info: file_id={}", file_id);

        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await.expect("db pool is not set");

        let result = sqlx::query_as::<_, (i64, String, Option<String>)>(
            r#"
            SELECT id, path, file_type
            FROM file_info
            WHERE id = ?
            "#
        )
        .bind(file_id)
        .fetch_one(&pool)
        .await;

        match result {
            Ok((id, path, file_type)) => {
                Ok(FileInfo {
                    id: id,
                    path: path,
                    file_type: file_type,
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
    path: String,
    file_type: FileType,
}

#[derive(Debug,PartialEq)]
enum FileKind {
    Access,
    Create,
    Modify,
    Remove,
}

impl fmt::Display for FileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileKind::Access => write!(f, "access"),
            FileKind::Create => write!(f, "create"),
            FileKind::Modify => write!(f, "modify"),
            FileKind::Remove => write!(f, "remove"),
        }
    }
}

#[derive(Debug,PartialEq)]
enum FileType {
    File,
    Dir,
    Symlink,
    Other,
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileType::File => write!(f, "file"),
            FileType::Dir => write!(f, "dir"),
            FileType::Symlink => write!(f, "symlink"),
            _ => write!(f, ""),
        }
    }
}

fn event_to_file_event_info(event: &watchexec_events::Event) -> Option<FileEventInfo> {
    let mut source = None;
    let mut kind: Option<FileKind> = None;
    let mut p: Option<String> = None;
    let mut ft: Option<FileType> = None;
    for tag in event.tags.iter() {
        match tag {
            watchexec_events::Tag::Source(s) => source = Some(s),
            watchexec_events::Tag::FileEventKind(k) => {
                match k {
                    watchexec_events::filekind::FileEventKind::Access(_) => kind = Some(FileKind::Access),
                    watchexec_events::filekind::FileEventKind::Create(_) => kind = Some(FileKind::Create),
                    watchexec_events::filekind::FileEventKind::Modify(_) => kind = Some(FileKind::Modify),
                    watchexec_events::filekind::FileEventKind::Remove(_) => kind = Some(FileKind::Remove),
                    _ => {
                        error!("Unknown file event kind: {:?}", k)
                    }
                }
            },
            watchexec_events::Tag::Path { path, file_type } => {
                p = Some(path.to_string_lossy().to_string());
                match file_type {
                    Some(watchexec_events::FileType::File) => ft = Some(FileType::File),
                    Some(watchexec_events::FileType::Dir) => ft = Some(FileType::Dir),
                    Some(watchexec_events::FileType::Symlink) => ft = Some(FileType::Symlink),
                    Some(watchexec_events::FileType::Other) => ft = Some(FileType::Other),
                    _ => {
                        error!("Unknown file type: {:?}", file_type)
                    }
                }
            },
            _ => {}
        }
    }
    if let (Some(watchexec_events::Source::Filesystem), Some(kind), Some(path)) = (source, kind, p) {
        Some(FileEventInfo {
            kind,
            path,
            file_type: ft.unwrap_or(FileType::Other),
        })
    } else {
        None
    }
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
    pub file_type: Option<String>,
}

#[tauri::command]
pub async fn list_file_logs(file_log: State<'_, FilelogPlugin>, date: String) -> Result<Vec<FileLog>, String> {
    file_log.list_file_logs(&date).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_file_info(file_log: State<'_, FilelogPlugin>, file_id: i64) -> Result<FileInfo, String> {
    file_log.get_file_info(file_id).await.map_err(|e| e.to_string())
}
