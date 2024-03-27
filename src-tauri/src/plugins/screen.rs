use anyhow::Result;
use chrono::{DateTime, Utc};
use image::RgbaImage;
use once_cell::sync::Lazy;
use regex::Regex;
use log::{debug, error};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use xcap::Monitor;
use tauri::AppHandle;
use tauri::http;

use crate::app::db;
use crate::plugins::Plugin;

const KIND: &str = "screenshot";

pub struct ScreenshotPlugin {
    running: Arc<Mutex<bool>>,
}

impl ScreenshotPlugin {
    pub fn new() -> Self {
        Self {
            running: Arc::new(Mutex::new(false)),
        }
    }
}

impl Plugin for ScreenshotPlugin {
    fn start(&mut self) {
        *self.running.lock().unwrap() = true;
        let running = Arc::clone(&self.running);
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        tokio::spawn(async move {
            loop {
                if !*running.lock().unwrap() {
                    break;
                }
                interval.tick().await;
                take_screenshot().await.unwrap_or_else(|e| {
                    error!("Error on taking screenshot: {:?}", e);
                });
            }
        });
    }

    fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
    }
}

async fn take_screenshot() -> Result<()> {
    debug!("screenshot");
    let monitors = Monitor::all().unwrap();

    for monitor in monitors {
        let screenshot = Screenshot {
            monitor: monitor.id() as i64,
            timestamp: chrono::Utc::now(),
            image: monitor.capture_image().unwrap(),
        };
        // TODO: run in a separate thread
        screenshot.save().await?;
        screenshot.insert().await?;

        break; // save only the first screen for now
    }
    Ok(())
}

struct Screenshot {
    monitor: i64,
    timestamp: DateTime<Utc>,
    image: RgbaImage,
}

impl Screenshot {
    async fn insert(&self) -> Result<i64> {
        let event_id = db::insert_eventlog(self.timestamp, KIND).await?;

        let result = sqlx::query(
            r#"
            INSERT INTO screenshot (event_id, monitor_id)
            VALUES (?, ?)
            "#
        )
        .bind(event_id)
        .bind(self.monitor)
        .execute(db::pool())
        .await?;

        // Update event_log with log_id
        let log_id = result.last_insert_rowid();
        db::update_eventlog_logid(event_id, log_id).await?;
        Ok(log_id)
    }

    async fn save(&self) -> Result<()> {
        let (path, thumb_path) = image_path(self.timestamp, self.monitor);
        self.image.save(path).unwrap();

        // thumbnail
        let width = self.image.width() / 8;
        let height = self.image.height() / 8;
        let mut image = self.image.clone();
        let thumb = image::imageops::thumbnail(&mut image, width, height);
        thumb.save(thumb_path).unwrap();

        Ok(())
    }
}

fn image_path(timestamp: DateTime<Utc>, monitor_id: i64) -> (PathBuf, PathBuf) {
    // Create directories if not exists
    let base_dir = Path::new(r"F:\immic-dev"); // TODO settingのdata-dirを使う
    if !base_dir.exists() {
        std::fs::create_dir(&base_dir).unwrap();
    }
    let screen_dir = base_dir.join("screen");
    if !screen_dir.exists() {
        std::fs::create_dir(&screen_dir).unwrap();
    }
    let date_dir = screen_dir.join(timestamp.format("%Y%m%d").to_string());
    if !date_dir.exists() {
        std::fs::create_dir(&date_dir).unwrap();
    }
    let filename = format!("{}-{}.jpg", timestamp.format("%H%M%S"), monitor_id);
    let path = date_dir.join(filename);
    let thumb_path = date_dir.join(format!("{}-{}-t.jpg", timestamp.format("%H%M%S"), monitor_id));

    (path, thumb_path)
}

#[derive(Debug, serde::Serialize)]
pub struct ScreenshotLog {
    pub id: i64,
    pub event_id: i64,
    pub timestamp: i64,
    pub date: String,
    pub monitor_id: i64,
}

#[tauri::command]
pub async fn list_screenshots(date: &str) -> Result<Vec<String>, String> {
    debug!("list_screenshots: date: {}", date);
    let screenshot_logs: Vec<ScreenshotLog> = sqlx::query_as::<_,
      (i64, i64, String, String, i64,
       i64, i64)>(
        r#"
        SELECT
          e.id, e.timestamp, e.date, e.kind, e.log_id,
          s.id, s.monitor_id
        FROM event_log e
        INNER JOIN screenshot s ON e.log_id = s.id
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
        let (event_id, timestamp, date, _kind, _log_id,
             id, monitor_id,
            ) = row;
        ScreenshotLog {
            id: *id,
            event_id: *event_id,
            timestamp: *timestamp,
            date: date.clone(),
            monitor_id: *monitor_id,
        }
    })
    .collect();
    debug!("list_screenshots: screenshot_logs: {:?}", screenshot_logs);

    let screenshots: Vec<String> = screenshot_logs.iter().map(|log| {
        let timestamp = DateTime::from_timestamp(log.timestamp, 0).unwrap();
        format!("{}/{}-{}", timestamp.format("%Y%m%d"), timestamp.format("%H%M%S"), log.monitor_id)
    }).collect();

    Ok(screenshots)
}

pub fn handle_iss_protocol(_app: &AppHandle, request: &http::Request) -> Result<http::Response, Box<dyn Error>> {
    let uri = request.uri();
    if !check_iss_uri(uri) {
        return Err("Invalid uri".into());
    }

    // split the uri into date directory and filename
    // skip the first 16 characters: iss://localhost/
    let mut parts = uri[16..].split('/');
    let date = parts.next().unwrap();
    let filename = parts.next().unwrap();
    let base_dir = Path::new(r"F:\immic-dev"); // TODO use data-dir from setting
    let screen_dir = base_dir.join("screen");
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
