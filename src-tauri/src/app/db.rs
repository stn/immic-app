use anyhow::{ensure, Result};
use chrono::{DateTime, Utc};
use log::info;
use std::path::PathBuf;
use std::sync::OnceLock;
use sqlx::Sqlite;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use tauri::AppHandle;

use super::setting::with_setting;

#[derive(Debug, serde::Serialize)]
pub struct EventLog {
    pub id: i64,
    pub timestamp: i64,
    pub date: String,
    pub kind: String,
}

static POOL: OnceLock<sqlx::Pool<Sqlite>> = OnceLock::new();

pub fn pool() -> Option<&'static sqlx::Pool<Sqlite>> {
    POOL.get()
}

pub async fn init(app: &tauri::AppHandle) -> Result<()> {
    let path = db_path(app)?;

    POOL.get_or_init(|| {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);
        let pool = SqlitePoolOptions::new()
            .connect_lazy_with(options);
        pool
    });
    
    // TODO migrateのバージョンを確認して実行するべきかを判断する
    // after_connectを使うといいかもしれない。
    // https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html#method.after_connect
    migrate().await.unwrap();

    Ok(())
}

fn db_path(app: &AppHandle) -> Result<String> {
    let db_path_buf = with_setting(app.clone(), |store| {
        Ok(store.get("data-dir")
            .and_then(|v| v.as_str())
            .map(PathBuf::from))
    })?;
    ensure!(db_path_buf.is_some(), "data-dir is not set");

    let mut db_path_buf = db_path_buf.unwrap();
    db_path_buf.push("immic.db");
    let db_path = db_path_buf.to_string_lossy().to_string();

    info!("db_path: {}", db_path);

    Ok(db_path)
}

async fn migrate() -> sqlx::Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool().unwrap())
        .await?;
    Ok(())
}

pub async fn close() {
    if let Some(pool) = pool() {
        pool.close().await;
    }
}

pub async fn insert_eventlog(datetime: DateTime<Utc>, kind: &str) -> Result<i64> {
    // timestamp to date string in local timezone
    let ts = datetime.timestamp();
    let timeframe = ts / 60;
    let local_time = datetime.with_timezone(&chrono::Local);
    let date = local_time.format("%Y%m%d").to_string();
    let pool = pool();
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
    .execute(pool.unwrap()).await?;
    Ok(result.last_insert_rowid())
}

pub async fn update_eventlog_logid(id: i64, log_id: i64) -> Result<()> {
    let pool = pool();
    sqlx::query(
        r#"
        UPDATE event_log
        SET log_id = ?
        WHERE id = ?
        "#
    )
    .bind(log_id)
    .bind(id)
    .execute(pool.unwrap()).await?;
    Ok(())
}

#[tauri::command]
pub async fn list_eventlog_dates() -> Result<Vec<String>, String> {
    let result: Vec<String> = sqlx::query_as::<_, (String,)>(r#"
        SELECT DISTINCT date
        FROM event_log
        ORDER BY date DESC
        "#
    )
    .fetch_all(pool().unwrap())
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

#[tauri::command]
pub async fn list_eventlog_on(date: String) -> Result<Vec<EventLog>, String> {
    let result: Vec<EventLog> = sqlx::query_as::<_, (i64, i64, String, String)>(
        r#"
        SELECT id, timestamp, date, kind
        FROM event_log
        WHERE date = ?
        ORDER BY id
        "#
    )
    .bind(date)
    .fetch_all(pool().unwrap())
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

// pub async fn list_eventlog_by(date: &str, kind: &str) -> Result<Vec<EventLog>> {
//     let result: Vec<EventLog> = sqlx::query_as::<_, (i64, i64, String)>(
//         r#"
//         SELECT id, timestamp, date
//         FROM event_log
//         WHERE date = ? AND kind = ?
//         ORDER BY id
//         "#
//     )
//     .bind(date)
//     .bind(kind)
//     .fetch_all(pool())
//     .await?
//     .iter()
//     .map(|row| {
//         let (id, timestamp, date) = row;
//         EventLog {
//             id: *id,
//             timestamp: *timestamp,
//             date: date.clone(),
//             kind: kind.to_string(),
//         }
//     })
//     .collect();
//     Ok(result)
// }
