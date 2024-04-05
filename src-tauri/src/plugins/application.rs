use active_win_pos_rs::get_active_window;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use log::{debug, error};
use std::sync::{Arc, Mutex};
use sqlx;
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, State, Wry,
};

use crate::plugins::db;

const KIND: &str = "application";

pub fn init() -> TauriPlugin<Wry> {
    plugin::Builder::new("application")
        .invoke_handler(tauri::generate_handler![
            list_application_logs,
            get_application_info,
        ])
        .setup(|app_handle| {
            debug!("application plugin setup");
            let application = ApplicationPlugin::new(app_handle);
            app_handle.manage(application);
            Ok(())
        })
        .build()
}

#[derive(Clone)]
pub struct ApplicationPlugin {
    app: AppHandle,
    running: Arc<Mutex<bool>>,
}

impl ApplicationPlugin {
    fn new(app: &AppHandle) -> Self {
        Self {
            app: app.clone(),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        debug!("ApplicationPlugin start");

        // Check pool
        let db = self.app.state::<db::ImmicDb>();
        db.pool().await.unwrap();

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

        *self.running.lock().unwrap() = true;
        // let running = Arc::clone(&self.running);
        // let app = self;
        let self_clone = self.clone();
        tokio::spawn(async move {
            let mut last_win_info = None;
            let mut last_id = -1;
            let mut last_info_id = -1;
            loop {
                if !*self_clone.running.lock().unwrap() {
                    break;
                }
                interval.tick().await;

                let win_info = check_application().await;

                // check if the last info is the same as the current info
                if win_info == last_win_info {
                    debug!("check_application: same as last info");
                    if let Err(e) = self_clone.insert_win_info_ref(last_id, last_info_id).await {
                        debug!("check_application: Error on inserting ref: {:?}", e);
                    }
                    continue;
                }

                if let Some(win_info) = win_info {
                    debug!("check_application: {:?}", win_info);
                    let ids = self_clone.insert_win_info(&win_info).await;
                    match ids {
                        Ok((id, info_id)) => {
                            last_win_info = Some(win_info);
                            last_id = id;
                            last_info_id = info_id;
                        },
                        Err(e) => {
                            debug!("check_application: Error on inserting application_info: {:?}", e);
                        },
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }

    async fn insert_win_info(&self, win_info: &WinInfo) -> Result<(i64, i64)> {
        let timestamp = Utc::now();

        // Insert event_log
        let db = self.app.state::<db::ImmicDb>();
        let event_id = db.insert_eventlog(timestamp, KIND).await?;

        // Search application_info by path
        let pool = db.pool().await.unwrap();
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT id
            FROM application_info
            WHERE path = ?
            "#
        )
        .bind(&win_info.path)
        .fetch_one(&pool)
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
                .bind(&win_info.path)
                .bind(&win_info.name)
                .execute(&pool)
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
        .bind(event_id)
        .bind(info_id)
        .bind(win_info.process_id)
        .bind(&win_info.title)
        .bind(win_info.x)
        .bind(win_info.y)
        .bind(win_info.width)
        .bind(win_info.height)
        .execute(&pool)
        .await?;

        // Update event_log with log_id
        let log_id = result.last_insert_rowid();
        db.update_eventlog_logid(event_id, log_id).await?;

        Ok((log_id, info_id))
    }

    async fn insert_win_info_ref(&self, ref_id: i64, info_id: i64) -> Result<i64> {
        let timestamp = Utc::now();

        // Insert event_log
        let db = self.app.state::<db::ImmicDb>();
        let event_id = db.insert_eventlog(timestamp, KIND).await?;

        let pool = db.pool().await.unwrap();
        let result = sqlx::query(
            r#"
            INSERT INTO application_log (event_id, info_id, ref_id)
            VALUES (?, ?, ?)
            "#
        )
        .bind(event_id)
        .bind(info_id)
        .bind(ref_id)
        .execute(&pool)
        .await?;

        // Update event_log with log_id
        let log_id = result.last_insert_rowid();
        db.update_eventlog_logid(event_id , log_id).await?;

        Ok(log_id)
    }

    pub async fn list_application_logs(&self, timestamp: i64, interval: db::Interval) -> Result<Vec<(String, Vec<ApplicationLog>)>> {
        let dt = DateTime::from_timestamp_millis(timestamp);
        if dt.is_none() {
            error!("Invalid timestamp: {}", timestamp);
            return Err(anyhow!("Invalid timestamp"));
        };
        let dt = dt.unwrap();

        let local_time = dt.with_timezone(&chrono::Local);
        let date = local_time.format("%Y%m%d").to_string();

        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await.unwrap();

        let application_logs: Vec<ApplicationLog> = sqlx::query_as::<_, (
            i64, i64, i64,
            i64, Option<i64>, Option<String>, Option<i64>, Option<i64>, Option<i64>, Option<i64>,
            i64, String, Option<String>,
        )>(
            r#"
            SELECT
            e.id, e.timestamp, e.timeframe,
            a.id, a.process_id, a.title, a.x, a.y, a.width, a.height,
            i.id, i.name, i.path
            FROM event_log e
            INNER JOIN application_log a ON e.log_id = a.id
            INNER JOIN application_info i ON a.info_id = i.id
            WHERE e.kind = "application" AND e.date = "20240405"
            ORDER BY e.timestamp
            "#
        )
        .bind(KIND)
        .bind(&date)
        .fetch_all(&pool)
        .await
        .unwrap_or(Vec::new())
        .iter()
        .map(|row| {
            let (
                event_id, timestamp, timeframe,
                id, process_id, title, x, y, width, height,
                info_id, name, path,
            ) = row;
            ApplicationLog {
                id: *id,
                event_id: *event_id,
                timestamp: *timestamp,
                timeframe: *timeframe,
                date: date.clone(),
                info_id: *info_id,
                name: name.clone(),
                path: path.clone(),
                process_id: *process_id,
                title: title.clone(),
                x: *x,
                y: *y,
                width: *width,
                height: *height,
            }
        })
        .collect();
        // debug!("list_applications: application_logs: {:?}", application_logs);

        db::partition_logs(application_logs, &local_time, interval)
    }

    pub async fn get_application_info(&self, app_id: i64) -> Result<ApplicationInfo> {
        debug!("get_application_info: app_id={}", app_id);

        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await.unwrap();
        let result = sqlx::query_as::<_, (i64, String, Option<String>)>(
            r#"
            SELECT id, path, name
            FROM application_info
            WHERE id = ?
            "#
        )
        .bind(app_id)
        .fetch_one(&pool)
        .await;
        
        match result {
            Ok((id, path, name)) => {
                Ok(ApplicationInfo {
                    id: id,
                    path: path,
                    name: name,
                })
            },
            Err(_) => {
                Err(anyhow!("Not found"))
            }
        }
    }
}

#[derive(Debug,PartialEq)]
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

#[derive(Debug, serde::Serialize)]
pub struct ApplicationLog {
    pub id: i64,
    pub event_id: i64,
    pub timestamp: i64,
    pub timeframe: i64,
    pub date: String,
    pub info_id: i64,
    pub name: String,
    pub path: Option<String>,
    pub process_id: Option<i64>,
    pub title: Option<String>,
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

impl db::Timestamp for ApplicationLog {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ApplicationInfo {
    pub id: i64,
    pub path: String,
    pub name: Option<String>,
}

#[tauri::command]
pub async fn list_application_logs(application: State<'_, ApplicationPlugin>, timestamp: i64, interval: db::Interval) -> Result<Vec<(String, Vec<ApplicationLog>)>, String> {
    application.list_application_logs(timestamp, interval).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_application_info(application: State<'_, ApplicationPlugin>, app_id: i64) -> Result<ApplicationInfo, String> {
    application.get_application_info(app_id).await.map_err(|e| e.to_string())
}
