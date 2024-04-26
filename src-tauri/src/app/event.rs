use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::plugins::{
    application::ApplicationLog,
    browser::BrowserLog,
    filelog::FileLog,
};

// use super::window::show_info;

const EVENT_LABEL: &str = "immic-event";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ImmicEvent {
    Application(ApplicationLog),
    Browser(BrowserLog),
    File(FileLog),
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
