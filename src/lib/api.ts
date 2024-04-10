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

export async function listDates(): Promise<string[]> {
  return await invoke("plugin:immicdb|list_eventlog_dates");
}

export async function listApplicationLogs(timestamp: number, interval: Interval): Promise<[string, ApplicationLog[]][]> {
  return await invoke("plugin:application|list_application_logs", { timestamp, interval });
}

export async function listBrowserLogs(timestamp: number, interval: Interval): Promise<[string, BrowserLog[]][]> {
  return await invoke("plugin:browser|list_browser_logs", { timestamp, interval });
}

export type HitsPerDay = {
  date: string;
  hits: number;
  application_name?: number;
  application_title?: number;
  browser_title?: number;
  browser_url?: number;
  file_path?: number;
}

export type SearchLogsResults = {
  hits: HitsPerDay[];
}

export async function searchLogs(query: string): Promise<SearchLogsResults> {
  return await invoke("plugin:search|search_logs", { query });
}

export async function listFileLogs(timestamp: number, interval: Interval): Promise<[string, FileLog[]][]> {
  return await invoke("plugin:filelog|list_file_logs", { timestamp, interval });
}

export async function listScreenshots(timestamp: number, interval: Interval): Promise<[string, ScreenshotLog[]][]> {
  return await invoke("plugin:screenshot|list_screenshots", { timestamp, interval });
}

export async function listTimeline(timestamp: number, interval: Interval): Promise<[string, [ScreenshotLog[], ApplicationLog[], BrowserLog[], FileLog[]]][]> {
  const applicationLogs = new Map(await listApplicationLogs(timestamp, interval));
  const browserLogs = new Map(await listBrowserLogs(timestamp, interval));
  const fileLogs = new Map(await listFileLogs(timestamp, interval));
  const screenshots = new Map(await listScreenshots(timestamp, interval));

  const hours = Array.from(new Set([...applicationLogs.keys(), ...browserLogs.keys(), ...fileLogs.keys(), ...screenshots.keys()])).sort();
  let timeline: [string, [ScreenshotLog[], ApplicationLog[], BrowserLog[], FileLog[]]][] = hours.map((hour) => {
    let apps = applicationLogs.get(hour) || [];
    let brs = browserLogs.get(hour) || [];
    let fls = fileLogs.get(hour) || [];
    let scr = screenshots.get(hour) || [];
    return [hour, [scr, apps, brs, fls]];
  });
  return timeline;
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
