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

    async fn insert_info(&self, info: &TabInfo) -> Result<i64> {
        assert!(info.url.is_some(), "url is required");

        let timestamp = DateTime::from_timestamp_millis(info.timestampMs).expect("Invalid timestamp");

        let db = self.app.state::<ImmicDb>();
        let event_id = db.insert_eventlog(timestamp, KIND).await?;

        // Search browser_info by url
        let pool = db.pool().await.expect("db pool is not set");
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT id
            FROM browser_info
            WHERE url = ?
            "#
        )
        .bind(&info.url)
        .fetch_one(&pool)
        .await;
        let info_id = match result {
            Ok((id,)) => id,
            Err(_) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO browser_info (url, fav_icon_url)
                    VALUES (?, ?)
                    "#
                )
                .bind(&info.url)
                .bind(&info.favIconUrl)
                .execute(&pool)
                .await?;
                result.last_insert_rowid()
            }
        };

        let result = sqlx::query(
            r#"
            INSERT INTO browser_log (event_id, info_id, title, referrer, tab_id, opener_tab_id, window_id)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(event_id)
        .bind(info_id)
        .bind(info.title.as_ref())
        .bind(info.referrer.as_ref())
        .bind(info.tabId)
        .bind(info.openerTabId)
        .bind(info.windowId)
        .execute(&pool)
        .await?;

        // Update event_log with log_id
        let log_id = result.last_insert_rowid();
        db.update_eventlog_logid(event_id , log_id).await?;

        Ok(log_id)
    }

    pub async fn insert_browser_log_with(&self, pool: &Pool<Sqlite>, log: &BrowserLog) -> Result<i64> {
        let timestamp = DateTime::from_timestamp(log.timestamp, 0).context("Invalid timestamp")?;

        let db = self.app.state::<ImmicDb>();
        let event_id = db.insert_eventlog_with(pool, timestamp, KIND).await?;

        // Search browser_info by url
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT id
            FROM browser_info
            WHERE url = ?
            "#
        )
        .bind(&log.url)
        .fetch_one(pool)
        .await;

        let info_id = match result {
            Ok((id,)) => id,
            Err(_) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO browser_info (url, fav_icon_url)
                    VALUES (?, ?)
                    "#
                )
                .bind(&log.url)
                .bind(&log.fav_icon_url)
                .execute(pool)
                .await?;

                result.last_insert_rowid()
            }
        };

        let result = sqlx::query(
            r#"
            INSERT INTO browser_log (event_id, info_id, title, referrer, tab_id, opener_tab_id, window_id)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(event_id)
        .bind(info_id)
        .bind(&log.title)
        .bind(&log.referrer)
        .bind(log.tab_id)
        .bind(log.opener_tab_id)
        .bind(log.window_id)
        .execute(pool)
        .await?;

        // Update event_log with log_id
        let log_id = result.last_insert_rowid();
        db.update_eventlog_logid_with(pool, event_id , log_id).await?;

        Ok(log_id)
    }

    pub async fn list_browser_logs_on(&self, date: String) -> Result<Vec<BrowserLog>> {
        let db = self.app.state::<ImmicDb>();
        let pool = db.pool().await.context("db pool is not set")?;

        let mut rows = sqlx::query_as::<_, (
            i64, i64, i64,
            i64, Option<String>, Option<String>, Option<i64>, Option<i64>, Option<i64>,
            i64, String, Option<String>,
        )>(
            r#"
            SELECT
            e.id, e.timestamp, e.timeframe,
            b.id, b.title, b.referrer, b.tab_id, b.opener_tab_id, b.window_id,
            i.id, i.url, i.fav_icon_url
            FROM event_log e
            INNER JOIN browser_log b ON e.log_id = b.id
            INNER JOIN browser_info i ON b.info_id = i.id
            WHERE e.kind = ? AND e.date = ?
            ORDER BY e.timestamp
            "#
        )
        .bind(KIND)
        .bind(&date)
        .fetch(&pool);

        let mut browserlogs = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let (
                event_id, timestamp, timeframe,
                id, title, referrer, tab_id, opener_tab_id, window_id,
                info_id, url, fav_icon_url,
            ) = row;
            browserlogs.push(BrowserLog {
                id,
                event_id,
                timestamp,
                timeframe,
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

        let result = sqlx::query_as::<_, (i64, String, Option<String>)>(
            r#"
            SELECT id, url, fav_icon_url
            FROM browser_info
            WHERE id = ?
            "#
        )
        .bind(browser_id)
        .fetch_one(&pool)
        .await;

        match result {
            Ok((id, url, fav_icon_url)) => {
                Ok(BrowserInfo {
                    id,
                    url,
                    fav_icon_url,
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

    if let Err(e) = browser_plugin.insert_info(&tab_info).await {
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
    pub timeframe: i64,
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
