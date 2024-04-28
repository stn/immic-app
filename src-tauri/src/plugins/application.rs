use active_win_pos_rs::get_active_window;
use anyhow::{Context as _, Result, ensure};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use log::{error,debug};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashSet, HashMap},
    path::MAIN_SEPARATOR,
    sync::{Arc, Mutex},
};
use sqlx::{
    Pool,
    sqlite::Sqlite,
};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, State, Wry,
};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    app::event::{
        ImmicEvent,
        emit_event_to_info,
    },
    plugins::{
        db,
        search::{HitsPerDay, SearchHit},
    },
};

pub const KIND: &str = "application";

#[cfg(target_os = "windows")]
static IGNORE_APPS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    vec![
        "LockApp.exe",
        "scrnsave.scr",
    ].iter().cloned().collect()
});

#[cfg(not(target_os = "windows"))]
static IGNORE_APPS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    vec![
    ].iter().cloned().collect()
});

pub fn init() -> TauriPlugin<Wry> {
    plugin::Builder::new("application")
        .invoke_handler(tauri::generate_handler![
            list_application_logs_on,
            // get_application_info,
            // search_for_title,
        ])
        .setup(|app| {
            debug!("application plugin setup");
            let application = ApplicationPlugin::new(app.clone());
            app.manage(application);
            Ok(())
        })
        .build()
}

#[derive(Clone)]
pub struct ApplicationPlugin {
    app: AppHandle,
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    cancel_token: Arc<Mutex<Option<CancellationToken>>>,
}

impl ApplicationPlugin {
    fn new(app: AppHandle) -> Self {
        Self {
            app,
            task_handle: Arc::new(Mutex::new(None)),
            cancel_token: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self) -> Result<()> {
        debug!("ApplicationPlugin start");

        let cancel_token = CancellationToken::new();
        self.cancel_token.lock().unwrap().replace(cancel_token.clone());

        let self_clone = self.clone();
        let task_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            let mut last_win_info = None;
            let mut last_id = -1;
            let mut last_info_id = -1;

            tokio::select! {
                _ = async move {
                    loop {
                        interval.tick().await;

                        let win_info = check_application().await;

                        if win_info.is_none() {
                            continue;
                        }

                        // check if the last info is the same as the current info
                        if win_info == last_win_info {
                            debug!("check_application: same as last info");
                            if let Err(e) = self_clone.insert_win_info_ref(last_id, last_info_id).await {
                                error!("check_application: Error on inserting ref: {:?}", e);
                            }
                            continue;
                        }

                        if let Some(win_info) = win_info {
                            debug!("check_application: {:?}", win_info);

                            let app_last_path = win_info.path.as_str().split(MAIN_SEPARATOR).last();
                            if app_last_path.is_none() {
                                error!("check_application: invalid path: {}", win_info.path);
                                continue;
                            }
                            let app_last_path = app_last_path.unwrap();
                            if IGNORE_APPS.contains(app_last_path) {
                                debug!("check_application: ignore app: {}", win_info.name);
                                continue;
                            }

                            let logs = self_clone.insert_application_log_and_emit(win_info.clone().into()).await;
                            match logs {
                                Ok(log) => {
                                    last_win_info.replace(win_info);
                                    last_id = log.id;
                                    last_info_id = log.info_id.unwrap();
                                },
                                Err(e) => {
                                    error!("check_application: Error on inserting application_info: {:?}", e);
                                },
                            }
                        }
                    }
                } => {},
                _ = cancel_token.cancelled() => {}
            }
        });
        self.task_handle.lock().unwrap().replace(task_handle);

        Ok(())
    }

    pub async fn stop(&self) {
        let cancel_token = self.cancel_token.lock().unwrap().take();
        cancel_token.map(|t| t.cancel());

        let task_handle = self.task_handle.lock().unwrap().take();
        if let Some(handle) = task_handle {
            handle.await.unwrap_or_else(|e| {
                error!("Error on stopping application task: {:?}", e);
            });
        }

        debug!("ApplicationPlugin stopped");
    }

    // async fn insert_win_info(&self, win_info: WinInfo) -> Result<(i64, i64)> {
    //     let timestamp = Utc::now();

    //     let db = self.app.state::<db::ImmicDb>();
    //     let pool = db.pool().await?;

    //     let application_log = ApplicationLog {
    //         id: 0,  // dummy
    //         timestamp: timestamp.timestamp(),
    //         date: "".to_string(), // dummy
    //         path: win_info.path,
    //         name: Some(win_info.name),
    //         process_id: Some(win_info.process_id),
    //         title: Some(win_info.title),
    //         x: Some(win_info.x),
    //         y: Some(win_info.y),
    //         width: Some(win_info.width),
    //         height: Some(win_info.height),
    //         ref_id: None,
    //     };

    //     self.insert_application_log_with(&pool, &application_log).await
    // }

    async fn insert_win_info_ref(&self, ref_id: i64, info_id: i64) -> Result<i64> {
        let timestamp = Utc::now();

        // Insert event_log
        let db = self.app.state::<db::ImmicDb>();
        let event_log = db.insert_eventlog(&timestamp, KIND).await?;

        let pool = db.pool().await?;
        let result = sqlx::query(
            r#"
            INSERT INTO application_log (event_id, info_id, ref_id)
            VALUES (?, ?, ?)
            "#
        )
        .bind(event_log.id)
        .bind(info_id)
        .bind(ref_id)
        .execute(&pool)
        .await?;

        let log_id = result.last_insert_rowid();
        Ok(log_id)
    }

    async fn insert_application_log_and_emit(&self, application_log: ApplicationLog) -> Result<ApplicationLog> {
        let result = self.insert_application_log(application_log).await?;

        // search for title
        let hits = self.search_for_title(&result).await?;

        // send event
        let event = ImmicEvent::Application(result.clone(), hits);
        emit_event_to_info(&self.app, event).context("Failed to emit event")?;

        Ok(result)
    }

    async fn insert_application_log(&self, application_log: ApplicationLog) -> Result<ApplicationLog> {
        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await?;

        self.insert_application_log_with(&pool, application_log).await
    }

    pub async fn insert_application_log_with(&self, pool: &Pool<Sqlite>, log: ApplicationLog) -> Result<ApplicationLog> {
        ensure!(log.ref_id.is_none(), "ref_id must be None");

        let timestamp = DateTime::from_timestamp(log.timestamp, 0).context("Invalid timestamp")?;

        // Insert event_log
        let db = self.app.state::<db::ImmicDb>();
        let event_log = db.insert_eventlog_with(pool, &timestamp, KIND).await?;

        // Search application_info by path
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT id
            FROM application_info
            WHERE path = ?
            "#
        )
        .bind(&log.path)
        .fetch_one(pool)
        .await;

        let info_id = match result {
            Ok((id,)) => id,
            Err(_) => {
                // Insert application_info for new path
                let result = sqlx::query(
                    r#"
                    INSERT INTO application_info (path, name)
                    VALUES (?, ?)
                    "#
                )
                .bind(&log.path)
                .bind(&log.name)
                .execute(pool)
                .await?;
                result.last_insert_rowid()
            }
        };

        let result = sqlx::query(
            r#"
            INSERT INTO application_log (event_id, info_id, process_id, title, x, y, width, height)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(event_log.id)
        .bind(info_id)
        .bind(log.process_id)
        .bind(&log.title)
        .bind(log.x)
        .bind(log.y)
        .bind(log.width)
        .bind(log.height)
        .execute(pool)
        .await?;

        let log_id = result.last_insert_rowid();

        let application_log = ApplicationLog {
            id: log_id,
            timestamp: log.timestamp,
            date: event_log.date,
            info_id: Some(info_id),
            path: log.path,
            name: log.name,
            process_id: log.process_id,
            title: log.title,
            x: log.x,
            y: log.y,
            width: log.width,
            height: log.height,
            ref_id: log.ref_id,
        };

        Ok(application_log)
    }

    pub async fn insert_application_log_ref_with(&self, pool: &Pool<Sqlite>, log: ApplicationLog, last_ids: &Option<(i64, i64)>) -> Result<i64> {
        ensure!(log.ref_id.is_some(), "ref_id must be Some");
        ensure!(last_ids.is_some(), "last_ids must be Some");

        let (ref_id, info_id) = last_ids.unwrap();

        let timestamp = DateTime::from_timestamp(log.timestamp, 0).context("Invalid timestamp")?;

        // Insert event_log
        let db = self.app.state::<db::ImmicDb>();
        let event_log = db.insert_eventlog_with(pool, &timestamp, KIND).await?;

        let result = sqlx::query(
            r#"
            INSERT INTO application_log (event_id, info_id, ref_id)
            VALUES (?, ?, ?)
            "#
        )
        .bind(event_log.id)
        .bind(info_id)
        .bind(ref_id)
        .execute(pool)
        .await?;

        let log_id = result.last_insert_rowid();
        Ok(log_id)
    }

    pub async fn list_application_logs_on(&self, date: &str) -> Result<Vec<ApplicationLog>> {
        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await?;

        let mut rows = sqlx::query_as::<_, (
            i64,
            i64, Option<i64>, Option<String>, Option<i64>, Option<i64>, Option<i64>, Option<i64>, Option<i64>,
            i64, String, Option<String>,
        )>(
            r#"
            SELECT
            e.timestamp,
            a.id,
            coalesce(a.process_id, a0.process_id) as process_id,
            coalesce(a.title, a0.title) as title,
            coalesce(a.x, a0.x) as x,
            coalesce(a.y, a0.y) as y,
            coalesce(a.width, a0.width) as width,
            coalesce(a.height, a0.height) as height,
            a.ref_id,
            i.id, i.path, i.name
            FROM event_log e
            INNER JOIN application_log a ON e.id = a.event_id
            INNER JOIN application_info i ON a.info_id = i.id
            LEFT JOIN application_log a0 ON a.ref_id = a0.id
            WHERE e.kind = ? AND e.date = ?
            ORDER BY e.id
            "#
        )
        .bind(KIND)
        .bind(date)
        .fetch(&pool);

        let mut application_logs = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let (
                timestamp,
                id, process_id, title, x, y, width, height, ref_id,
                info_id, path, name,
            ) = row;
            application_logs.push(ApplicationLog {
                id,
                timestamp,
                date: date.to_string(),
                info_id: Some(info_id),
                path,
                name,
                process_id,
                title,
                x,
                y,
                width,
                height,
                ref_id,
            });
        }
        Ok(application_logs)
    }

    // pub async fn get_application_info(&self, app_id: i64) -> Result<ApplicationInfo> {
    //     debug!("get_application_info: app_id={}", app_id);

    //     let db = self.app.state::<db::ImmicDb>();
    //     let pool = db.pool().await.unwrap();
    //     let result = sqlx::query_as::<_, (i64, String, Option<String>)>(
    //         r#"
    //         SELECT id, path, name
    //         FROM application_info
    //         WHERE id = ?
    //         "#
    //     )
    //     .bind(app_id)
    //     .fetch_one(&pool)
    //     .await;
        
    //     match result {
    //         Ok((id, path, name)) => {
    //             Ok(ApplicationInfo {
    //                 id: id,
    //                 path: path,
    //                 name: name,
    //             })
    //         },
    //         Err(_) => {
    //             Err(anyhow!("Not found"))
    //         }
    //     }
    // }

    pub async fn search_for_title(&self, log: &ApplicationLog) -> Result<Vec<HitsPerDay>> {
        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await?;

        let mut rows = sqlx::query_as::<_, (
            i64,
            i64, String,
        )>(
            r#"
            SELECT
                a.id,
                e.timestamp, e.date
            FROM application_log a
            INNER JOIN event_log e
                ON a.info_id = ?
                AND a.title = ?
                AND a.event_id = e.id
            ORDER BY e.timestamp
            "#
        )
        .bind(log.info_id)
        .bind(&log.title)
        .fetch(&pool);

        let mut hits: HashMap<String, HitsPerDay> = HashMap::new();
        while let Some((id, timestamp, date)) = rows.try_next().await? {
            hits.entry(date.clone())
                .and_modify(|h| {
                    h.count += 1;
                    h.application_title_count = h.application_title_count.map(|c| c + 1);
                    h.application_title_hits.as_mut().map(|hits| {
                        hits.push(SearchHit {
                            id,
                            timestamp,
                        });
                    });
                })
                .or_insert_with(|| {
                    let mut h = HitsPerDay::default();
                    h.date = date;
                    h.count = 1;
                    h.application_title_count = Some(1);
                    h.application_title_hits = Some(vec![SearchHit {
                        id,
                        timestamp,
                    }]);
                    h
                });
        }

        let mut hits: Vec<HitsPerDay> = hits
            .into_iter()
            .map(|(_date, h)| h)
            .collect();
        hits.sort_by(|a, b| a.date.cmp(&b.date).reverse());

        Ok(hits)
    }

}

#[derive(Clone,Debug,PartialEq)]
struct WinInfo {
    process_id: i64,
    path: String,
    name: String,
    title: String,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
}

async fn check_application() -> Option<WinInfo> {
    debug!("check_application");
    match get_active_window() {
        Ok(win) => {
            debug!("active_window: {:?}", win);
            let info = WinInfo {
                process_id: win.process_id as i64,
                path: win.process_path.to_string_lossy().to_string(),
                name: win.app_name,
                title: win.title,
                x: win.position.x as i64,
                y: win.position.y as i64,
                width: win.position.width as i64,
                height: win.position.height as i64,
            };
            Some(info)
        },
        Err(_) => {
            None
        }
    }
}


// ApplicationLog

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationLog {
    pub id: i64,
    pub timestamp: i64,
    pub date: String,
    pub info_id: Option<i64>,
    pub path: String,
    pub name: Option<String>,
    pub process_id: Option<i64>,
    pub title: Option<String>,
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub ref_id: Option<i64>,
}

impl From<WinInfo> for ApplicationLog {
    fn from(win_info: WinInfo) -> Self {
        ApplicationLog {
            id: 0,  // dummy
            timestamp: Utc::now().timestamp(),
            date: "".to_string(), // dummy
            info_id: None,
            path: win_info.path,
            name: Some(win_info.name),
            process_id: Some(win_info.process_id),
            title: Some(win_info.title),
            x: Some(win_info.x),
            y: Some(win_info.y),
            width: Some(win_info.width),
            height: Some(win_info.height),
            ref_id: None,
        }
    }
}

impl db::Timestamp for ApplicationLog {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

// #[derive(Debug, Serialize)]
// pub struct ApplicationInfo {
//     pub id: i64,
//     pub path: String,
//     pub name: Option<String>,
// }

#[tauri::command]
pub async fn list_application_logs_on(application: State<'_, ApplicationPlugin>, date: String) -> Result<Vec<ApplicationLog>, String> {
    application.list_application_logs_on(&date).await.map_err(|e| e.to_string())
}

// #[tauri::command]
// pub async fn get_application_info(application: State<'_, ApplicationPlugin>, app_id: i64) -> Result<ApplicationInfo, String> {
//     application.get_application_info(app_id).await.map_err(|e| e.to_string())
// }

// #[tauri::command]
// pub async fn search_for_title(application: State<'_, ApplicationPlugin>, log: ApplicationLog) -> Result<Vec<SearchHit>, String> {
//     application.search_for_title(&log).await.map_err(|e| e.to_string())
// }
