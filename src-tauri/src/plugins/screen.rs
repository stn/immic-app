use chrono::Local;
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::Path;
use xcap::Monitor;
use tauri::AppHandle;
use tauri::http;

pub struct ScreenshotPlugin;

impl ScreenshotPlugin {
    pub fn new() -> Self {
        Self {}
    }
    pub fn start(&mut self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                take_screenshot();
            }
        });
    }
}

fn take_screenshot() {
    println!("screenshot");
    let monitors = Monitor::all().unwrap();

    for monitor in monitors {
        let mut image = monitor.capture_image().unwrap();
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
        let filename = format!("{}-{}.jpg", dt.format("%H%M%S"), monitor.id());
        let path = date_dir.join(filename);
        image.save(path).unwrap();

        // thumbnail
        let width = image.width() / 8;
        let height = image.height() / 8;
        let thumb = image::imageops::thumbnail(&mut image, width, height);
        thumb.save(date_dir.join(format!("{}-{}-t.jpg", dt.format("%H%M%S"), monitor.id()))).unwrap();

        break; // save only the first screen for now
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
    // println!("Invalid uri: {}", uri);
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
                // println!("filename: {}", filename);
                let caps = RE.captures(&filename).unwrap();
                let image_name = &caps[1];
                println!("image_name: {}", image_name);
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
