use anyhow::Result;
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
        println!("filelog");

        let (tx, mut rx) = mpsc::channel::<Vec<FileInfo>>(32);

        let manager = tokio::spawn(async move {
            while let Some(infos) = rx.recv().await {
                println!("Received file_infos: {:?}", infos);
                for info in infos.iter() {
                    println!("filelog: {:?}", info);
                    match info.insert().await {
                        Ok(id) => {
                            println!("filelog: inserted id: {:?}", id);
                        },
                        Err(e) => {
                            eprintln!("Error on insert filelog: {:?}", e);
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

        // let (tx, mut rx) = mpsc::channel(256);

        // let mut watcher = notify::recommended_watcher(move |res: _| match res {
        //     Ok(event) => {
        //         let event: Event = event;

        //         // only handle newly created files
        //         if event.kind.is_create() {
        //             event.paths.iter().for_each(|path| {
        //                 let file_path = path.to_string_lossy();
        //                 // THIS PRINTS EVERYTIME I CREATE A FILE
        //                 println!("Created new file: {:?}", file_path);
        //                 if let Err(err) = tx.blocking_send(file_path.to_string()) {
        //                     eprintln!("Error handling file {:#?}", err);
        //                 };
        //             });
        //         }
        //     }
        //     Err(e) => eprintln!("watch error: {:?}", e),
        // })
        // .ok()
        // .unwrap();

        // let path = Path::new("D:\\Works\\src\\github.com\\stn\\");
        // println!("Watching path: {:?}", path);
        // watcher
        //     .watch(path, RecursiveMode::Recursive)
        //     .unwrap();

        // tokio::task::spawn(async move {
        //     while let Some(file) = rx.recv().await {
        //         // NOTHING HERE
        //         println!("Received file: {:?}", file);
        //     }
        // });
    }

    fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
    }
}

// async fn check_application() {
//     println!("application");
//     match get_active_window() {
//         Ok(win) => {
//             // println!("active_window: {:?}", win);
//             let log = ApplicationLog {
//                 process_id: win.process_id as i64,
//                 name: win.app_name,
//                 title: win.title,
//                 x: win.position.x as i64,
//                 y: win.position.y as i64,
//                 width: win.position.width as i64,
//                 height: win.position.height as i64,
//             };
//             println!("application log: {:?}", log);
//             insert_application_log(log).await.unwrap_or_else(|e| {
//                 println!("check_application: Error on insert_application_log: {:?}", e);
//             });
//         },
//         Err(e) => {
//             println!("active_window: {:?}", e);
//         },
//     }
// }

// async fn insert_application_log(log: ApplicationLog) -> Result<()> {
//     let pool = db::pool();
//     sqlx::query(
//         "INSERT INTO application (eventId, kind, processId, name, title, x, y, width, height) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
//     )
//         .bind(1)
//         .bind("active")
//         .bind(log.process_id)
//         .bind(log.name)
//         .bind(log.title)
//         .bind(log.x)
//         .bind(log.y)
//         .bind(log.width)
//         .bind(log.height)
//         .execute(pool).await?;
//     Ok(())
// }

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
            INSERT INTO file (event_id, kind, path, file_type)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(event_id)
        .bind(&self.kind.to_string())
        .bind(&self.path)
        .bind(&self.file_type.to_string())
        .execute(db::pool())
        .await?;

        Ok(result.last_insert_rowid())
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
                        eprintln!("Unknown file event kind: {:?}", k)
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
                        eprintln!("Unknown file type: {:?}", file_type)
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
    println!("list_filelogs: date: {}", date);
    let filelogs = sqlx::query_as::<_,
      (i64, i64, String, String,
       i64, i64, Option<String>, Option<String>, Option<String>)
    >(
        r#"
        SELECT
          e.id, e.timestamp, e.date, e.kind,
          f.id, f.event_id, f.kind, f.path, f.file_type
        FROM event e
        INNER JOIN file f ON e.id = f.event_id
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
    println!("list_filelogs: filelogs: {:?}", filelogs);
    Ok(filelogs)
}
