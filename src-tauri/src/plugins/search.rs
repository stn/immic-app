use std::collections::HashMap;

use anyhow::{Context, Result};
use log::debug;
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, State, Wry,
};

use crate::plugins::{
    application,
    browser,
    db::ImmicDb,
    filelog,
};

pub fn init() -> TauriPlugin<Wry> {
    plugin::Builder::new("search")
        .invoke_handler(tauri::generate_handler![
            search_logs,
        ])
        .setup(|app_handle| {
            debug!("search plugin setup");
            let search = SearchPlugin::new(app_handle);
            app_handle.manage(search);
            Ok(())
        })
        .build()
}

#[derive(Clone)]
pub struct SearchPlugin {
    app: AppHandle,
}

impl SearchPlugin {
    fn new(app: &AppHandle) -> Self {
        Self {
            app: app.clone(),
        }
    }

    // Returns the number of items that hit the query per day
    pub async fn search_logs(&self, query: String) -> Result<SearchLogsResult> {
        debug!("search_logs: query={}", query);

        let db = self.app.try_state::<ImmicDb>().context("Failed to get db plugin")?;
        let pool = db.pool().await.context("Failed to get db pool")?;

        let mut hits: HashMap<String, HitsPerDay> = HashMap::new();

        // application title
        let application_title_hits: Vec<(String, i64)> = sqlx::query_as::<_, (
            String, i64,
        )>(format!(
            r#"
            SELECT
            e.date, COUNT(e.id) AS count
            FROM event_log e
            INNER JOIN application_log a ON e.log_id = a.id
            WHERE e.kind = '{0}' AND a.title LIKE '%{1}%'
            GROUP BY e.date
            ORDER BY e.date
            "#,
            application::KIND,
            query).as_str()
        )
        .fetch_all(&pool)
        .await
        .unwrap_or(Vec::new());

        for (date, count) in application_title_hits {
            hits
                .entry(date.clone())
                .and_modify(|h| {
                    h.hits += count;
                    h.application_title = Some(count);
                })
                .or_insert_with(|| {
                    let mut h = HitsPerDay::default();
                    h.date = date;
                    h.hits = count;
                    h.application_title = Some(count);
                    h
                });
        }

        // application path
        let application_path_hits: Vec<(String, i64)> = sqlx::query_as::<_, (
            String, i64,
        )>(format!(
            r#"
            SELECT
            e.date, COUNT(e.id) AS count
            FROM event_log e
            INNER JOIN application_log a ON e.log_id = a.id
            INNER JOIN application_info i ON a.info_id = i.id
            WHERE e.kind = '{0}' AND i.path LIKE '%{1}%'
            GROUP BY e.date
            ORDER BY e.date
            "#,
            application::KIND,
            query).as_str()
        )
        .fetch_all(&pool)
        .await
        .unwrap_or(Vec::new());

        for (date, count) in application_path_hits {
            hits
                .entry(date.clone())
                .and_modify(|h| {
                    h.hits += count;
                    h.application_path = Some(count);
                })
                .or_insert_with(|| {
                    let mut h = HitsPerDay::default();
                    h.date = date;
                    h.hits = count;
                    h.application_path = Some(count);
                    h
                });
        }

        // browser title
        let browser_title_hits: Vec<(String, i64)> = sqlx::query_as::<_, (
            String, i64,
        )>(format!(
            r#"
            SELECT
            e.date, COUNT(e.id) AS count
            FROM event_log e
            INNER JOIN browser_log b ON e.log_id = b.id
            WHERE e.kind = '{0}' AND b.title LIKE '%{1}%'
            GROUP BY e.date
            ORDER BY e.date
            "#,
            browser::KIND,
            query).as_str()
        )
        .fetch_all(&pool)
        .await
        .unwrap_or(Vec::new());

        for (date, count) in browser_title_hits {
            hits
                .entry(date.clone())
                .and_modify(|h| {
                    h.hits += count;
                    h.browser_title = Some(count);
                })
                .or_insert_with(|| {
                    let mut h = HitsPerDay::default();
                    h.date = date;
                    h.hits = count;
                    h.browser_title = Some(count);
                    h
                });
        }

        // browser url
        let browser_url_hits: Vec<(String, i64)> = sqlx::query_as::<_, (
            String, i64,
        )>(format!(
            r#"
            SELECT
            e.date, COUNT(e.id) AS count
            FROM event_log e
            INNER JOIN browser_log b ON e.log_id = b.id
            INNER JOIN browser_info i ON b.info_id = i.id
            WHERE e.kind = '{0}' AND i.url LIKE '%{1}%'
            GROUP BY e.date
            ORDER BY e.date
            "#,
            browser::KIND,
            query).as_str()
        )
        .fetch_all(&pool)
        .await
        .unwrap_or(Vec::new());

        for (date, count) in browser_url_hits {
            hits
                .entry(date.clone())
                .and_modify(|h| {
                    h.hits += count;
                    h.browser_url = Some(count);
                })
                .or_insert_with(|| {
                    let mut h = HitsPerDay::default();
                    h.date = date;
                    h.hits = count;
                    h.browser_url = Some(count);
                    h
                });
        }

        // file path
        let file_path_hits: Vec<(String, i64)> = sqlx::query_as::<_, (
            String, i64,
        )>(format!(
            r#"
            SELECT
            e.date, COUNT(e.id) AS count
            FROM event_log e
            INNER JOIN file_log f ON e.log_id = f.id
            INNER JOIN file_info i ON f.info_id = i.id
            WHERE e.kind = '{0}' AND i.path LIKE '%{1}%'
            GROUP BY e.date
            ORDER BY e.date
            "#,
            filelog::KIND,
            query).as_str()
        )
        .fetch_all(&pool)
        .await
        .unwrap_or(Vec::new());

        for (date, count) in file_path_hits {
            hits
                .entry(date.clone())
                .and_modify(|h| {
                    h.hits += count;
                    h.file_path = Some(count);
                })
                .or_insert_with(|| {
                    let mut h = HitsPerDay::default();
                    h.date = date;
                    h.hits = count;
                    h.file_path = Some(count);
                    h
                });
        }

        let mut hits: Vec<HitsPerDay> = hits
            .into_iter()
            .map(|(_date, h)| h)
            .collect();
        hits.sort_by(|a, b| a.date.cmp(&b.date).reverse());
        Ok(SearchLogsResult { hits })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SearchLogsResult {
    pub hits: Vec<HitsPerDay>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct HitsPerDay {
    pub date: String,
    pub hits: i64,
    pub application_path: Option<i64>,
    pub application_title: Option<i64>,
    pub browser_title: Option<i64>,
    pub browser_url: Option<i64>,
    pub file_path: Option<i64>,
}

#[tauri::command]
pub async fn search_logs(browser: State<'_, SearchPlugin>, query: String) -> Result<SearchLogsResult, String> {
    browser.search_logs(query).await.map_err(|e| e.to_string())
}
