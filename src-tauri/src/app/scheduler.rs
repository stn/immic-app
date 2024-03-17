// use std::sync::Mutex;
// use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

// pub struct AppScheduler {
//     scheduler: Mutex<Option<JobScheduler>>,
// }

// impl AppScheduler {
//     pub fn new() -> Self {
//         Self {
//             scheduler: None,
//         }
//     }

//     pub async fn start(&mut self) -> Result<(), JobSchedulerError> {
//         // let (sched, sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);
//         // self.scheduler = Some(sched);
//         // tauri::async_runtime::spawn(sched_service);
//         let mut scheduler = JobScheduler::new().await?;
//         self.scheduler.lock().unwrap().replace(scheduler);
//         scheduler.start().await?;
//         Ok(())
//     }

//     pub fn insert(&mut self,
//                   job: Job,
//                 //   command: impl Fn(JobId) + Send + Sync + 'static
//     ) -> Option<JobId> {
//         match &mut self.scheduler {
//             Some(scheduler) => Some(scheduler.insert(job, command)),
//             None => {
//                 eprintln!("No scheduler!");
//                 None
//             }
//         }
//     }
// }
