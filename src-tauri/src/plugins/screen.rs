use anyhow::Result;
use chrono::{DateTime, Utc};
use image::RgbaImage;
use once_cell::sync::Lazy;
use regex::Regex;
use log::{debug, error};
use std::error::Error;
use std::fs;
use std::path::Path;
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
        // Create directories if not exists
        let base_dir = Path::new(r"F:\immic-dev"); // TODO settingのdata-dirを使う
        if !base_dir.exists() {
            std::fs::create_dir(&base_dir).unwrap();
        }
        let screen_dir = base_dir.join("screen");
        if !screen_dir.exists() {
            std::fs::create_dir(&screen_dir).unwrap();
        }
        let date_dir = screen_dir.join(self.timestamp.format("%Y%m%d").to_string());
        if !date_dir.exists() {
            std::fs::create_dir(&date_dir).unwrap();
        }
        let filename = format!("{}-{}.jpg", self.timestamp.format("%H%M%S"), self.monitor);
        let path = date_dir.join(filename);

        self.image.save(path).unwrap();

        // thumbnail
        let width = self.image.width() / 8;
        let height = self.image.height() / 8;
        let mut image = self.image.clone();
        let thumb = image::imageops::thumbnail(&mut image, width, height);
        thumb.save(date_dir.join(format!("{}-{}-t.jpg", self.timestamp.format("%H%M%S"), self.monitor))).unwrap();
        Ok(())
    }
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
    let base_dir = Path::new(r"F:\immic-dev");
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
 

#[tauri::command]
pub fn list_screen_dates() -> Result<Vec<String>, String> {
    // List all screenshot dates
    let base_dir = Path::new(r"F:\immic-dev");
    let screen_dir = base_dir.join("screen");
    let mut dates = vec![];
    if screen_dir.exists() {
        let paths = std::fs::read_dir(screen_dir).unwrap();
        for path in paths {
            let path = path.unwrap().path();
            if path.is_dir() {
                let date = path.file_name().unwrap().to_str().unwrap().to_string();
                dates.push(date);
            }
        }
    }
    Ok(dates)
}

#[tauri::command]
pub fn list_screens(date: &str) -> Result<Vec<String>, String> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d{6}-[A-Za-z0-9]+)\.jpg$").unwrap());
    // List all screenshots in a date
    let base_dir = Path::new(r"F:\immic-dev");
    let screen_dir = base_dir.join("screen");
    let date_dir = screen_dir.join(date);
    let mut screenshots = vec![];
    if date_dir.exists() {
        let paths = std::fs::read_dir(date_dir).unwrap();
        for path in paths {
            let path = path.unwrap().path();
            let filename = path.file_name().unwrap().to_str().unwrap();
            if RE.is_match(&filename) && path.is_file() {
                let caps = RE.captures(&filename).unwrap();
                let image_name = &caps[1];
                screenshots.push(image_name.to_string());
            }
        }
    }
    Ok(screenshots)
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
