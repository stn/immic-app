use anyhow::{anyhow, Context as _, Result};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use image::RgbaImage;
use once_cell::sync::Lazy;
use regex::Regex;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use sqlx::{
    Pool,
    sqlite::Sqlite,
};
use tauri::{
    http, plugin::{self, TauriPlugin}, AppHandle, Manager, State, Wry};
use xcap::Monitor;

use crate::plugins::{
    db,
    setting::SettingPlugin,
};

const KIND: &str = "screenshot";

const DATA_DIR_SETTING: &str = "data-dir";
const SCREENSHOT_DIR: &str = "screenshot";

pub fn init() -> TauriPlugin<Wry> {
    plugin::Builder::new("screenshot")
        .invoke_handler(tauri::generate_handler![
            list_screenshot_logs_on,
        ])
        .setup(|app_handle| {
            debug!("screenshot plugin setup");
            let screen = ScreenshotPlugin::new(app_handle.clone());
            app_handle.manage(screen);
            Ok(())
        })
        .register_uri_scheme_protocol(
            "iss",
             move |app, request| {
                handle_iss_protocol(&app, &request)
            }
        )
        .build()
}

#[derive(Clone)]
pub struct ScreenshotPlugin {
    app: AppHandle,
    image_dir: Arc<Mutex<Option<PathBuf>>>,
    running: Arc<Mutex<bool>>,
}

impl ScreenshotPlugin {
    fn new(app: AppHandle) -> Self {
        Self {
            app,
            image_dir: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&self) -> Result<()> {
        debug!("ScreenshotPlugin start");

        let image_dir = image_base_dir(&self.app);
        if let Err(e) = image_dir {
            error!("failed to get image_dir: {}", e);
            return Err(e);
        }
        let image_dir = image_dir.unwrap();
        debug!("image_dir: {:?}", image_dir);
        self.image_dir.lock().unwrap().replace(image_dir);

        // if self.image_dir.is_none() {
        //     return Err(anyhow!("image_dir is not set"));
        // }

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        // let image_dir = self.image_dir.unwrap().clone();
        *self.running.lock().unwrap() = true;
        let self_clone = self.clone();
        tokio::spawn(async move {
            loop {
                if !*self_clone.running.lock().unwrap() {
                    break;
                }
                interval.tick().await;
                self_clone.take_screenshot().await.unwrap_or_else(|e| {
                    error!("Error on taking screenshot: {:?}", e);
                });
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }

    async fn take_screenshot(&self) -> Result<()> {
        debug!("screenshot");
        let monitors = Monitor::all().unwrap();

        for monitor in monitors {
            let screenshot = Screenshot {
                monitor: monitor.id() as i64,
                timestamp: chrono::Utc::now(),
                image: monitor.capture_image()?,
            };
            if is_blank(&screenshot.image) {
                debug!("Blank screen: monitor: {}", screenshot.monitor);
                break;
            }
            self.save_screenshot(&screenshot).await?;
            self.insert_screenshot(&screenshot).await?;

            break; // save only the first screen for now
        }
        Ok(())
    }

    async fn insert_screenshot(&self, screenshot: &Screenshot) -> Result<i64> {
        let db = self.app.state::<db::ImmicDb>();
        let event_id = db.insert_eventlog(screenshot.timestamp, KIND).await?;

        let pool = db.pool().await.expect("db pool is not set");
        let result = sqlx::query(
            r#"
            INSERT INTO screenshot (event_id, monitor_id)
            VALUES (?, ?)
            "#
        )
        .bind(event_id)
        .bind(screenshot.monitor)
        .execute(&pool)
        .await?;

        // Update event_log with log_id
        let log_id = result.last_insert_rowid();
        // db.update_eventlog_logid(event_id, log_id).await?;
        Ok(log_id)
    }

    async fn save_screenshot(&self, screenshot: &Screenshot) -> Result<()> {
        let image_dir = self.image_dir.lock().unwrap().clone().unwrap();
        let path = image_path(&image_dir, screenshot.timestamp, screenshot.monitor);
        screenshot.image.save(path).expect("failed to save screenshot");

        Ok(())
    }

    pub async fn insert_screenshot_log_with(&self, pool: &Pool<Sqlite>, log: &ScreenshotLog) -> Result<i64> {
        let timestamp = DateTime::from_timestamp(log.timestamp, 0).context("Invalid timestamp")?;

        let db = self.app.state::<db::ImmicDb>();
        let event_id = db.insert_eventlog_with(pool, timestamp, KIND).await?;

        let result = sqlx::query(
            r#"
            INSERT INTO screenshot (event_id, monitor_id)
            VALUES (?, ?)
            "#
        )
        .bind(event_id)
        .bind(log.monitor_id)
        .execute(pool)
        .await?;

        // Update event_log with log_id
        let log_id = result.last_insert_rowid();
        // db.update_eventlog_logid_with(pool, event_id, log_id).await?;
        Ok(log_id)
    }

    pub async fn list_screenshot_logs_on(&self, date: String) -> Result<Vec<ScreenshotLog>> {
        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await.context("db pool is not set")?;

        let mut rows = sqlx::query_as::<_, (
            i64, i64, String, String,
            i64, i64
        )>(
            r#"
            SELECT
            e.id, e.timestamp, e.date, e.kind,
            s.id, s.monitor_id
            FROM event_log e
            INNER JOIN screenshot s ON e.id = s.event_id
            WHERE e.kind = ? AND e.date = ?
            ORDER BY e.id
            "#
        )
        .bind(KIND)
        .bind(date)
        .fetch(&pool);

        let mut screenshot_logs = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let (
                event_id, timestamp, date, _kind,
                id, monitor_id,
            ) = row;
            screenshot_logs.push(ScreenshotLog {
                id,
                event_id,
                timestamp,
                date,
                monitor_id,
            });
        }
        Ok(screenshot_logs)
    }

    // pub async fn get_screenshots_for(&self, timeframe: i64) -> Result<Vec<String>> {
    //     let db = self.app.state::<db::ImmicDb>();
    //     let pool = db.pool().await.expect("db pool is not set");

    //     let ss: Vec<String> = sqlx::query_as::<_, (i64, i64)>(
    //         r#"
    //         SELECT
    //             e.timestamp,
    //             s.monitor_id
    //         FROM event_log e
    //         INNER JOIN screenshot s ON e.log_id = s.id
    //         WHERE e.kind = ? AND e.timeframe = ?
    //         "#
    //     )
    //     .bind(KIND)
    //     .bind(timeframe)
    //     .fetch_all(&pool)
    //     .await
    //     .unwrap_or(Vec::new())
    //     .iter()
    //     .map(|(timestamp, monitor_id)| {
    //         let ts = DateTime::from_timestamp(*timestamp, 0).unwrap();
    //         let dir = image_dir_name(ts);
    //         let filename = image_basename(ts, *monitor_id);
    //         format!("{}/{}", dir, filename)
    //     })
    //     .collect();

    //     Ok(ss)  
    // }
}

fn image_base_dir(app: &AppHandle) -> Result<PathBuf> {
    let setting = app.state::<SettingPlugin>();
    let data_dir = setting.get(DATA_DIR_SETTING)?
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .map(PathBuf::from);
    if data_dir.is_none() {
        return Err(anyhow!("{} is not set", DATA_DIR_SETTING));
    }

    let image_dir = data_dir.unwrap().join(SCREENSHOT_DIR);

    // Create directories if not exists
    if !image_dir.exists() {
        std::fs::create_dir_all(&image_dir).unwrap();
    }

    Ok(image_dir)
}

fn image_dir_name(timestamp: DateTime<Utc>) -> String {
    timestamp.format("%Y%m%d").to_string()
}

fn image_basename(timestamp: DateTime<Utc>, monitor_id: i64) -> String {
    format!("{}-{}", timestamp.format("%H%M%S"), monitor_id)
}

fn image_path(dir: &PathBuf, timestamp: DateTime<Utc>, monitor_id: i64) -> PathBuf {
    let date_dir = dir.join(image_dir_name(timestamp));
    if !date_dir.exists() {
        std::fs::create_dir(&date_dir).unwrap();
    }
    let basename = image_basename(timestamp, monitor_id);
    date_dir.join(format!("{}.jpg", basename))
}

fn is_blank(image: &RgbaImage) -> bool {
    static ALMOST_BLACK_THRESHOLD: u8 = 20;
    static NON_BLANK_THRESHOLD: u32 = 400;

    let mut count = 0;
    for pixel in image.pixels().step_by(120) {
        if pixel.0[0] > ALMOST_BLACK_THRESHOLD || pixel.0[1] > ALMOST_BLACK_THRESHOLD || pixel.0[2] > ALMOST_BLACK_THRESHOLD {
            count += 1;
        }
        if count > NON_BLANK_THRESHOLD {
            // debug!("Non blank screen: {}", count);
            return false;
        }
    }
    // debug!("Blank screen: count: {}", count);
    return true;
}

struct Screenshot {
    monitor: i64,
    timestamp: DateTime<Utc>,
    image: RgbaImage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotLog {
    pub id: i64,
    pub event_id: i64,
    pub timestamp: i64,
    pub date: String,
    pub monitor_id: i64,
}

impl db::Timestamp for ScreenshotLog {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

pub fn handle_iss_protocol(app: &AppHandle, request: &http::Request) -> Result<http::Response, Box<dyn Error>> {
    let uri = request.uri();
    if !check_iss_uri(uri) {
        return Err("Invalid uri".into());
    }

    // split the uri into date directory and filename
    // skip the first 16 characters: iss://localhost/
    let mut parts = uri[16..].split('/');
    let date = parts.next().unwrap();
    let filename = parts.next().unwrap();

    let screen_dir = image_base_dir(app).expect("image_dir is not set");

    let date_dir = screen_dir.join(date);
    let path = date_dir.join(format!("{}.jpg", filename));
    if path.exists() {
        let builder = http::ResponseBuilder::new();
        let response = if let Ok(data) = fs::read(path) {
            builder.status(200).mimetype("image/jpeg").body(data).unwrap()
        } else {
            builder.status(404).body(Vec::new()).unwrap()
        };
        Ok(response)
    } else {
        Err("Not found".into())
    }
}

fn check_iss_uri(uri: &str) -> bool {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^iss://localhost/\d{8}/\d{6}-[A-Za-z0-9]*(-t)?$").unwrap());
    if RE.is_match(uri) {
        return true;
    }
    error!("Invalid uri: {}", uri);
    false
}

#[tauri::command]
pub async fn list_screenshot_logs_on(screenshot_plugin: State<'_, ScreenshotPlugin>, date: String) -> Result<Vec<ScreenshotLog>, String> {
    screenshot_plugin.list_screenshot_logs_on(date).await.map_err(|e| e.to_string())
}

// pub async fn get_screenshots_for(screenshot_plugin: State<'_, ScreenshotPlugin>, timeframe: i64) -> Result<Vec<String>, String> {
//     screenshot_plugin.get_screenshots_for(timeframe).await.map_err(|e| e.to_string())
// }

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_iss_uri() {
        assert!(check_iss_uri("iss://localhost/20210901/123456-abcdef"));
        assert!(check_iss_uri("iss://localhost/20210901/123456-abcdef-t"));
        assert!(!check_iss_uri("iss://localhost/20210901/123456-abcdef-t-t"));
        assert!(!check_iss_uri("iss://localhost/20210901/123456-abcdef.png"));
        assert!(!check_iss_uri("iss://localhost/20210901/123456-abcdef/"));
        assert!(!check_iss_uri("iss://localhost/20210901/123456-abcdef/abc"));
        assert!(!check_iss_uri("iss://localhost/20210901/123456-abcdef/.."));
        assert!(!check_iss_uri("iss://localhost//20210901/123456-abcdef"));
        assert!(!check_iss_uri("iss://localhost/../20210901/123456-abcdef"));
    }
}
