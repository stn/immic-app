use active_win_pos_rs::get_active_window;
use anyhow::Result;
use sqlx;
use tauri::App;

use crate::app::db;

#[derive(Debug)]
pub struct ApplicationLog {
    process_id: i64,
    name: String,
    title: String,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
}

pub struct ApplicationPlugin;

impl ApplicationPlugin {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&mut self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                check_application().await;
            }
        });
    }
}

async fn check_application() {
    println!("application");
    match get_active_window() {
        Ok(win) => {
            // println!("active_window: {:?}", win);
            let log = ApplicationLog {
                process_id: win.process_id as i64,
                name: win.app_name,
                title: win.title,
                x: win.position.x as i64,
                y: win.position.y as i64,
                width: win.position.width as i64,
                height: win.position.height as i64,
            };
            println!("application log: {:?}", log);
            insert_application_log(log).await.unwrap_or_else(|e| {
                println!("check_application: Error on insert_application_log: {:?}", e);
            });
        },
        Err(e) => {
            println!("active_window: {:?}", e);
        },
    }
}

async fn insert_application_log(log: ApplicationLog) -> Result<()> {
    let pool = db::pool();
    sqlx::query(
        "INSERT INTO application (eventId, kind, processId, name, title, x, y, width, height) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
        .bind(1)
        .bind("active")
        .bind(log.process_id)
        .bind(log.name)
        .bind(log.title)
        .bind(log.x)
        .bind(log.y)
        .bind(log.width)
        .bind(log.height)
        .execute(pool).await?;
    Ok(())
}
