use async_cron_scheduler::{Job, JobId, Scheduler};
use chrono::offset::Local;

pub struct AppScheduler {
    scheduler: Option<Scheduler<Local>>,
}
impl AppScheduler {
    pub fn new() -> Self {
        Self {
            scheduler: None,
        }
    }
    pub fn start(&mut self) {
        let (sched, sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);
        self.scheduler = Some(sched);
        tauri::async_runtime::spawn(sched_service);
    }
    pub fn insert(&mut self,
                  job: Job<Local>,
                  command: impl Fn(JobId) + Send + Sync + 'static
    ) -> Option<JobId> {
        match &mut self.scheduler {
            Some(scheduler) => Some(scheduler.insert(job, command)),
            None => {
                eprintln!("No scheduler!");
                None
            }
        }
    }
}
