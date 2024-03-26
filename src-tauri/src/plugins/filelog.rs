use anyhow::Result;
use log::{debug, error};
use std::fmt;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use watchexec::Watchexec;

use crate::app::db;
use crate::plugins::Plugin;

const KIND: &str = "file";

pub struct FilelogPlugin {
    running: Arc<Mutex<bool>>,
}

impl FilelogPlugin {
    pub fn new() -> Self {
        Self {
            running: Arc::new(Mutex::new(false)),
        }
    }
}

impl Plugin for FilelogPlugin {
    fn start(&mut self) {
        debug!("filelog");

        let (tx, mut rx) = mpsc::channel::<Vec<FileInfo>>(32);

        let manager = tokio::spawn(async move {
            while let Some(infos) = rx.recv().await {
                debug!("Received file_infos: {:?}", infos);
                for info in infos.iter() {
                    debug!("filelog: {:?}", info);
                    match info.insert().await {
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
        let running = Arc::clone(&self.running);
        let wx = Watchexec::new(move |mut action| {
            if !*running.lock().unwrap() {
                action.quit();
                return action;
            }

            let infos: Vec<FileInfo> = action.events.iter().filter_map(event_to_file_info).collect();
            tx.try_send(infos).unwrap();

            // // if Ctrl-C is received, quit
            // if action.signals().any(|sig| sig == Signal::Interrupt) {
            //     action.quit();
            // }

            action
        }).unwrap();

        // watch the current directory
        wx.config.pathset(["f:\\"]);

        tokio::spawn(async move {
            wx.main().await.unwrap().unwrap();
        });
    }

    fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
    }
}

#[derive(Debug,PartialEq)]
struct FileInfo {
    kind: FileKind,
    path: String,
    file_type: FileType,
}

impl FileInfo {
    async fn insert(&self) -> Result<i64> {
        let timestamp = chrono::Utc::now();
        let event_id = db::insert_eventlog(timestamp, KIND).await?;

        let result = sqlx::query(
            r#"
            INSERT INTO file_log (event_id, kind, path, file_type)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(event_id)
        .bind(&self.kind.to_string())
        .bind(&self.path)
        .bind(&self.file_type.to_string())
        .execute(db::pool())
        .await?;

        // Update event_log with log_id
        let log_id = result.last_insert_rowid();
        db::update_eventlog_logid(event_id , log_id).await?;
        Ok(log_id)
    }
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

fn event_to_file_info(event: &watchexec_events::Event) -> Option<FileInfo> {
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
        Some(FileInfo {
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
    pub kind: Option<String>,
    pub path: Option<String>,
    pub file_type: Option<String>,
}

#[tauri::command]
pub async fn list_filelogs(date: String) -> Result<Vec<FileLog>, String> {
    debug!("list_filelogs: date: {}", date);
    let filelogs = sqlx::query_as::<_,
      (i64, i64, String, String,
       i64, i64, Option<String>, Option<String>, Option<String>)
    >(
        r#"
        SELECT
          e.id, e.timestamp, e.date, e.kind, e.log_id,
          f.id, f.kind, f.path, f.file_type
        FROM event_log e
        INNER JOIN file_log f ON e.log_id = f.id
        WHERE e.kind = ? AND e.date = ?
        ORDER BY event_id
        "#
    )
    .bind(KIND)
    .bind(date)
    .fetch_all(db::pool())
    .await
    .unwrap_or(Vec::new())
    .iter()
    .map(|row| {
        let (event_id, timestamp, date, _,
             id, _, kind, path, file_type,
        ) = row;
        FileLog {
            id: *id,
            event_id: *event_id,
            timestamp: *timestamp,
            date: date.clone(),
            kind: kind.clone(),
            path: path.clone(),
            file_type: file_type.clone(),
        }
    })
    .collect();
    debug!("list_filelogs: filelogs: {:?}", filelogs);
    Ok(filelogs)
}
