import { invoke } from "@tauri-apps/api/tauri";

export async function settingSet(key: string, value: any) {
  return await invoke("plugin:setting|set", { key, value });
}

export async function settingGet<T>(key: string) {
  return await invoke("plugin:setting|get", { key }) as T;
}

export async function settingLoad() {
  return await invoke("plugin:setting|load");
}

export async function settingSave() {
  return await invoke("plugin:setting|save");
}

export async function quitApp() {
  return await invoke("quit_app");
}

export async function showMain() {
  await invoke("show_main_cmd");
}
