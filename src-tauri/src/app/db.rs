use anyhow::Result;
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

pub fn pool() -> &'static sqlx::Pool<Sqlite> {
    POOL.get().expect("DB pool is not initialized")
}

pub async fn init(app: &tauri::AppHandle) {
    POOL.get_or_init(|| {
        let options = SqliteConnectOptions::new()
            .filename(db_path(&app))
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
}

fn db_path(app: &AppHandle) -> String {
    let data_dir: Option<PathBuf> = with_setting(app.clone(), |store| {
        match store.get("data-dir").and_then(|v| v.as_str()) {
            Some(dir) => Ok(Some(PathBuf::from(dir))),
            None => Ok(app.path_resolver().app_data_dir()),
        }
    }).expect("failed to get data-dir");
    let data_dir = data_dir.map_or(".".to_string(), |v| v.to_string_lossy().to_string());
    let db_path = data_dir + "/immic.db";
    info!("db_path: {}", db_path);
    db_path
}

async fn migrate() -> sqlx::Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool())
        .await?;
    Ok(())
}

pub async fn close() {
    pool().close().await;
}

pub async fn insert_eventlog(datetime: DateTime<Utc>, kind: &str) -> Result<i64> {
    // timestamp to date string in local timezone
    let local_time = datetime.with_timezone(&chrono::Local);
    let date = local_time.format("%Y%m%d").to_string();
    let pool = pool();
    let result = sqlx::query(
        r#"
        INSERT INTO event (timestamp, date, kind)
        VALUES (?, ?, ?)
        "#
    )
    .bind(datetime.timestamp_millis())
    .bind(date)
    .bind(kind)
    .execute(pool).await?;
    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn list_eventlog_dates() -> Result<Vec<String>, String> {
    let result: Vec<String> = sqlx::query_as::<_, (String,)>(r#"
        SELECT DISTINCT date
        FROM event
        ORDER BY date DESC
        "#
    )
    .fetch_all(pool())
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
        FROM event
        WHERE date = ?
        ORDER BY id
        "#
    )
    .bind(date)
    .fetch_all(pool())
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
//         FROM event
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
