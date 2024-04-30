import { invoke } from "@tauri-apps/api/tauri";
import {
  ApplicationLog,
  BrowserLog,
  FileLog,
  ScreenshotLog,
} from "./events";

export type Interval = "Hourly" | "Daily" | "Weekly" | "Monthly" | "Yearly";

export async function settingSet(key: string, value: any): Promise<void> {
  return await invoke("plugin:setting|set", { key, value });
}

export async function settingGet<T>(key: string): Promise<T> {
  return await invoke("plugin:setting|get", { key }) as T;
}

export async function settingLoad(): Promise<void> {
  return await invoke("plugin:setting|load");
}

export async function settingSave(): Promise<void> {
  return await invoke("plugin:setting|save");
}

export async function quitApp(): Promise<void> {
  await invoke("quit_app");
}

export async function showMain(): Promise<void> {
  await invoke("show_main_cmd");
}

export async function openHourly(timestamp: number): Promise<void> {
  await invoke("open_hourly_cmd", { timestamp });
}

export async function listDates(): Promise<string[]> {
  return await invoke("plugin:immicdb|list_eventlog_dates");
}

export async function exportLogs(filename: string): Promise<void> {
  return await invoke("plugin:immicdb|export_logs", { filename });
}

export async function importLogs(filename: string): Promise<void> {
  return await invoke("plugin:immicdb|import_logs", { filename });
}

export type SearchLogsResults = {
  hits: HitsPerDay[];
}

export type HitsPerDay = {
  date: string;
  count: number;
  application_name_count?: number;
  application_name_hits?: SearchHit[];
  application_title_count?: number;
  application_title_hits?: SearchHit[];
  browser_title_count?: number;
  browser_title_hits?: SearchHit[];
  browser_url_count?: number;
  browser_url_hits?: SearchHit[];
  file_path_count?: number;
  file_path_hits?: SearchHit[];
}

export type SearchHit = {
  id: number;
  timestamp: number;
}

export async function searchLogs(query: string): Promise<SearchLogsResults> {
  return await invoke("plugin:search|search_logs", { query });
}

export async function listTimelineOn(date: string): Promise<[string, [ScreenshotLog[], ApplicationLog[], BrowserLog[], FileLog[]]][]> {
  let [application_logs, browser_logs, file_logs, screenshot_logs] = await listAnyLogsOn(date);
  let application_logs_map = partitionLogHourly(application_logs);
  let browser_logs_map = partitionLogHourly(browser_logs);
  let file_logs_map = partitionLogHourly(file_logs);
  let screenshot_logs_map = partitionLogHourly(screenshot_logs);
  const hours = Array.from(new Set([...application_logs_map.keys(), ...browser_logs_map.keys(), ...file_logs_map.keys(), ...screenshot_logs_map.keys()])).sort();
  let timeline: [string, [ScreenshotLog[], ApplicationLog[], BrowserLog[], FileLog[]]][] = hours.map((hour) => {
    let apps = application_logs_map.get(hour) || [];
    let brs = browser_logs_map.get(hour) || [];
    let fls = file_logs_map.get(hour) || [];
    let scr = screenshot_logs_map.get(hour) || [];
    return [hour, [scr, apps, brs, fls]]; // scr is the first
  });
  return timeline;
}

async function listAnyLogsOn(date: string): Promise<[ApplicationLog[], BrowserLog[], FileLog[], ScreenshotLog[]]> {
  type AnyLog = { ApplicationLogEntry: ApplicationLog } | { BrowserLogEntry: BrowserLog } | { FileLogEntry: FileLog } | { ScreenshotLogEntry: ScreenshotLog }

  const any_logs = await invoke<AnyLog[]>("plugin:immicdb|list_any_logs_on", { date });
  let application_logs: ApplicationLog[] = [];
  let browser_logs: BrowserLog[] = [];
  let file_logs: FileLog[] = [];
  let screenshot_logs: ScreenshotLog[] = [];
  for (let log of any_logs) {
    // check if log is ApplicationLogEntry
    if ("ApplicationLogEntry" in log) {
      application_logs.push(log.ApplicationLogEntry);
    } else if ("BrowserLogEntry" in log) {
      browser_logs.push(log.BrowserLogEntry);
    } else if ("FileLogEntry" in log) {
      file_logs.push(log.FileLogEntry);
    } else if ("ScreenshotLogEntry" in log) {
      screenshot_logs.push(log.ScreenshotLogEntry);
    }
  }
  return [application_logs, browser_logs, file_logs, screenshot_logs];
}

function partitionLogHourly<T extends { timestamp: number }>(logs: T[]): Map<string, T[]> {
  let hourly_logs = new Map();
  for (let log of logs) {
    // convert timestamp to hour in localtime
    let hour = new Date(log.timestamp * 1000).toLocaleString("en-US", { hour: "2-digit", hour12: false });
    if (hour === "24") {
      hour = "00";
    }
    if (hourly_logs.has(hour)) {
      hourly_logs.get(hour).push(log);
    } else {
      hourly_logs.set(hour, [log]);
    }
  }
  return hourly_logs;
}

export function image_url(screenshot: ScreenshotLog): string {
  const ts_image = timestamp2image(screenshot.timestamp);
  return `https://iss.localhost/${ts_image}-${screenshot.monitor_id}`;
}

export function thumb_image_url(screenshot: ScreenshotLog): string {
  const ts_image = timestamp2image(screenshot.timestamp);
  return `https://iss.localhost/${ts_image}-${screenshot.monitor_id}-t`;
}

function timestamp2image(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const year = date.getUTCFullYear().toString().padStart(4, "0");
  const month = (date.getUTCMonth() + 1).toString().padStart(2, "0");
  const day = date.getUTCDate().toString().padStart(2, "0");
  const hour = date.getUTCHours().toString().padStart(2, "0");
  const minute = date.getUTCMinutes().toString().padStart(2, "0");
  const second = date.getUTCSeconds().toString().padStart(2, "0");
  return `${year}${month}${day}/${hour}${minute}${second}`;
}
