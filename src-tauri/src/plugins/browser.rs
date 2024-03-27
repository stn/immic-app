// use std::sync::{Arc, Mutex};
use actix_web::{post, web};
use anyhow::Result;
use chrono::DateTime;
use log::{debug, error};

use crate::app::db;
// use crate::plugins::Plugin;

const KIND: &str = "browser";

// pub struct BrowserPlugin {
//     // running: Arc<Mutex<bool>>,
// }

// impl BrowserPlugin {
//     pub fn new() -> BrowserPlugin {
//         BrowserPlugin {
//             // running: Arc::new(Mutex::new(false)),
//         }
//     }
// }

// impl Plugin for BrowserPlugin {
//     fn start(&mut self) {
//         // *self.running.lock().unwrap() = true;
//         // let running = Arc::clone(&self.running);
//     }

//     fn stop(&mut self) {
//         // *self.running.lock().unwrap() = false;
//     }
// }

#[derive(Debug, PartialEq, serde::Deserialize)]
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

impl TabInfo {
    async fn insert(&self) -> Result<i64> {
        assert!(self.url.is_some(), "url is required");

        let timestamp = DateTime::from_timestamp_millis(self.timestampMs).expect("Invalid timestamp");
        let event_id = db::insert_eventlog(timestamp, KIND).await?;

        // Search browser_info by url
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT id
            FROM browser_info
            WHERE url = ?
            "#
        )
        .bind(&self.url)
        .fetch_one(db::pool().unwrap())
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
                .bind(&self.url)
                .bind(&self.favIconUrl)
                .execute(db::pool().unwrap())
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
        .bind(self.title.as_ref())
        .bind(self.referrer.as_ref())
        .bind(self.tabId)
        .bind(self.openerTabId)
        .bind(self.windowId)
        .execute(db::pool().unwrap())
        .await?;

        // Update event_log with log_id
        let log_id = result.last_insert_rowid();
        db::update_eventlog_logid(event_id , log_id).await?;
        Ok(log_id)
    }
}

#[post("/api/v1/browserlog")]
pub async fn browserlog(tab_info: web::Json<TabInfo>) -> actix_web::Result<String> {
    debug!("tab_info: {:?}", tab_info);

    if let Err(e) = tab_info.insert().await {
        error!("Error on insert: {:?}", e);
        return Err(actix_web::error::ErrorInternalServerError(e));
    }

    Ok("ok".to_string())
}

#[derive(Debug, serde::Serialize)]
pub struct BrowserLog {
    pub id: i64,
    pub event_id: i64,
    pub timestamp: i64,
    pub date: String,
    pub info_id: i64,
    pub title: Option<String>,
    pub referrer: Option<String>,
    pub tab_id: Option<i64>,
    pub opener_tab_id: Option<i64>,
    pub window_id: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct BrowserInfo {
    pub id: i64,
    pub url: String,
    pub fav_icon_url: Option<String>,
}

#[tauri::command]
pub async fn list_browser_logs(date: String) -> Result<Vec<BrowserLog>, String> {
    debug!("list_browsers: date: {}", date);
    let browser_logs = sqlx::query_as::<_,
      (i64, i64, String, String, i64,
       i64, i64, Option<String>, Option<String>, Option<i64>, Option<i64>, Option<i64>)>(
        r#"
        SELECT
          e.id, e.timestamp, e.date, e.kind, e.log_id,
          b.id, b.info_id, b.title, b.referrer, b.tab_id, b.opener_tab_id, b.window_id
        FROM event_log e
        INNER JOIN browser_log b ON e.log_id = b.id
        WHERE e.kind = ? AND e.date = ?
        ORDER BY e.timestamp
        "#
    )
    .bind(KIND)
    .bind(date)
    .fetch_all(db::pool().unwrap())
    .await
    .unwrap_or(Vec::new())
    .iter()
    .map(|row| {
        let (event_id, timestamp, date, _kind, _log_id,
             id, info_id, title, referrer, tab_id, opener_tab_id, window_id) = row;
        BrowserLog {
            id: *id,
            event_id: *event_id,
            timestamp: *timestamp,
            date: date.clone(),
            info_id: *info_id,
            title: title.clone(),
            referrer: referrer.clone(),
            tab_id: *tab_id,
            opener_tab_id: *opener_tab_id,
            window_id: *window_id,
        }
    })
    .collect();
    debug!("list_browsers: browser_logs: {:?}", browser_logs);
    Ok(browser_logs)
}

#[tauri::command]
pub async fn get_browser_info(browser_id: i64) -> Result<BrowserInfo, String> {
    debug!("get_browser_info: browser_id={}", browser_id);

    sqlx::query_as::<_, (i64, String, Option<String>)>(
        r#"
        SELECT id, url, fav_icon_url
        FROM browser_info
        WHERE id = ?
        "#
    )
    .bind(browser_id)
    .fetch_one(db::pool().unwrap())
    .await
    .map_or(Err("Not found".to_string()), |row| {
        let (id, url, fav_icon_url) = row;
        Ok(BrowserInfo {
            id: id,
            url: url,
            fav_icon_url: fav_icon_url,
        })
    })
}
