use anyhow::{Context, Result};
use log::debug;
use serde::Serialize;
use std::collections::HashMap;
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, State, Wry,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::plugins::{
    application::ApplicationPlugin,
    browser::BrowserPlugin,
    db::ImmicDb,
    filelog::FilelogPlugin,
};

#[derive(Debug, Serialize)]
pub struct SearchLogsResult {
    pub hits: Vec<HitsPerDay>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct HitsPerDay {
    pub date: String,
    pub count: i64,
    pub application_name_count: Option<i64>,
    pub application_name_hits: Option<Vec<SearchHit>>,
    pub application_title_count: Option<i64>,
    pub application_title_hits: Option<Vec<SearchHit>>,
    pub browser_title_count: Option<i64>,
    pub browser_title_hits: Option<Vec<SearchHit>>,
    pub browser_url_count: Option<i64>,
    pub browser_url_hits: Option<Vec<SearchHit>>,
    pub file_path_count: Option<i64>,
    pub file_path_hits: Option<Vec<SearchHit>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchHit {
    pub timestamp: i64,
    pub id: i64,
    pub text: Option<String>,
}

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
    pub async fn search_logs(&self, query: &str) -> Result<SearchLogsResult> {
        let db = self.app.try_state::<ImmicDb>().context("Failed to get db plugin")?;
        let pool = db.pool().await.context("Failed to get db pool")?;

        let mut hits: HashMap<String, HitsPerDay> = HashMap::new();
        let query = format!("%{}%", tokenize_query(query).join("%"));
        debug!("search_logs: query={}", query);

        let application = self.app.try_state::<ApplicationPlugin>().context("Failed to get application plugin")?;

        let application_title_hits = application.search_for_title(&pool, &query).await?;
        for (k, v) in application_title_hits {
            if hits.contains_key(&k) {
                let h = hits.get_mut(&k).unwrap();
                h.count += v.count;
                h.application_title_count = v.application_title_count;
                h.application_title_hits = v.application_title_hits;
            } else {
                hits.insert(k, v);
            }
        }

        let application_name_hits = application.search_for_name(&pool, &query).await?;
        for (k, v) in application_name_hits {
            if hits.contains_key(&k) {
                let h = hits.get_mut(&k).unwrap();
                h.count += v.count;
                h.application_name_count = v.application_name_count;
                h.application_name_hits = v.application_name_hits;
            } else {
                hits.insert(k, v);
            }
        }

        let browser = self.app.try_state::<BrowserPlugin>().context("Failed to get browser plugin")?;

        let browser_title_hits = browser.search_for_title(&pool, &query).await?;
        for (k, v) in browser_title_hits {
            if hits.contains_key(&k) {
                let h = hits.get_mut(&k).unwrap();
                h.count += v.count;
                h.browser_title_count = v.browser_title_count;
                h.browser_title_hits = v.browser_title_hits;
            } else {
                hits.insert(k, v);
            }
        }

        let browser_url_hits = browser.search_for_url(&pool, &query).await?;
        for (k, v) in browser_url_hits {
            if hits.contains_key(&k) {
                let h = hits.get_mut(&k).unwrap();
                h.count += v.count;
                h.browser_url_count = v.browser_url_count;
                h.browser_url_hits = v.browser_url_hits;
            } else {
                hits.insert(k, v);
            }
        }

        let filelog = self.app.try_state::<FilelogPlugin>().context("Failed to get filelog plugin")?;

        let file_path_hits = filelog.search_for_path(&pool, &query).await?;
        for (k, v) in file_path_hits {
            if hits.contains_key(&k) {
                let h = hits.get_mut(&k).unwrap();
                h.count += v.count;
                h.file_path_count = v.file_path_count;
                h.file_path_hits = v.file_path_hits;
            } else {
                hits.insert(k, v);
            }
        }

        let mut hits: Vec<HitsPerDay> = hits
            .into_iter()
            .map(|(_date, h)| h)
            .collect();
        hits.sort_by(|a, b| a.date.cmp(&b.date).reverse());

        Ok(SearchLogsResult { hits })
    }
}

#[tauri::command]
pub async fn search_logs(browser: State<'_, SearchPlugin>, query: String) -> Result<SearchLogsResult, String> {
    browser.search_logs(&query).await.map_err(|e| e.to_string())
}

pub fn tokenize_query(query: &str) -> Vec<&str> {
    query.unicode_words().collect::<Vec<&str>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_query() {
        let query = "Hello, world!";
        let expected = vec!["Hello", "world"];
        assert_eq!(tokenize_query(query), expected);

        let query = "こんにちは世界！";
        let expected = vec!["こ", "ん", "に", "ち", "は", "世", "界"];
        assert_eq!(tokenize_query(query), expected);
    }
}
