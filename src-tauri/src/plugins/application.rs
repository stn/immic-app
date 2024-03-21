use active_win_pos_rs::get_active_window;
use anyhow::{bail, Result};
use std::sync::{Arc, Mutex};
use sqlx;

use crate::app::db;
use crate::plugins::Plugin;

#[derive(Debug)]
pub struct ApplicationLog {
    id: i64,
    event_id: i64,
    timestamp: i64,
    date: String,
    process_id: i64,
    name: String,
    title: String,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
}

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
                    println!("check_application: same as last info");
                    if let Err(e) = insert_ref(last_id).await {
                        println!("check_application: Error on inserting ref: {:?}", e);
                    }
                    continue;
                }

                if let Some(info) = info {
                    println!("check_application: {:?}", info);
                    let id = info.insert().await;
                    match id {
                        Ok(id) => {
                            last_info = Some(info);
                            last_id = id;
                        },
                        Err(e) => {
                            println!("check_application: Error on inserting application_info: {:?}", e);
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
    println!("check_application");
    match get_active_window() {
        Ok(win) => {
            // println!("active_window: {:?}", win);
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
        let event_id = db::insert_eventlog(timestamp, "application").await?;

        let result = sqlx::query(
            r#"
            INSERT INTO application (event_id, process_id, name, title, x, y, width, height)
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
    let event_id = db::insert_eventlog(timestamp, "application").await?;

    let result = sqlx::query(
        r#"
        INSERT INTO application (event_id, ref_id)
        VALUES (?, ?)
        "#
    )
    .bind(event_id)
    .bind(id)
    .execute(db::pool())
    .await?;

    Ok(result.last_insert_rowid())
}
