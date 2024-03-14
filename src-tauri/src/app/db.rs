use async_cron_scheduler::Scheduler;
use chrono::Local;
use once_cell::sync::Lazy;
use sqlx::Sqlite;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

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

pub async fn init() {
    // TODO migrateのバージョンを確認して実行するべきかを判断する
    // after_connectを使うといいかもしれない。
    // https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html#method.after_connect
    migrate().await.unwrap();
}

// impl Database {
//     pub async fn new() -> Self {
//         let pool = connect().await.unwrap();
//         migrate(&pool).await.unwrap();
//         Self { pool }
//     }
//
//     pub async fn insert(&self, table: String, key: String, value: String) -> sqlx::Result<()> {
//         sqlx::query(
//             format!(
//                 "INSERT INTO {} (key, value) VALUES (?, ?)",
//                 table
//             ).as_str()
//         )
//             .bind(key)
//             .bind(value)
//             .execute(&self.pool)
//             .await?;
//         Ok(())
//     }
// }

// async fn connect() -> sqlx::Result<sqlx::SqlitePool> {
//     let options = SqliteConnectOptions::new()
//         .filename("eventlog.db")
//         .create_if_missing(true)
//         .journal_mode(SqliteJournalMode::Wal)
//         .synchronous(SqliteSynchronous::Normal);
//     let pool = SqlitePoolOptions::new()
//         .connect_with(options)
//         .await?;
//     Ok(pool)
// }

async fn migrate() -> sqlx::Result<()> {
    sqlx::migrate!()
        .run(&*POOL)
        .await?;
    Ok(())
}

// pub async fn insert(table: String, key: String, value: String) -> sqlx::Result<()> {
//     sqlx::query(
//         format!(
//             "INSERT INTO {} (key, value) VALUES (?, ?)",
//             table
//         ).as_str()
//     )
//         .bind(key)
//         .bind(value)
//         .execute(&*POOL)
//         .await?;
//     Ok(())
// }

#[macro_export]
macro_rules! execute (
    ($query:expr) => ({
        sqlx::query($query).execute(&*POOL)
    });
);
