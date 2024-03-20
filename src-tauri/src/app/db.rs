use anyhow::Result;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use sqlx::Sqlite;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

#[derive(Debug)]
pub struct EventLog {
    id: i64,
    timestamp: i64,
    date: String,
    kind: String,
}

static POOL: Lazy<sqlx::Pool<Sqlite>> = Lazy::new(|| {
    let options = SqliteConnectOptions::new()
        .filename("../eventlog.db")
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
