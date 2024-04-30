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
use std::collections::HashMap;
use sqlx::{
    sqlite::Sqlite,
    Pool,
};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, State, Wry,
};
use url::Url;

use crate::{
    app::event::{
        emit_event_to_info,
        ImmicEvent,
    },
    plugins::{
        db::{self, ImmicDb},
        search::{
            HitsPerDay,
            SearchHit,
        },
        setting::SettingPlugin,
    },
};

pub const KIND: &str = "browser";
const SERVER_PORT_SETTING: &str = "server-port";
const DEFAULT_SERVER_PORT: u16 = 3294;

// TODO: set in setting
const DEBOUNCE_THRESHOLD: i64 = 60;

pub fn init() -> TauriPlugin<Wry> {
    plugin::Builder::new("browser")
        .invoke_handler(tauri::generate_handler![
            list_browser_logs_on,
            // get_browser_info,
        ])
        .setup(|app_handle| {
            debug!("browser plugin setup");
            let browser = BrowserPlugin::new(app_handle.clone());
            app_handle.manage(browser);
            Ok(())
        })
        .build()
}

pub struct BrowserPlugin {
    app: AppHandle,
}

impl BrowserPlugin {
    fn new(app: AppHandle) -> Self {
        Self {
            app
        }
    }

    pub fn start(&self) -> Result<()> {
        debug!("browser plugin start");

        let app = self.app.clone();
        std::thread::spawn(move || {
            init_server(app).unwrap_or_else(|e| {
                error!("Browser server error: {}", e);
            });
        });

        Ok(())
    }

    pub async fn stop(&self) {
        debug!("browser plugin stop");
    }

    // async fn maybe_insert_info(&self, info: TabInfo) -> Result<Option<i64>> {
    //     if self.check_debounce(&info).await? {
    //         debug!("browsers: debounced!");
    //         return Ok(None);
    //     }
    //     let id = self.insert_info(info).await?;
    //     Ok(Some(id))
    // }

    async fn maybe_insert_browser_log_and_emit(&self, log: BrowserLog) -> Result<Option<BrowserLog>> {
        if self.check_debounce(&log).await? {
            debug!("browsers: debounced!");
            return Ok(None);
        }

        let result = self.insert_browser_log(log).await?;

        // search for the result
        let hits = self.search_for(&result).await?;

        // send an event
        let event = ImmicEvent::Browser(result.clone(), hits);
        emit_event_to_info(&self.app, event).context("Failed to emit event")?;

        Ok(Some(result))
    }

    async fn insert_browser_log(&self, log: BrowserLog) -> Result<BrowserLog> {
        let db = self.app.state::<ImmicDb>();
        let pool = db.pool().await.context("db pool is not set")?;

        self.insert_browser_log_with(&pool, log).await
    }

    // async fn insert_info(&self, info: TabInfo) -> Result<i64> {
    //     ensure!(info.url.is_some(), "url is required");

    //     let db = self.app.state::<ImmicDb>();
    //     let pool = db.pool().await.context("db pool is not set")?;

    //     let timestamp = DateTime::from_timestamp_millis(info.timestampMs).context("Invalid timestamp")?;
    //     let browser_log = BrowserLog {
    //         id: 0,  // dummy
    //         timestamp: timestamp.timestamp(),
    //         date: "".to_string(),  // dummy
    //         url: info.url.unwrap(),  // checked in the above
    //         title: info.title,
    //         fav_icon_url: info.favIconUrl,
    //         referrer: info.referrer,
    //         tab_id: info.tabId,
    //         opener_tab_id: info.openerTabId,
    //         window_id: info.windowId,
    //     };

    //     self.insert_browser_log_with(&pool, browser_log).await
    // }

    pub async fn insert_browser_log_with(&self, pool: &Pool<Sqlite>, log: BrowserLog) -> Result<BrowserLog> {
        let timestamp = DateTime::from_timestamp(log.timestamp, 0).context("Invalid timestamp")?;

        let db = self.app.state::<ImmicDb>();
        let event_log = db.insert_eventlog_with(pool, &timestamp, KIND).await?;

        let (origin, url, query) = parse_url(&log.url)?;

        // browser_origin
        let result = sqlx::query_as::<_, (i64, Option<String>)>(
            r#"
            SELECT id, fav_icon_url
            FROM browser_origin
            WHERE origin = ?
            "#
        )
        .bind(&origin)
        .fetch_one(pool)
        .await;
        
        let origin_id = match result {
            Ok((id, fav_icon_url)) => {
                if fav_icon_url.is_none() && log.fav_icon_url.is_some() {
                    sqlx::query(
                        r#"
                        UPDATE browser_origin SET fav_icon_url = ? WHERE id = ?
                        "#
                    )
                    .bind(&log.fav_icon_url)
                    .bind(id)
                    .execute(pool)
                    .await?;
                }
                id
            },
            Err(_) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO browser_origin (origin, fav_icon_url) VALUES (?, ?)
                    "#
                )
                .bind(&origin)
                .bind(&log.fav_icon_url)
                .execute(pool)
                .await?;
                result.last_insert_rowid()
            }
        };

        // browser_url
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT id
            FROM browser_url
            WHERE url = ?
            "#
        )
        .bind(&url)
        .fetch_one(pool)
        .await;

        let url_id = match result {
            Ok((id,)) => {
                sqlx::query(
                    r#"
                    UPDATE browser_url SET last_update = ? WHERE id = ?
                    "#
                )
                .bind(timestamp.timestamp())
                .bind(id)
                .execute(pool)
                .await?;
                id
            },
            Err(_) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO browser_url (url, origin_id, last_update) VALUES (?, ?, ?)
                    "#
                )
                .bind(&url)
                .bind(origin_id)
                .bind(timestamp.timestamp())
                .execute(pool)
                .await?;
                result.last_insert_rowid()
            }
        };

        let referrer_id = match &log.referrer {
            Some(referrer) => {
                match parse_url(referrer) {
                    Ok((origin, url, _query)) => {
                        // browser_origin for referrer
                        let result = sqlx::query_as::<_, (i64,)>(
                            r#"
                            SELECT id
                            FROM browser_origin
                            WHERE origin = ?
                            "#
                        )
                        .bind(&origin)
                        .fetch_one(pool)
                        .await;
                        let origin_id = match result {
                            Ok((id,)) => id,
                            Err(_) => {
                                let result = sqlx::query(
                                    r#"
                                    INSERT INTO browser_origin (origin) VALUES (?)
                                    "#
                                )
                                .bind(&origin)
                                .execute(pool)
                                .await?;
                                result.last_insert_rowid()
                            }
                        };

                        // browser_url for referrer
                        let result = sqlx::query_as::<_, (i64,)>(
                            r#"
                            SELECT id
                            FROM browser_url
                            WHERE url = ?
                            "#
                        )
                        .bind(&url)
                        .fetch_one(pool)
                        .await;
                        let url_id = match result {
                            Ok((id,)) => id,
                            Err(_) => {
                                let result = sqlx::query(
                                    r#"
                                    INSERT INTO browser_url (url, origin_id) VALUES (?, ?)
                                    "#
                                )
                                .bind(&url)
                                .bind(origin_id)
                                .execute(pool)
                                .await?;
                                result.last_insert_rowid()
                            }
                        };
                        Some(url_id)
                    },
                    Err(e) => {
                        error!("Error on parse_url(referrer {}): {}", referrer, e);
                        None
                    }
                }
            },
            None => None,
        };
        
        // INSERT INTO browser_log
        let result = sqlx::query(
            r#"
            INSERT INTO browser_log (event_id, origin_id, url_id, url_query, title, referrer_id, tab_id, opener_tab_id, window_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(event_log.id)
        .bind(origin_id)
        .bind(url_id)
        .bind(query)
        .bind(&log.title)
        .bind(referrer_id)
        .bind(log.tab_id)
        .bind(log.opener_tab_id)
        .bind(log.window_id)
        .execute(pool)
        .await?;

        let log_id = result.last_insert_rowid();

        let browser_log = BrowserLog {
            id: log_id,
            event_id: event_log.id,
            timestamp: log.timestamp,
            date: event_log.date,
            url_id,
            url,
            title: log.title,
            fav_icon_url: log.fav_icon_url,
            referrer: log.referrer,
            tab_id: log.tab_id,
            opener_tab_id: log.opener_tab_id,
            window_id: log.window_id,
        };

        Ok(browser_log)
    }

    async fn check_debounce(&self, log: &BrowserLog) -> Result<bool> {
        let (_origin, url, _query) = parse_url(&log.url)?;

        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await.context("db pool is not set")?;

        let result = sqlx::query_as::<_, (Option<i64>,)>(
            r#"
            SELECT last_update
            FROM browser_url
            WHERE url = ?
            "#
        )
        .bind(&url)
        .fetch_one(&pool)
        .await;

        if let Ok((Some(last_update),)) = result {
            if log.timestamp - last_update < DEBOUNCE_THRESHOLD {
                // faviconの更新があるケースがあるのをどうするか。
                // faviconの更新だけここで行うか？
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn list_browser_logs_on(&self, date: &str) -> Result<Vec<BrowserLog>> {
        let db = self.app.state::<ImmicDb>();
        let pool = db.pool().await.context("db pool is not set")?;

        let mut rows = sqlx::query_as::<_, (
            i64, i64,
            i64, Option<String>, Option<String>, Option<i64>, Option<i64>, Option<i64>,
            Option<String>,
            i64, String,
            Option<String>,
        )>(
            r#"
            SELECT
            e.id, e.timestamp,
            b.id, b.url_query, b.title, b.tab_id, b.opener_tab_id, b.window_id,
            o.fav_icon_url,
            u.id, u.url,
            r.url
            FROM event_log e
            INNER JOIN browser_log b ON e.id = b.event_id
            INNER JOIN browser_origin o ON b.origin_id = o.id
            INNER JOIN browser_url u ON b.url_id = u.id
            LEFT JOIN browser_url r ON b.referrer_id = r.id
            WHERE e.kind = ? AND e.date = ?
            ORDER BY e.id
            "#
        )
        .bind(KIND)
        .bind(date)
        .fetch(&pool);

        let mut browserlogs = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let (
                event_id, timestamp,
                id, url_query, title, tab_id, opener_tab_id, window_id,
                fav_icon_url,
                url_id, url,
                referrer,
            ) = row;
            let url = match url_query {
                Some(url_query) => {
                    if url_query.is_empty() {
                        url
                    } else {
                        format!("{}?{}", url, url_query)
                    }
                },
                None => url,
            };
            browserlogs.push(BrowserLog {
                id,
                event_id,
                timestamp,
                date: date.to_string(),
                url_id,
                url,
                title,
                fav_icon_url,
                referrer,
                tab_id,
                opener_tab_id,
                window_id,
            });
        }
        Ok(browserlogs)
    }

    // pub async fn get_browser_info(&self, browser_id: i64) -> Result<BrowserInfo> {
    //     debug!("get_browser_info: browser_id={}", browser_id);

    //     let db = self.app.state::<ImmicDb>();
    //     let pool = db.pool().await.expect("db pool is not set");

    //     let result = sqlx::query_as::<_, (i64, String, Option<String>, Option<i64>)>(
    //         r#"
    //         SELECT id, url, fav_icon_url, last_update
    //         FROM browser_info
    //         WHERE id = ?
    //         "#
    //     )
    //     .bind(browser_id)
    //     .fetch_one(&pool)
    //     .await;

    //     match result {
    //         Ok((id, url, fav_icon_url, last_update)) => {
    //             Ok(BrowserInfo {
    //                 id,
    //                 url,
    //                 fav_icon_url,
    //                 last_update,
    //             })
    //         },
    //         Err(e) => Err(anyhow!("Not found: {}", e)),
    //     }
    // }

    pub async fn search_for(&self, log: &BrowserLog) -> Result<Vec<HitsPerDay>> {
        let db = self.app.state::<db::ImmicDb>();
        let pool = db.pool().await?;

        let mut rows = sqlx::query_as::<_, (
            i64,
            i64, String,
        )>(
            r#"
            SELECT
                b.id,
                e.timestamp, e.date
            FROM browser_log b
            INNER JOIN event_log e
                ON b.url_id = ?
                AND b.event_id = e.id
            ORDER BY e.timestamp
            "#
        )
        .bind(log.url_id)
        .fetch(&pool);

        let mut hits: HashMap<String, HitsPerDay> = HashMap::new();
        while let Some((id, timestamp, date)) = rows.try_next().await? {
            hits.entry(date.clone())
                .and_modify(|h| {
                    h.count += 1;
                    h.browser_url_count = h.browser_url_count.map(|c| c + 1);
                    h.browser_url_hits.as_mut().map(|hits| {
                        hits.push(SearchHit {
                            id,
                            timestamp,
                        });
                    });
                })
                .or_insert_with(|| {
                    let mut h = HitsPerDay::default();
                    h.date = date;
                    h.count = 1;
                    h.browser_url_count = Some(1);
                    h.browser_url_hits = Some(vec![SearchHit {
                        id,
                        timestamp,
                    }]);
                    h
                });
        }

        let mut hits: Vec<HitsPerDay> = hits
            .into_iter()
            .map(|(_date, h)| h)
            .collect();
        hits.sort_by(|a, b| a.date.cmp(&b.date).reverse());

        Ok(hits)
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

    if tab_info.url.is_none() || tab_info.url.as_ref().unwrap().is_empty() {
        return Err(actix_web::error::ErrorBadRequest("url is required"));
    }

    let timestamp = DateTime::from_timestamp_millis(tab_info.timestampMs);
    if timestamp.is_none() {
        return Err(actix_web::error::ErrorBadRequest("Invalid timestamp"));
    }

    let browser_log = BrowserLog {
        id: 0,  // dummy
        event_id: 0,  // dummy
        timestamp: timestamp.unwrap().timestamp(),
        date: "".to_string(),  // dummy
        url_id: 0,  // dummy
        url: tab_info.url.clone().unwrap(),
        title: tab_info.title.clone(),
        fav_icon_url: tab_info.favIconUrl.clone(),
        referrer: tab_info.referrer.clone(),
        tab_id: tab_info.tabId,
        opener_tab_id: tab_info.openerTabId,
        window_id: tab_info.windowId,
    };

    let browser_plugin = data.get_ref().state::<BrowserPlugin>();
    if let Err(e) = browser_plugin.maybe_insert_browser_log_and_emit(browser_log).await {
        error!("Error on insert: {:?}", e);
        return Err(actix_web::error::ErrorInternalServerError(e));
    }

    Ok("ok".to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrowserLog {
    pub id: i64,
    pub event_id: i64,
    pub timestamp: i64,
    pub date: String,
    pub url_id: i64,
    pub url: String,
    pub title: Option<String>,
    pub fav_icon_url: Option<String>,
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

// #[derive(Debug, Serialize)]
// pub struct BrowserInfo {
//     pub id: i64,
//     pub url: String,
//     pub fav_icon_url: Option<String>,
//     pub last_update: Option<i64>,
// }

fn parse_url(url: &str) -> Result<(String, String, Option<String>)> {
    let parsed = Url::parse(url)?;
    let query = match &parsed[url::Position::BeforeQuery..] {
        s if s.is_empty() => None,
        s => Some(s.to_string()),
    };

    // actually this is not origin but we don't want to keep username and password parts.
    let origin = parsed.host_str().ok_or_else(|| anyhow!("Invalid url"))?;

    Ok((
        origin.to_string(),
        parsed[..url::Position::AfterPath].to_string(),
        query,
    ))
}

#[tauri::command]
pub async fn list_browser_logs_on(browser: State<'_, BrowserPlugin>, date: String) -> Result<Vec<BrowserLog>, String> {
    browser.list_browser_logs_on(&date).await.map_err(|e| e.to_string())
}

// #[tauri::command]
// pub async fn get_browser_info(browser: State<'_, BrowserPlugin>, browser_id: i64) -> Result<BrowserInfo, String> {
//     browser.get_browser_info(browser_id).await.map_err(|e| e.to_string())
// }

// tokioでも動かせれるはず
// https://docs.rs/actix-web/4.5.1/actix_web/rt/index.html#running-actix-web-using-tokiomain
// BroserPlugin::startにできないか？
#[actix_web::main]
async fn init_server(app: AppHandle) -> std::io::Result<()> {
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
