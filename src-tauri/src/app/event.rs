use anyhow::{Context, Result};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::plugins::{
    application::ApplicationLog,
    browser::BrowserLog,
    filelog::FileLog,
    search::{
        SearchHit,
        HitsPerDay,
    },
};

// use super::window::show_info;

const EVENT_LABEL: &str = "immic-event";

#[derive(Clone, Debug, Serialize)]
pub enum ImmicEvent {
    Application(ApplicationLog, Vec<HitsPerDay>),
    Browser(BrowserLog, Vec<HitsPerDay>),
    File(FileLog, Vec<SearchHit>),
}

pub fn emit_event(app: &AppHandle, event: ImmicEvent) -> Result<()> {
    app.emit_all(EVENT_LABEL, event).context("Failed to emit event")
}

pub fn emit_event_to(app: &AppHandle, event: ImmicEvent, label: &str) -> Result<()> {
    app.emit_to(label, EVENT_LABEL, event).context("Failed to emit event")
}

pub fn emit_event_to_info(app: &AppHandle, event: ImmicEvent) -> Result<()> {
    emit_event_to(app, event, "info")?;
    // show_info(app)?;
    Ok(())
}
