import { invoke } from "@tauri-apps/api/tauri";
import {
  ApplicationLog,
  BrowserLog,
  FileLog,
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
  browser?: number;
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

export async function listScreenshots(timestamp: number, interval: Interval): Promise<[string, string][]> {
  return await invoke("plugin:screenshot|list_screenshots", { timestamp, interval });
}

export async function listTimeline(timestamp: number, interval: Interval): Promise<[string, [string, ApplicationLog[], BrowserLog[], FileLog[]]][]> {
  const applicationLogs = new Map(await listApplicationLogs(timestamp, interval));
  const browserLogs = new Map(await listBrowserLogs(timestamp, interval));
  const fileLogs = new Map(await listFileLogs(timestamp, interval));
  const screenshots = new Map(await listScreenshots(timestamp, interval));

  const hours = Array.from(new Set([...applicationLogs.keys(), ...browserLogs.keys(), ...fileLogs.keys(), ...screenshots.keys()])).sort();
  let timeline: [string, [string, ApplicationLog[], BrowserLog[], FileLog[]]][] = hours.map((hour) => {
    let apps = applicationLogs.get(hour) || [];
    let brs = browserLogs.get(hour) || [];
    let fls = fileLogs.get(hour) || [];
    let scr = screenshots.get(hour) || "";
    return [hour, [scr, apps, brs, fls]];
  });
  return timeline;
}
