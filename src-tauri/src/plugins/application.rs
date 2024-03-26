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

async fn check_application() -> Option<ApplicationInfo> {
    debug!("check_application");
    match get_active_window() {
        Ok(win) => {
            debug!("active_window: {:?}", win);
            let info = ApplicationInfo {
                process_id: win.process_id as i64,
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
struct ApplicationInfo {
    process_id: i64,
    name: String,
    title: String,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
}

impl ApplicationInfo {
    async fn insert(&self) -> Result<i64> {
        let timestamp = chrono::Utc::now();
        let event_id = db::insert_eventlog(timestamp, KIND).await?;

        let result = sqlx::query(
            r#"
            INSERT INTO application_log (event_id, process_id, name, title, x, y, width, height)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(event_id)
        .bind(self.process_id)
        .bind(&self.name)
        .bind(&self.title)
        .bind(self.x)
        .bind(self.y)
        .bind(self.width)
        .bind(self.height)
        .execute(db::pool())
        .await?;

        Ok(result.last_insert_rowid())
    }
}

async fn insert_ref(id: i64) -> Result<i64> {
    let timestamp = chrono::Utc::now();
    let event_id = db::insert_eventlog(timestamp, KIND).await?;

    let result = sqlx::query(
        r#"
        INSERT INTO application_log (event_id, ref_id)
        VALUES (?, ?)
        "#
    )
    .bind(event_id)
    .bind(id)
    .execute(db::pool())
    .await?;

    Ok(result.last_insert_rowid())
}


// ApplicationLog

#[derive(Debug, serde::Serialize)]
pub struct ApplicationLog {
    pub id: i64,
    pub event_id: i64,
    pub timestamp: i64,
    pub date: String,
    pub process_id: Option<i64>,
    pub name: Option<String>,
    pub title: Option<String>,
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub ref_id: Option<i64>,
}

#[tauri::command]
pub async fn list_applications(date: String) -> Result<Vec<ApplicationLog>, String> {
    debug!("list_applications: date: {}", date);
    let application_logs = sqlx::query_as::<_,
      (i64, i64, String, String, i64, i64, Option<i64>, Option<String>, Option<String>, Option<i64>, Option<i64>, Option<i64>, Option<i64>, Option<i64>)>(
        r#"
        SELECT e.id, e.timestamp, e.date, e.kind, a.id, a.event_id, a.process_id, a.name, a.title, a.x, a.y, a.width, a.height, a.ref_id
        FROM event_log e
        INNER JOIN application_log a ON e.id = a.event_id
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
        let (event_id, timestamp, date, _, id, _, process_id, name, title, x, y, width, height, ref_id) = row;
        ApplicationLog {
            id: *id,
            event_id: *event_id,
            timestamp: *timestamp,
            date: date.clone(),
            process_id: *process_id,
            name: name.clone(),
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
