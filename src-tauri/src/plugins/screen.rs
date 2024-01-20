use async_cron_scheduler::{Job, JobId};
use chrono::Local;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use screenshots::Screen;
use tauri::{App, AppHandle, Manager, State};
use tauri::http;

use crate::app::scheduler::AppScheduler;

pub struct ScreenshotPlugin {
    job_id: Option<JobId>,
}

impl ScreenshotPlugin {
    pub fn new() -> Self {
        Self {
            job_id: None,
        }
    }
    pub fn start(&mut self, app: &App) {
        let scheduler: State<Mutex<AppScheduler>> = app.state();
        let job = Job::cron("0 * * * * *").unwrap();
        self.job_id = scheduler.lock().unwrap().insert(job, |_id| take_screenshot());
    }
}

fn take_screenshot() {
    println!("screenshot");
    let screens = Screen::all().unwrap();

    for screen in screens {
        let image = screen.capture().unwrap();
        let dt = Local::now();
        // Create directories if not exists
        let base_dir = Path::new(r"F:\immic-dev");
        if !base_dir.exists() {
            std::fs::create_dir(&base_dir).unwrap();
        }
        let screen_dir = base_dir.join("screen");
        if !screen_dir.exists() {
            std::fs::create_dir(&screen_dir).unwrap();
        }
        let date_dir = screen_dir.join(dt.format("%Y%m%d").to_string());
        if !date_dir.exists() {
            std::fs::create_dir(&date_dir).unwrap();
        }
        let filename = format!("{}-{}.png", dt.format("%H%M%S"), screen.display_info.id);
        let path = date_dir.join(filename);
        image.save(path).unwrap();
    }
}

pub fn handle_iss_protocol(_app: &AppHandle, request: &http::Request) -> Result<http::Response, Box<dyn Error>> {
    // let screenshot: State<Mutex<ScreenshotPlugin>> = app.state();
    // let mut screenshot = screenshot.lock().unwrap();
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
    let path = date_dir.join(filename);
    if path.exists() {
        let builder = http::ResponseBuilder::new();
        let response = if let Ok(data) = fs::read(path) {
            builder.status(200).mimetype("image/png").body(data).unwrap()
        } else {
            builder.status(404).body(Vec::new()).unwrap()
        };
        Ok(response)
    } else {
        Err("Not found".into())
    }
}

fn check_iss_uri(uri: &str) -> bool {
    let re = Regex::new(r"^iss://localhost/\d{8}/\d{6}-[A-Za-z0-9]*\.png$").unwrap();
    if re.is_match(uri) {
        return true;
    }
    print!("Invalid uri: {}", uri);
    false
}
 

#[tauri::command]
pub fn list_dates() -> Result<Vec<String>, String> {
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
    // List all screenshots in a date
    let base_dir = Path::new(r"F:\immic-dev");
    let screen_dir = base_dir.join("screen");
    let date_dir = screen_dir.join(date);
    let mut screenshots = vec![];
    if date_dir.exists() {
        let re = Regex::new(r"\d{6}-[A-Za-z0-9]*\.png$").unwrap();
        let paths = std::fs::read_dir(date_dir).unwrap();
        for path in paths {
            let path = path.unwrap().path();
            if path.is_file() {
                let filename = path.file_name().unwrap().to_str().unwrap().to_string();
                if re.is_match(&filename) {
                    screenshots.push(filename);
                }
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
        assert!(check_iss_uri("iss://localhost/20210901/123456-abcdef.png"));
        assert!(!check_iss_uri("iss://localhost/20210901/123456-abcdef.jpg"));
        assert!(!check_iss_uri("iss://localhost/20210901/123456-abcdef"));
        assert!(!check_iss_uri("iss://localhost/20210901/123456-abcdef.png/"));
        assert!(!check_iss_uri("iss://localhost/20210901/123456-abcdef.png/abc"));
        assert!(!check_iss_uri("iss://localhost/20210901/123456-abcdef.png/abc/"));
        assert!(!check_iss_uri("iss://localhost//20210901/123456-abcdef.png"));
        assert!(!check_iss_uri("iss://localhost/../20210901/123456-abcdef.png"));
    }
}
