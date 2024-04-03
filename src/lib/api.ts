import { invoke } from "@tauri-apps/api/tauri";
import {
  ApplicationLog,
  BrowserLog,
  FileLog,
} from "./events";

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

export async function listApplicationLogs(date: string): Promise<ApplicationLog[]> {
  return await invoke("plugin:application|list_application_logs", { date });
}

export async function listBrowserLogs(date: string): Promise<BrowserLog[]> {
  return await invoke("plugin:browser|list_browser_logs", { date });
}

export async function listFileLogs(date: string): Promise<FileLog[]> {
  return await invoke("plugin:filelog|list_file_logs", { date });
}

export async function listScreenshots(date: string): Promise<string[]> {
  return await invoke("plugin:screenshot|list_screenshots", { date });
}
