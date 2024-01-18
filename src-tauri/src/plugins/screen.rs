use async_cron_scheduler::{Job, JobId};
use chrono::Local;
use std::fmt::Debug;
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
        let filename = format!("ss-{}-{}.png", dt.format("%Y%m%d-%H%M%S"), screen.display_info.id);
        let path = Path::new(r"F:\immic-dev").join("screen").join(filename);
        image.save(path).unwrap();
    }
}
