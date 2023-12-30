
use async_cron_scheduler::{Job, JobId, Scheduler};
use chrono::offset::Local;
use std::sync::Mutex;

use screenshots::Screen;

pub struct ScreenshotPlugin {
    scheduler: Mutex<Option<Scheduler<Local>>>,
}
impl ScreenshotPlugin {
    pub fn new() -> Self {
        Self {
            scheduler: Mutex::new(None),
        }
    }
    pub fn start(&self) {
        let (mut sched, sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);
        let mut scheduler = self.scheduler.lock().unwrap();

        let job = Job::cron("0 * * * * *").unwrap();
        sched.insert(job, |id| println!("Job!"));

        scheduler.replace(sched);
        tauri::async_runtime::spawn(sched_service);
    }
}

pub fn take_screenshot() {
    let screens = Screen::all().unwrap();

    for screen in screens {
        let image = screen.capture().unwrap();
        image
            .save(format!("ss-{}.png", screen.display_info.id))
            .unwrap();
    }
}
