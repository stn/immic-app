use std::collections::HashMap;

use anyhow::{Context, Result};
use log::debug;
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, State, Wry,
};

use crate::plugins::{
    browser,
    db::ImmicDb,
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

        let browser_hits: Vec<(String, i64)> = sqlx::query_as::<_, (
            String, i64,
        )>(format!(
            r#"
            SELECT
            e.date, COUNT(e.id) AS count
            FROM event_log e
            INNER JOIN browser_log b ON e.log_id = b.id
            INNER JOIN browser_info i ON b.info_id = i.id
            WHERE e.kind = '{0}' AND (i.url LIKE '%{1}%' OR b.title LIKE '%{1}%')
            GROUP BY e.date
            ORDER BY e.date
            "#,
            browser::KIND,
            query).as_str()
        )
        .fetch_all(&pool)
        .await
        .unwrap_or(Vec::new());

        let mut hits: HashMap<String, HitsPerDay> = HashMap::new();
        for (date, count) in browser_hits {
            hits
                .entry(date.clone())
                .and_modify(|h| {
                    h.count += count;
                    h.browser = count;
                })
                .or_insert(
                    HitsPerDay {
                        date,
                        count: count,
                        browser: count,
                    }
                );
        }
        
        Ok(SearchLogsResult {
            hits: hits
                .into_iter()
                .map(|(_date, h)| h)
                .collect(),
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SearchLogsResult {
    pub hits: Vec<HitsPerDay>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct HitsPerDay {
    pub date: String,
    pub count: i64,
    pub browser: i64,
}

#[tauri::command]
pub async fn search_logs(browser: State<'_, SearchPlugin>, query: String) -> Result<SearchLogsResult, String> {
    browser.search_logs(query).await.map_err(|e| e.to_string())
}
