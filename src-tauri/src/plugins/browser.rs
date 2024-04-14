use actix_cors::Cors;
use actix_web::{
    post,
    http, middleware, web,
    App, HttpServer,
};
use anyhow::{anyhow, Context as _, Result};
use chrono::DateTime;
use futures::TryStreamExt;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use sqlx::{
    Pool,
    sqlite::Sqlite,
};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, State, Wry,
};

use crate::plugins::{
    db::ImmicDb,
    setting::SettingPlugin,
};

use super::db;

pub const KIND: &str = "browser";
const SERVER_PORT_SETTING: &str = "server-port";
const DEFAULT_SERVER_PORT: u16 = 3294;
const DEBOUNCE_THRESHOLD: i64 = 60;

pub fn init() -> TauriPlugin<Wry> {
    plugin::Builder::new("browser")
        .invoke_handler(tauri::generate_handler![
            list_browser_logs_on,
            get_browser_info,
        ])
        .setup(|app_handle| {
            debug!("browser plugin setup");
            let browser = BrowserPlugin::new(app_handle);
            app_handle.manage(browser);
            Ok(())
        })
        .build()
}

#[derive(Clone)]
pub struct BrowserPlugin {
    app: AppHandle,
}

impl BrowserPlugin {
    fn new(app: &AppHandle) -> Self {
        Self {
            app: app.clone(),
        }
    }

    async fn maybe_insert_info(&self, info: &TabInfo) -> Result<Option<i64>> {
        if self.check_debounce(info).await? {
            debug!("browsers: debounced!");
            return Ok(None);
        }
        let id = self.insert_info(info).await?;
        Ok(Some(id))
    }

    async fn insert_info(&self, info: &TabInfo) -> Result<i64> {
        assert!(info.url.is_some(), "url is required");

        let timestamp = DateTime::from_timestamp_millis(info.timestampMs).expect("Invalid timestamp");

        let db = self.app.state::<ImmicDb>();
        let event_id = db.insert_eventlog(timestamp, KIND).await?;

        let pool = db.pool().await.expect("db pool is not set");

        // Upsert browser_info by url
        let result = sqlx::query(
            r#"
            INSERT OR REPLACE INTO browser_info (id, url, fav_icon_url, last_update)
            VALUES (
                (SELECT id FROM browser_info WHERE url = ?),
                ?,
                ?,
                ?
            );
            "#
        )
        .bind(&info.url)
        .bind(&info.url)
        .bind(&info.favIconUrl)
        .bind(timestamp.timestamp())
        .execute(&pool)
        .await?;

        let info_id = result.last_insert_rowid();
        
        // Search browser_info by referrer
        let referrer_id = match &info.referrer {
            Some(referrer) => {
                if referrer.is_empty() {
                    None
                } else {
                    let result = sqlx::query(
                        r#"
                        INSERT OR REPLACE INTO browser_info (id, url)
                        VALUES (
                            (SELECT id FROM browser_info WHERE url = ?),
                            ?
                        );
                        "#
                    )
                    .bind(referrer)
                    .bind(referrer)
                    .execute(&pool)
                    .await?;

                    let referrer_id = result.last_insert_rowid();
                    Some(referrer_id)
                }
            },
            None => None,
        };

        let result = sqlx::query(
            r#"
            INSERT INTO browser_log (event_id, info_id, title, referrer_id, tab_id, opener_tab_id, window_id)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(event_id)
        .bind(info_id)
        .bind(info.title.as_ref())
        .bind(referrer_id)
        .bind(info.tabId)
        .bind(info.openerTabId)
        .bind(info.windowId)
        .execute(&pool)
        .await?;

        let log_id = result.last_insert_rowid();
        Ok(log_id)
    }

    pub async fn insert_browser_log_with(&self, pool: &Pool<Sqlite>, log: &BrowserLog) -> Result<i64> {
        let timestamp = DateTime::from_timestamp(log.timestamp, 0).context("Invalid timestamp")?;

        let db = self.app.state::<ImmicDb>();
        let event_id = db.insert_eventlog_with(pool, timestamp, KIND).await?;

        // Upsert browser_info by url
        let result = sqlx::query(
            r#"
            INSERT OR REPLACE INTO browser_info (id, url, fav_icon_url, last_update)
            VALUES (
                (SELECT id FROM browser_info WHERE url = ?),
                ?,
                ?,
                ?
            );
            "#
        )
        .bind(&log.url)
        .bind(&log.url)
        .bind(&log.fav_icon_url)
        .bind(timestamp.timestamp())
        .execute(pool)
        .await?;

        let info_id = result.last_insert_rowid();

        // Search browser_info by referrer
        let referrer_id = match &log.referrer {
            Some(referrer) => {
                if referrer.is_empty() {
                    None
                } else {
                    let result = sqlx::query(
                        r#"
                        INSERT OR REPLACE INTO browser_info (id, url)
                        VALUES (
                            (SELECT id FROM browser_info WHERE url = ?),
                            ?
                        );
                        "#
                    )
                    .bind(referrer)
                    .bind(referrer)
                    .execute(pool)
                    .await?;

                    let referrer_id = result.last_insert_rowid();
                    Some(referrer_id)
                }
            },
            None => None,
        };

        let result = sqlx::query(
            r#"
            INSERT INTO browser_log (event_id, info_id, title, referrer_id, tab_id, opener_tab_id, window_id)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(event_id)
        .bind(info_id)
        .bind(&log.title)
        .bind(referrer_id)
        .bind(log.tab_id)
        .bind(log.opener_tab_id)
        .bind(log.window_id)
        .execute(pool)
        .await?;

        // Update event_log with log_id
        let log_id = result.last_insert_rowid();
        // db.update_eventlog_logid_with(pool, event_id , log_id).await?;

        Ok(log_id)
    }

    async fn check_debounce(&self, info: &TabInfo) -> Result<bool> {
        let timestamp = DateTime::from_timestamp_millis(info.timestampMs).expect("Invalid timestamp");
        let timestamp = timestamp.timestamp();

        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await.context("db pool is not set")?;

        let result = sqlx::query_as::<_, (Option<i64>,)>(
            r#"
            SELECT last_update
            FROM browser_info
            WHERE url = ?
            "#
        )
        .bind(&info.url)
        .fetch_one(&pool)
        .await;

        if let Ok((last_update,)) = result {
            if last_update.is_none() {
                return Ok(false);
            }

            let last_update = last_update.unwrap();
            if timestamp - last_update < DEBOUNCE_THRESHOLD {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn list_browser_logs_on(&self, date: String) -> Result<Vec<BrowserLog>> {
        let db = self.app.state::<ImmicDb>();
        let pool = db.pool().await.context("db pool is not set")?;

        let mut rows = sqlx::query_as::<_, (
            i64, i64,
            i64, Option<String>, Option<i64>, Option<i64>, Option<i64>,
            i64, String, Option<String>,
            Option<String>,
        )>(
            r#"
            SELECT
            e.id, e.timestamp,
            b.id, b.title, b.tab_id, b.opener_tab_id, b.window_id,
            i.id, i.url, i.fav_icon_url,
            r.url
            FROM event_log e
            INNER JOIN browser_log b ON e.id = b.event_id
            INNER JOIN browser_info i ON b.info_id = i.id
            LEFT JOIN browser_info r ON b.referrer_id = r.id
            WHERE e.kind = ? AND e.date = ?
            ORDER BY e.id
            "#
        )
        .bind(KIND)
        .bind(&date)
        .fetch(&pool);

        let mut browserlogs = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let (
                event_id, timestamp,
                id, title, tab_id, opener_tab_id, window_id,
                info_id, url, fav_icon_url,
                referrer,
            ) = row;
            browserlogs.push(BrowserLog {
                id,
                event_id,
                timestamp,
                date: date.clone(),
                info_id,
                url,
                fav_icon_url,
                title,
                referrer,
                tab_id,
                opener_tab_id,
                window_id,
            });
        }
        Ok(browserlogs)
    }

    pub async fn get_browser_info(&self, browser_id: i64) -> Result<BrowserInfo> {
        debug!("get_browser_info: browser_id={}", browser_id);

        let db = self.app.state::<ImmicDb>();
        let pool = db.pool().await.expect("db pool is not set");

        let result = sqlx::query_as::<_, (i64, String, Option<String>, Option<i64>)>(
            r#"
            SELECT id, url, fav_icon_url, last_update
            FROM browser_info
            WHERE id = ?
            "#
        )
        .bind(browser_id)
        .fetch_one(&pool)
        .await;

        match result {
            Ok((id, url, fav_icon_url, last_update)) => {
                Ok(BrowserInfo {
                    id,
                    url,
                    fav_icon_url,
                    last_update,
                })
            },
            Err(e) => Err(anyhow!("Not found: {}", e)),
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[allow(non_snake_case)]
struct TabInfo {
  tabId: Option<i64>,
  url: Option<String>,
  title: Option<String>,
  favIconUrl: Option<String>,
  referrer: Option<String>,
  openerTabId: Option<i64>,
  windowId: Option<i64>,
  timestampMs: i64,
}

#[post("/api/v1/browserlog")]
pub async fn browserlog(tab_info: web::Json<TabInfo>, data: web::Data<AppHandle>) -> actix_web::Result<String> {
    debug!("tab_info: {:?}", tab_info);

    let browser_plugin = data.get_ref().state::<BrowserPlugin>();

    if let Err(e) = browser_plugin.maybe_insert_info(&tab_info).await {
        error!("Error on insert: {:?}", e);
        return Err(actix_web::error::ErrorInternalServerError(e));
    }

    Ok("ok".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowserLog {
    pub id: i64,
    pub event_id: i64,
    pub timestamp: i64,
    pub date: String,
    pub info_id: i64,
    pub url: String,
    pub fav_icon_url: Option<String>,
    pub title: Option<String>,
    pub referrer: Option<String>,
    pub tab_id: Option<i64>,
    pub opener_tab_id: Option<i64>,
    pub window_id: Option<i64>,
}

impl db::Timestamp for BrowserLog {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

#[derive(Debug, Serialize)]
pub struct BrowserInfo {
    pub id: i64,
    pub url: String,
    pub fav_icon_url: Option<String>,
    pub last_update: Option<i64>,
}

#[tauri::command]
pub async fn list_browser_logs_on(browser: State<'_, BrowserPlugin>, date: String) -> Result<Vec<BrowserLog>, String> {
    browser.list_browser_logs_on(date).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_browser_info(browser: State<'_, BrowserPlugin>, browser_id: i64) -> Result<BrowserInfo, String> {
    browser.get_browser_info(browser_id).await.map_err(|e| e.to_string())
}

// tokioでも動かせれるはず
// https://docs.rs/actix-web/4.5.1/actix_web/rt/index.html#running-actix-web-using-tokiomain
// BroserPlugin::startにできないか？
#[actix_web::main]
pub async fn init_server(app: AppHandle) -> std::io::Result<()> {
    let server_port = server_port(&app);

    let data = web::Data::new(app.clone());
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|_origin, _req_head| {
                true
            })
            .allowed_methods(vec!["GET", "POST"])
            // .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .service(browserlog)
    })
    .bind(("127.0.0.1", server_port))?
    .run()
    .await
}

fn server_port(app: &AppHandle) -> u16 {
    // debug!("db_path");
    let setting = app.state::<SettingPlugin>();
    let server_port = match setting.get(SERVER_PORT_SETTING) {
        Ok(Some(s)) => {
            match s.as_str() {
                Some(s) => {
                    match s.parse::<u16>() {
                        Ok(port) => port,
                        Err(e) => {
                            error!("Invalid port: {}", e);
                            DEFAULT_SERVER_PORT
                        }
                    }
                },
                None => {
                    error!("Invalid port: {:?}", s);
                    DEFAULT_SERVER_PORT
                }
            }
        },
        _ => DEFAULT_SERVER_PORT,
    };

    debug!("server_port: {}", server_port);

    server_port
}
