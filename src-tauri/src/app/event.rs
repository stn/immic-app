use anyhow::{Context, Result};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::{
    app::window::show_main,
    plugins::{
        application::ApplicationLog,
        browser::BrowserLog,
        filelog::FileLog,
        search::HitsPerDay,
    },
};

// use super::window::show_info;

const EVENT_LABEL: &str = "immic-event";

#[derive(Clone, Debug, Serialize)]
pub enum ImmicEvent {
    Application(ApplicationLog, Vec<HitsPerDay>),
    Browser(BrowserLog, Vec<HitsPerDay>),
    File(FileLog, Vec<HitsPerDay>),
    OpenHourly(i64), // timestamp
}

pub fn emit_event(app: &AppHandle, event: ImmicEvent) -> Result<()> {
    app.emit_all(EVENT_LABEL, event).context("Failed to emit event")
}

pub fn emit_event_to(app: &AppHandle, event: ImmicEvent, label: &str) -> Result<()> {
    app.emit_to(label, EVENT_LABEL, event).context("Failed to emit event")
}

pub fn emit_event_to_main(app: &AppHandle, event: ImmicEvent) -> Result<()> {
    emit_event_to(app, event, "main")?;
    Ok(())
}

pub fn emit_event_to_info(app: &AppHandle, event: ImmicEvent) -> Result<()> {
    emit_event_to(app, event, "info")?;
    Ok(())
}

#[tauri::command]
pub fn open_hourly_cmd(app: AppHandle, timestamp: i64) -> Result<(), String> {
    show_main(&app).map_err(|e| e.to_string())?;
    emit_event_to_main(&app, ImmicEvent::OpenHourly(timestamp)).map_err(|e| e.to_string())
}
