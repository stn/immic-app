use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, Timelike, Utc};
use futures::TryStreamExt;
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
use tokio::{
    fs::File,
    io::{
        AsyncBufReadExt,
        AsyncWriteExt,
        BufReader,
        BufWriter,
    },
};

use crate::plugins::{
    application::{ApplicationPlugin, ApplicationLog},
    browser::{BrowserPlugin, BrowserLog},
    filelog::{FilelogPlugin, FileLog},
    screenshot::{ScreenshotPlugin, ScreenshotLog},
    setting::SettingPlugin,
};

const DATABASE_FILE: &str = "immic.db";
const DATA_DIR_SETTING: &str = "data-dir";
const IMPORT_DATABASE_FILE: &str = "immic-imported.db";

pub fn init() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("immicdb")
        .invoke_handler(tauri::generate_handler![
            list_eventlog_dates,
            list_any_logs_on,
            export_logs,
            import_logs,
        ])
        .setup(move |app| {
            debug!("immicdb plugin setup");

            let db = ImmicDb::new(app.clone());
            app.manage(db);

            Ok(())
        })
        .on_event(|app, event| {
            match event {
                RunEvent::Exit => {
                    debug!("RunEvent::Exit");
                    let db = app.state::<ImmicDb>();
                    let pool = db.pool.lock().unwrap().clone();
                    if let Some(pool) = pool {
                        tokio::spawn(async move {
                            pool.close().await;
                        });
                    }
                },
                _ => (),
            }
        })
        .build()
}

pub struct ImmicDb {
    app: AppHandle,
    pool: Mutex<Option<Pool<Sqlite>>>,
}

impl ImmicDb {
    fn new(app: AppHandle) -> Self {
        Self {
            app,
            pool: Mutex::new(None),
        }
    }

    pub async fn pool(&self) -> Result<Pool<Sqlite>> {
        self.pool.lock().unwrap().clone()
            .ok_or_else(|| anyhow!("pool is not initialized"))
    }

    pub fn start(&self) -> Result<()> {
        debug!("start immicdb");

        if self.pool.lock().unwrap().is_some() {
            return Ok(());
        }

        let path = self.db_path()?;
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
        self.db_path_with(DATABASE_FILE)
    }

    fn import_db_path(&self) -> Result<PathBuf> {
        self.db_path_with(IMPORT_DATABASE_FILE)
    }

    fn db_path_with(&self, database_file: &str) -> Result<PathBuf> {
        let setting = self.app.state::<SettingPlugin>();
        let data_dir = setting.get(DATA_DIR_SETTING)?
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .map(PathBuf::from);
        if data_dir.is_none() {
            return Err(anyhow!("{} is not set", DATA_DIR_SETTING));
        }

        let db_path = data_dir.unwrap().join(database_file);
        Ok(db_path)
    }

    pub async fn migrate(&self) -> Result<()> {
        // debug!("migrate immicdb");
        let pool = self.pool().await?;
        self.migrate_with(&pool).await?;
        Ok(())
    }

    async fn migrate_with(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(pool)
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
        self.insert_eventlog_with(&pool, datetime, kind).await
    }

    pub async fn insert_eventlog_with(&self, pool: &Pool<Sqlite>, datetime: DateTime<Utc>, kind: &str) -> Result<i64> {
        // timestamp to date string in local timezone
        let ts = datetime.timestamp();
        let local_time = datetime.with_timezone(&chrono::Local);
        let date = local_time.format("%Y%m%d").to_string();
        let result = sqlx::query(
            r#"
            INSERT INTO event_log (timestamp, date, kind)
            VALUES (?, ?, ?)
            "#
        )
        .bind(ts)
        .bind(date)
        .bind(kind)
        .execute(pool).await?;
        Ok(result.last_insert_rowid())
    }

    // pub async fn update_eventlog_logid(&self, id: i64, log_id: i64) -> Result<()> {
    //     let pool = self.pool().await?;
    //     self.update_eventlog_logid_with(&pool, id, log_id).await
    // }

    // pub async fn update_eventlog_logid_with(&self, pool: &Pool<Sqlite>, id: i64, log_id: i64) -> Result<()> {
    //     sqlx::query(
    //         r#"
    //         UPDATE event_log
    //         SET log_id = ?
    //         WHERE id = ?
    //         "#
    //     )
    //     .bind(log_id)
    //     .bind(id)
    //     .execute(pool).await?;
    //     Ok(())
    // }

    pub async fn list_eventlog_dates(&self) -> Result<Vec<String>> {
        let pool = self.pool().await?;
        let mut rows = sqlx::query_as::<_, (String,)>(r#"
            SELECT DISTINCT date
            FROM event_log
            ORDER BY date DESC
            "#
        )
        .fetch(&pool);

        let mut dates: Vec<String> = Vec::new();
        while let Some((date,)) = rows.try_next().await? {
            dates.push(date);
        }
        Ok(dates)
    }

    pub async fn list_any_logs_on(&self, date: String) -> Result<Vec<AnyLog>> {
        let application = self.app.state::<ApplicationPlugin>();
        let application_logs = application.list_application_logs_on(date.clone()).await?;

        let browser = self.app.state::<BrowserPlugin>();
        let browser_logs = browser.list_browser_logs_on(date.clone()).await?;

        let filelog = self.app.state::<FilelogPlugin>();
        let file_logs = filelog.list_file_logs_on(date.clone()).await?;

        let screenshot = self.app.state::<ScreenshotPlugin>();
        let screenshot_logs = screenshot.list_screenshot_logs_on(date.clone()).await?;

        Ok(application_logs.into_iter().map(AnyLog::ApplicationLogEntry)
            .chain(browser_logs.into_iter().map(AnyLog::BrowserLogEntry))
            .chain(file_logs.into_iter().map(AnyLog::FileLogEntry))
            .chain(screenshot_logs.into_iter().map(AnyLog::ScreenshotLogEntry))
            .collect())
    }

    pub async fn export_logs(&self, filename: String) -> Result<()> {
        let file = File::options()
            .write(true)
            .create_new(true)
            .open(filename)
            .await?;
        let mut writer = BufWriter::new(file);
        let mut dates = self.list_eventlog_dates().await?;
        dates.sort();
        for date in dates.into_iter() {
            let logs = self.list_any_logs_on(date).await?;
            for log in logs.into_iter() {
                let line = serde_json::to_string(&log)?;
                writer.write(line.as_bytes()).await?;
                writer.write(b"\n").await?;
            }
            writer.flush().await?;
        }

        Ok(())
    }

    pub async fn import_logs(&self, filename: String) -> Result<()> {
        debug!("import_logs: {}", filename);

        let file = File::open(filename).await?;
        let mut reader = BufReader::new(file);

        let application = self.app.state::<ApplicationPlugin>();
        let browser = self.app.state::<BrowserPlugin>();
        let filelog = self.app.state::<FilelogPlugin>();
        let screenshot = self.app.state::<ScreenshotPlugin>();

        let path = self.import_db_path()?;
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);
        let pool = SqlitePoolOptions::new().connect_with(options).await?;

        self.migrate_with(&pool).await?;

        let mut last_application_log: Option<(i64, i64)> = None;

        let mut line = String::new();
        while reader.read_line(&mut line).await? > 0 {
            let log: AnyLog = serde_json::from_str(&line)?;
            debug!("import {:?}", log);
            match log {
                AnyLog::ApplicationLogEntry(log) => {
                    if log.ref_id.is_none() {
                        let ids = application.insert_application_log_with(&pool, &log).await?;
                        last_application_log.replace(ids);
                    } else {
                        application.insert_application_log_ref_with(&pool, &log, &last_application_log).await?;
                    }
                },
                AnyLog::BrowserLogEntry(log) => {
                    browser.insert_browser_log_with(&pool, &log).await?;
                },
                AnyLog::FileLogEntry(log) => {
                    filelog.insert_file_log_with(&pool, &log).await?;
                },
                AnyLog::ScreenshotLogEntry(log) => {
                    screenshot.insert_screenshot_log_with(&pool, &log).await?;
                },
            }
            line.clear();
        }

        pool.close().await;

        Ok(())
    }
}


// EventLog

#[derive(Debug, Serialize, Deserialize)]
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
pub async fn list_any_logs_on(db: State<'_, ImmicDb>, date: String) -> Result<Vec<AnyLog>, String> {
    db.list_any_logs_on(date).await.map_err(|e| e.to_string())
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
                // can be happened when the timezone is changed
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


// Export and Import

#[derive(Debug, Deserialize, Serialize)]
pub enum AnyLog {
    ApplicationLogEntry(ApplicationLog),
    BrowserLogEntry(BrowserLog),
    FileLogEntry(FileLog),
    ScreenshotLogEntry(ScreenshotLog),
}

#[tauri::command]
pub async fn export_logs(db: State<'_, ImmicDb>, filename: String) -> Result<(), String> {
    db.export_logs(filename).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_logs(db: State<'_, ImmicDb>, filename: String) -> Result<(), String> {
    db.import_logs(filename).await.map_err(|e| e.to_string())
}
