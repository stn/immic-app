use std::sync::{Arc, Mutex};
// use sqlx;

// use watchexec_signals::Signal;
use watchexec::Watchexec;

// use crate::app::db;
use crate::plugins::Plugin;

// #[derive(Debug)]
// pub struct ApplicationLog {
//     process_id: i64,
//     name: String,
//     title: String,
//     x: i64,
//     y: i64,
//     width: i64,
//     height: i64,
// }

pub struct FilelogPlugin {
    running: Arc<Mutex<bool>>,
}

impl FilelogPlugin {
    pub fn new() -> Self {
        Self {
            running: Arc::new(Mutex::new(false)),
        }
    }
}

impl Plugin for FilelogPlugin {
    fn start(&mut self) {
        println!("filelog");
        *self.running.lock().unwrap() = true;
        let running = Arc::clone(&self.running);
        let wx = Watchexec::new(move |mut action| {
            if !*running.lock().unwrap() {
                action.quit();
            }

            // print any events
            for event in action.events.iter() {
                eprintln!("EVENT: {event:?}");
            }

            // // if Ctrl-C is received, quit
            // if action.signals().any(|sig| sig == Signal::Interrupt) {
            //     action.quit();
            // }

            action
        }).unwrap();

        // watch the current directory
        wx.config.pathset(["f:\\"]);

        tokio::spawn(async move {
            wx.main().await.unwrap();
        });

        // let (tx, mut rx) = mpsc::channel(256);

        // let mut watcher = notify::recommended_watcher(move |res: _| match res {
        //     Ok(event) => {
        //         let event: Event = event;

        //         // only handle newly created files
        //         if event.kind.is_create() {
        //             event.paths.iter().for_each(|path| {
        //                 let file_path = path.to_string_lossy();
        //                 // THIS PRINTS EVERYTIME I CREATE A FILE
        //                 println!("Created new file: {:?}", file_path);
        //                 if let Err(err) = tx.blocking_send(file_path.to_string()) {
        //                     eprintln!("Error handling file {:#?}", err);
        //                 };
        //             });
        //         }
        //     }
        //     Err(e) => eprintln!("watch error: {:?}", e),
        // })
        // .ok()
        // .unwrap();

        // let path = Path::new("D:\\Works\\src\\github.com\\stn\\");
        // println!("Watching path: {:?}", path);
        // watcher
        //     .watch(path, RecursiveMode::Recursive)
        //     .unwrap();

        // tokio::task::spawn(async move {
        //     while let Some(file) = rx.recv().await {
        //         // NOTHING HERE
        //         println!("Received file: {:?}", file);
        //     }
        // });
    }

    fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
    }
}

// async fn check_application() {
//     println!("application");
//     match get_active_window() {
//         Ok(win) => {
//             // println!("active_window: {:?}", win);
//             let log = ApplicationLog {
//                 process_id: win.process_id as i64,
//                 name: win.app_name,
//                 title: win.title,
//                 x: win.position.x as i64,
//                 y: win.position.y as i64,
//                 width: win.position.width as i64,
//                 height: win.position.height as i64,
//             };
//             println!("application log: {:?}", log);
//             insert_application_log(log).await.unwrap_or_else(|e| {
//                 println!("check_application: Error on insert_application_log: {:?}", e);
//             });
//         },
//         Err(e) => {
//             println!("active_window: {:?}", e);
//         },
//     }
// }

// async fn insert_application_log(log: ApplicationLog) -> Result<()> {
//     let pool = db::pool();
//     sqlx::query(
//         "INSERT INTO application (eventId, kind, processId, name, title, x, y, width, height) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
//     )
//         .bind(1)
//         .bind("active")
//         .bind(log.process_id)
//         .bind(log.name)
//         .bind(log.title)
//         .bind(log.x)
//         .bind(log.y)
//         .bind(log.width)
//         .bind(log.height)
//         .execute(pool).await?;
//     Ok(())
// }
