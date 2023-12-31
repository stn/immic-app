use async_cron_scheduler::{Job, JobId};
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
        image
            .save(format!("ss-{}.png", screen.display_info.id))
            .unwrap();
    }
}
