use active_win_pos_rs::get_active_window;
use anyhow::Result;
use log::debug;
use std::sync::{Arc, Mutex};
use sqlx;

use crate::app::db;
use crate::plugins::Plugin;

const KIND: &str = "application";

pub struct ApplicationPlugin {
    running: Arc<Mutex<bool>>,
}

impl ApplicationPlugin {
    pub fn new() -> ApplicationPlugin {
        ApplicationPlugin {
            running: Arc::new(Mutex::new(false)),
        }
    }
}

impl Plugin for ApplicationPlugin {
    fn start(&mut self) {
        *self.running.lock().unwrap() = true;
        let running = Arc::clone(&self.running);
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        let app = self;
        tokio::spawn(async move {
            let mut last_info = None;
            let mut last_id = -1;
            loop {
                if !*running.lock().unwrap() {
                    break;
                }
                interval.tick().await;

                let info = check_application().await;

                // check if the last info is the same as the current info
                if info == last_info {
                    debug!("check_application: same as last info");
                    if let Err(e) = insert_ref(last_id).await {
                        debug!("check_application: Error on inserting ref: {:?}", e);
                    }
                    continue;
                }

                if let Some(info) = info {
                    debug!("check_application: {:?}", info);
                    let id = info.insert().await;
                    match id {
                        Ok(id) => {
                            last_info = Some(info);
                            last_id = id;
                        },
                        Err(e) => {
                            debug!("check_application: Error on inserting application_info: {:?}", e);
                        },
                    }
                }
            }
        });
    }

    fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
    }
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

impl WinInfo {
    async fn insert(&self) -> Result<i64> {
        let timestamp = chrono::Utc::now();
        let event_id = db::insert_eventlog(timestamp, KIND).await?;

        // Search application_info by path
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT id
            FROM application_info
            WHERE path = ?
            "#
        )
        .bind(&self.path)
        .fetch_one(db::pool())
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
                .bind(&self.path)
                .bind(&self.name)
                .execute(db::pool())
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
        .bind(self.process_id)
        .bind(&self.title)
        .bind(self.x)
        .bind(self.y)
        .bind(self.width)
        .bind(self.height)
        .execute(db::pool())
        .await?;

        // Update event_log with log_id
        let log_id = result.last_insert_rowid();
        db::update_eventlog_logid(event_id, log_id).await?;
        Ok(log_id)
    }
}

async fn insert_ref(ref_id: i64) -> Result<i64> {
    let timestamp = chrono::Utc::now();
    let event_id = db::insert_eventlog(timestamp, KIND).await?;

    let result = sqlx::query(
        r#"
        INSERT INTO application_log (event_id, ref_id)
        VALUES (?, ?)
        "#
    )
    .bind(event_id)
    .bind(ref_id)
    .execute(db::pool())
    .await?;

    // Update event_log with log_id
    let log_id = result.last_insert_rowid();
    db::update_eventlog_logid(event_id , log_id).await?;
    Ok(log_id)
}


// ApplicationLog

#[derive(Debug, serde::Serialize)]
pub struct ApplicationLog {
    pub id: i64,
    pub event_id: i64,
    pub timestamp: i64,
    pub date: String,
    pub info_id: i64,
    pub process_id: Option<i64>,
    pub title: Option<String>,
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub ref_id: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct ApplicationInfo {
    pub id: i64,
    pub path: String,
    pub name: Option<String>,
}

#[tauri::command]
pub async fn list_application_logs(date: String) -> Result<Vec<ApplicationLog>, String> {
    debug!("list_application_logs: date: {}", date);
    let application_logs = sqlx::query_as::<_,
      (i64, i64, String, String, i64,
       i64, i64, Option<i64>, Option<String>, Option<i64>, Option<i64>, Option<i64>, Option<i64>, Option<i64>)>(
        r#"
        SELECT
          e.id, e.timestamp, e.date, e.kind, e.log_id,
          a.id, a.info_id, a.process_id, a.title, a.x, a.y, a.width, a.height, a.ref_id
        FROM event_log e
        INNER JOIN application_log a ON e.log_id = a.id
        WHERE e.kind = ? AND e.date = ?
        ORDER BY e.timestamp
        "#
    )
    .bind(KIND)
    .bind(date)
    .fetch_all(db::pool())
    .await
    .unwrap_or(Vec::new())
    .iter()
    .map(|row| {
        let (event_id, timestamp, date, _, _,
             id, info_id, process_id, title, x, y, width, height, ref_id
            ) = row;
        ApplicationLog {
            id: *id,
            event_id: *event_id,
            timestamp: *timestamp,
            date: date.clone(),
            info_id: *info_id,
            process_id: *process_id,
            title: title.clone(),
            x: *x,
            y: *y,
            width: *width,
            height: *height,
            ref_id: *ref_id,
        }
    })
    .collect();
    debug!("list_applications: application_logs: {:?}", application_logs);
    Ok(application_logs)
}

#[tauri::command]
pub async fn get_application_info(app_id: i64) -> Result<ApplicationInfo, String> {
    debug!("get_application_info: app_id={}", app_id);

    sqlx::query_as::<_, (i64, String, Option<String>)>(
        r#"
        SELECT id, path, name
        FROM application_info
        WHERE id = ?
        "#
    )
    .bind(app_id)
    .fetch_one(db::pool())
    .await
    .map_or(Err("Not found".to_string()), |row| {
        let (id, path, name) = row;
        Ok(ApplicationInfo {
            id: id,
            path: path,
            name: name,
        })
    })
}
