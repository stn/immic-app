use anyhow::{Context, Result};
use futures::TryStreamExt;
use log::debug;
use std::collections::HashMap;
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, State, Wry,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::plugins::{
    application,
    browser,
    db::ImmicDb,
    filelog,
};

#[derive(Debug, serde::Serialize)]
pub struct SearchLogsResult {
    pub hits: Vec<HitsPerDay>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct HitsPerDay {
    pub date: String,
    pub hits: i64,
    pub application_name: Option<i64>,
    pub application_title: Option<i64>,
    pub browser_title: Option<i64>,
    pub browser_url: Option<i64>,
    pub file_path: Option<i64>,
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

        // application title
        let mut rows = sqlx::query_as::<_, (
            String, i64,
        )>(
            r#"
            SELECT
            e.date, COUNT(e.id) AS count
            FROM event_log e
            INNER JOIN application_log a
                ON e.kind = ?
                AND e.id = a.event_id
                AND a.title LIKE ?
            GROUP BY e.date
            ORDER BY e.date
            "#
        )
        .bind(application::KIND)
        .bind(&query)
        .fetch(&pool);

        while let Some((date, count)) = rows.try_next().await? {
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

        // application name
        let mut rows = sqlx::query_as::<_, (
            String, i64,
        )>(
            r#"
            SELECT
            e.date, COUNT(e.id) AS count
            FROM event_log e
            INNER JOIN application_log a
                ON e.kind = ?
                AND e.id = a.event_id
            INNER JOIN application_info i
                ON a.info_id = i.id
                AND i.name LIKE ?
            GROUP BY e.date
            ORDER BY e.date
            "#
        )
        .bind(application::KIND)
        .bind(&query)
        .fetch(&pool);

        while let Some((date, count)) = rows.try_next().await? {
            hits
                .entry(date.clone())
                .and_modify(|h| {
                    h.hits += count;
                    h.application_name = Some(count);
                })
                .or_insert_with(|| {
                    let mut h = HitsPerDay::default();
                    h.date = date;
                    h.hits = count;
                    h.application_name = Some(count);
                    h
                });
        }

        // browser title
        let mut rows = sqlx::query_as::<_, (
            String, i64,
        )>(
            r#"
            SELECT
            e.date, COUNT(e.id) AS count
            FROM event_log e
            INNER JOIN browser_log b
                ON e.kind = ?
                AND e.id = b.event_id
                AND b.title LIKE ?
            GROUP BY e.date
            ORDER BY e.date
            "#
        )
        .bind(browser::KIND)
        .bind(&query)
        .fetch(&pool);

        while let Some((date, count)) = rows.try_next().await? {
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
        let mut rows = sqlx::query_as::<_, (
            String, i64,
        )>(
            r#"
            SELECT
            e.date, COUNT(e.id) AS count
            FROM event_log e
            INNER JOIN browser_log b
                ON e.kind = ?
                AND e.id = b.event_id
            INNER JOIN browser_url u
                ON b.url_id = u.id
                AND u.url LIKE ?
            GROUP BY e.date
            ORDER BY e.date
            "#
        )
        .bind(browser::KIND)
        .bind(&query)
        .fetch(&pool);

        while let Some((date, count)) = rows.try_next().await? {
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
        let mut rows = sqlx::query_as::<_, (
            String, i64,
        )>(
            r#"
            SELECT
            e.date, COUNT(e.id) AS count
            FROM event_log e
            INNER JOIN file_log f
                ON e.kind = ?
                AND e.id = f.event_id
            INNER JOIN file_info i
                ON f.info_id = i.id
                AND i.path LIKE ?
            GROUP BY e.date
            ORDER BY e.date
            "#
        )
        .bind(filelog::KIND)
        .bind(&query)
        .fetch(&pool);

        while let Some((date, count)) = rows.try_next().await? {
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

#[tauri::command]
pub async fn search_logs(browser: State<'_, SearchPlugin>, query: String) -> Result<SearchLogsResult, String> {
    browser.search_logs(&query).await.map_err(|e| e.to_string())
}

fn tokenize_query(query: &str) -> Vec<&str> {
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
