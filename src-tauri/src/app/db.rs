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
