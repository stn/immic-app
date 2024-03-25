use anyhow::Result;
use chrono::{DateTime, Utc};
use log::debug;
use once_cell::sync::Lazy;
use sqlx::Sqlite;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

#[derive(Debug, serde::Serialize)]
pub struct EventLog {
    pub id: i64,
    pub timestamp: i64,
    pub date: String,
    pub kind: String,
}

static POOL: Lazy<sqlx::Pool<Sqlite>> = Lazy::new(|| {
    let options = SqliteConnectOptions::new()
        .filename(db_path())
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);
    let pool = SqlitePoolOptions::new()
        .connect_lazy_with(options);
    pool
});

pub fn pool() -> &'static sqlx::Pool<Sqlite> {
    &*POOL
}

pub async fn init() {
    // TODO migrateのバージョンを確認して実行するべきかを判断する
    // after_connectを使うといいかもしれない。
    // https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html#method.after_connect
    migrate().await.unwrap();
}

async fn migrate() -> sqlx::Result<()> {
    sqlx::migrate!("./migrations")
        .run(&*POOL)
        .await?;
    Ok(())
}

pub async fn close() {
    POOL.close().await;
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

fn db_path() -> String {
    if let Ok(path) = std::env::var("DB_PATH") {
        debug!("DB_PATH: {:?}", path);
        if path != "" {
            return path;
        }
    }
    "immic.db".to_string()
}
