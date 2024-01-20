use async_cron_scheduler::{Job, JobId};
use chrono::Local;
use std::path::Path;
use std::sync::Mutex;
use screenshots::Screen;
use tauri::{App, Manager, State};

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
        let filename = format!("ss-{}-{}.png", dt.format("%H%M%S"), screen.display_info.id);
        let path = date_dir.join(filename);
        image.save(path).unwrap();
    }
}
