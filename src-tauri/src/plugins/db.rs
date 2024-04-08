use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, Timelike, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::Mutex,
};
use sqlx::{
    Pool, Sqlite,
    sqlite::{
        SqliteConnectOptions,
        SqliteJournalMode,
        SqlitePoolOptions,
        SqliteSynchronous,
    },
};
use tauri::{
    AppHandle, Manager, RunEvent, State, Wry,
    plugin::TauriPlugin,
};

use crate::plugins::setting::SettingPlugin;

const DATABASE_FILE: &str = "immic.db";
const DATA_DIR_SETTING: &str = "data-dir";

pub fn init() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("immicdb")
        .invoke_handler(tauri::generate_handler![
            list_eventlog_dates,
            list_eventlog_on,
        ])
        .setup(move |app| {
            debug!("immicdb plugin setup");

            let db = ImmicDb::new(app.clone());
            app.manage(db);

            Ok(())
        })
        .on_event(|app, event| {
            // let app = app.clone();
            match event {
                // RunEvent::Ready => {
                //     debug!("RunEvent::Ready");
                //     tokio::spawn(async move {
                //         let db = app.state::<ImmicDb>();
                //         let (resp_tx, resp_rx) = oneshot::channel();
                //         if let Err(e) = db.tx.send(Command::Migrate { resp: resp_tx }).await {
                //             error!("failed to send migrate command: {}", e);
                //             return;
                //         }
                //         if let Err(e) = resp_rx.await {
                //             error!("failed to migrate: {}", e);
                //         }
                //     });
                // },
                RunEvent::Exit => {
                    debug!("RunEvent::Exit");
                    let db = app.state::<ImmicDb>();
                    let pool = db.pool.lock().unwrap().clone();
                    if let Some(pool) = pool {
                        tokio::spawn(async move {
                            pool.close().await;
                        });
                    }
                    // let (resp_tx, resp_rx) = oneshot::channel();
                    // let cmd = Command::Close {
                    //     resp: resp_tx,
                    // };
                    // tokio::spawn(async move {
                    //     if db.tx.send(cmd).await.is_err() {
                    //         error!("failed to send close command");
                    //         return;
                    //     }
                    //     if resp_rx.await.is_err() {
                    //         error!("failed to close db");
                    //     }
                    // });
                },
                _ => (),
            }
        })
        .build()
}

pub struct ImmicDb {
    app: AppHandle,
    pool: Mutex<Option<Pool<Sqlite>>>,
    // tx: mpsc::Sender<Command>,
}

impl ImmicDb {
    fn new(app: AppHandle) -> Self {
        // let (tx, mut rx) = mpsc::channel(128);

        let db = Self {
            app,
            pool: Mutex::new(None),
            // tx,
        };

        // tokio::spawn(async move {
        //     while let Some(cmd) = rx.recv().await {
        //         debug!("received command: {:?}", cmd);
        //         let db = app.state::<ImmicDb>();
        //         match cmd {
        //             Command::Migrate { resp } => {
        //                 debug!("migrate");
        //                 let result = db.migrate().await;
        //                 debug!("migrate result: {:?}", result);
        //                 resp.send(result).unwrap_or_else(|e| {
        //                     error!("failed to send migrate result: {:?}", e);
        //                 });
        //             },
        //             Command::Close { resp } => {
        //                 rx.close();
        //                 let pool = db.pool.lock().unwrap().clone();
        //                 if let Some(pool) = pool {
        //                     pool.close().await;
        //                 }
        //                 let _ = resp.send(Ok(()));
        //             },
        //             Command::Pool { resp } => {
        //                 let pool = db.pool.lock().unwrap().clone();
        //                 let _ = resp.send(match pool {
        //                     Some(pool) => Ok(pool),
        //                     None => {
        //                         debug!("initialize pool");
        //                         let path = db_path(app.clone()).unwrap();
        //                         debug!("db path: {:?}", path);
        //                         let options = SqliteConnectOptions::new()
        //                             .filename(path)
        //                             .create_if_missing(true)
        //                             .journal_mode(SqliteJournalMode::Wal)
        //                             .synchronous(SqliteSynchronous::Normal);
        //                         let pool = SqlitePoolOptions::new().connect_lazy_with(options);
        //                         db.pool.lock().unwrap().replace(pool.clone());
        //                         Ok(pool)
        //                     }
        //                 });
        //             },
        //         }
        //     }
        // });

        db
    }

    pub async fn pool(&self) -> Result<Pool<Sqlite>> {
        // let (resp_tx, resp_rx) = oneshot::channel();
        // let cmd = Command::Pool {
        //     resp: resp_tx,
        // };
        // debug!("send Pool command");
        // if self.tx.send(cmd).await.is_err() {
        //     return Err(anyhow!("failed to send command"));
        // }
        // debug!("await Pool response");
        // match resp_rx.await {
        //     Ok(result) => {
        //         debug!("pool result: {:?}", result);
        //         result
        //     },
        //     Err(e) => Err(anyhow!("failed to get pool: {}", e)),
        // }

        // let pool = self.pool.lock().unwrap().clone();
        // match self.pool.lock().unwrap().as_ref() {
        //     Some(pool) => Ok(pool.clone()),
        //     None => Err(anyhow!("pool is not initialized")),
        // }

        self.pool.lock().unwrap().clone()
            .ok_or_else(|| anyhow!("pool is not initialized"))

        // match pool {
        //     Some(pool) => Ok(pool),
        //     None => {
        //         debug!("initialize pool");
        //         let path = db_path(self.app.clone()).unwrap();
        //         debug!("db path: {:?}", path);
        //         let options = SqliteConnectOptions::new()
        //             .filename(path)
        //             .create_if_missing(true)
        //             .journal_mode(SqliteJournalMode::Wal)
        //             .synchronous(SqliteSynchronous::Normal);
        //         let pool = SqlitePoolOptions::new().connect_lazy_with(options);
        //         self.pool.lock().unwrap().replace(pool.clone());
        //         Ok(pool)
        //     }
        // }
    }

    pub fn start(&self) -> Result<()> {
        debug!("start immicdb");

        if self.pool.lock().unwrap().is_some() {
            return Ok(());
        }

        let path = self.db_path()?;
        // if let Err(e) = path {
        //     error!("failed to get db path: {}", e);
        //     return Err(e);
        // }
        // let path = path.unwrap();
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);
        let pool = SqlitePoolOptions::new().connect_lazy_with(options);
        self.pool.lock().unwrap().replace(pool);
        Ok(())
    }

    fn db_path(&self) -> Result<PathBuf> {
        // debug!("db_path");
        let setting = self.app.state::<SettingPlugin>();
        let data_dir = setting.get(DATA_DIR_SETTING)?
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .map(PathBuf::from);
        if data_dir.is_none() {
            return Err(anyhow!("{} is not set", DATA_DIR_SETTING));
        }
        // debug!("{}: {:?}", DATA_DIR_SETTING, data_dir);

        let db_path = data_dir.unwrap().join(DATABASE_FILE);
        // debug!("db_path: {}", db_path);

        Ok(db_path)
    }

    pub async fn migrate(&self) -> Result<()> {
        debug!("migrate immicdb");
        let pool = self.pool().await?;
        debug!("pool: {:?}", pool);
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let pool = self.pool().await?;
        pool.close().await;
        Ok(())
    }

    pub async fn insert_eventlog(&self, datetime: DateTime<Utc>, kind: &str) -> Result<i64> {
        let pool = self.pool().await?;

        // timestamp to date string in local timezone
        let ts = datetime.timestamp();
        let timeframe = ts / 60;
        let local_time = datetime.with_timezone(&chrono::Local);
        let date = local_time.format("%Y%m%d").to_string();
        let result = sqlx::query(
            r#"
            INSERT INTO event_log (timestamp, timeframe, date, kind)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(ts)
        .bind(timeframe)
        .bind(date)
        .bind(kind)
        .execute(&pool).await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn update_eventlog_logid(&self, id: i64, log_id: i64) -> Result<()> {
        let pool = self.pool().await?;
        sqlx::query(
            r#"
            UPDATE event_log
            SET log_id = ?
            WHERE id = ?
            "#
        )
        .bind(log_id)
        .bind(id)
        .execute(&pool).await?;
        Ok(())
    }

    pub async fn list_eventlog_dates(&self) -> Result<Vec<String>> {
        let pool = self.pool().await?;
        let result: Vec<String> = sqlx::query_as::<_, (String,)>(r#"
            SELECT DISTINCT date
            FROM event_log
            ORDER BY date DESC
            "#
        )
        .fetch_all(&pool)
        .await
        .unwrap_or(Vec::new())
        .iter()
        .map(|row| {
            let (date,) = row;
            date.clone()
        })
        .collect();
        Ok(result)
    }

    pub async fn list_eventlog_on(&self, date: String) -> Result<Vec<EventLog>> {
        let pool = self.pool().await?;
        let result: Vec<EventLog> = sqlx::query_as::<_, (i64, i64, String, String)>(
            r#"
            SELECT id, timestamp, date, kind
            FROM event_log
            WHERE date = ?
            ORDER BY id
            "#
        )
        .bind(date)
        .fetch_all(&pool)
        .await
        .unwrap_or(Vec::new())
        .iter()
        .map(|row| {
            let (id, timestamp, date, kind) = row;
            EventLog {
                id: *id,
                timestamp: *timestamp,
                date: date.clone(),
                kind: kind.clone(),
            }
        })
        .collect();
        Ok(result)
    }
}

// pub async fn with_pool<T, F: FnOnce(&Pool<Sqlite>) -> Result<T>>(
//     app: AppHandle,
//     f: F,
// ) -> Result<T> {
//     let db = app.state::<ImmicDb>();
//     let pool = db.pool().await?;
//     f(&pool)
// }


// EventLog

#[derive(Debug, Serialize)]
pub struct EventLog {
    pub id: i64,
    pub timestamp: i64,
    pub date: String,
    pub kind: String,
}

#[tauri::command]
pub async fn list_eventlog_dates(db: State<'_, ImmicDb>) -> Result<Vec<String>, String> {
    db.list_eventlog_dates().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_eventlog_on(db: State<'_, ImmicDb>, date: String) -> Result<Vec<EventLog>, String> {
    db.list_eventlog_on(date).await.map_err(|e| e.to_string())
}

// Interval

#[derive(Debug, Deserialize, Serialize)]
pub enum Interval {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

pub trait Timestamp {
    fn timestamp(&self) -> i64;
}

pub fn partition_logs<T: Timestamp>(logs: Vec<T>, local_time: &DateTime<Local>, interval: Interval) -> Result<Vec<(String, Vec<T>)>> {
    match interval {
        Interval::Hourly => {
            let mut ret = Vec::new();
            let mut ls = Vec::new();

            let mut hour = 0;
            let mut ts = local_time.with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().timestamp();
            for log in logs {
                let log_timestamp = log.timestamp();
                // assert!(log_timestamp >= ts, "log timestamp is less than ts: {} < {}", log_timestamp, ts);
                if log_timestamp < ts + 3600 {
                    ls.push(log);
                } else {
                    if ls.len() > 0 {
                        ret.push((format!("{hour:02}"), ls));
                        ls = Vec::new();
                    }
                    let dt = DateTime::from_timestamp(log_timestamp, 0).unwrap();
                    let lt = dt.with_timezone(&chrono::Local);
                    hour = lt.hour();
                    ts = local_time.with_hour(hour).unwrap().with_minute(0).unwrap().with_second(0).unwrap().timestamp();
                    ls.push(log);
                }
            }
            if ls.len() > 0 {
                ret.push((format!("{hour:02}"), ls));
            }

            Ok(ret)
        },
        _ => {
            Err(anyhow!("Not implemented yet"))
        }
    }
}

// pub fn first_logs<T: Timestamp>(logs: Vec<T>, local_time: &DateTime<Local>, interval: Interval) -> Result<Vec<(String, T)>> {
//     match interval {
//         Interval::Hourly => {
//             let mut ret = Vec::new();

//             let mut ts = local_time.with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().timestamp();
//             for log in logs {
//                 let log_timestamp = log.timestamp();
//                 if log_timestamp >= ts {
//                     let dt = DateTime::from_timestamp(log_timestamp, 0).unwrap();
//                     let local_time = dt.with_timezone(&Local);
//                     let hour = local_time.hour();
//                     ret.push((hour.to_string(), log));
//                     if hour == 23 {
//                         break;
//                     }
//                     ts = local_time.with_hour(hour + 1).unwrap().with_minute(0).unwrap().with_second(0).unwrap().timestamp();
//                 }
//             }

//             Ok(ret)
//         },
//         _ => {
//             Err(anyhow!("Not implemented yet"))
//         }
//     }
// }
