import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import * as GS from "@tauri-apps/api/globalShortcut";

import { Button } from "@/components/ui/button";

import { ApplicationLog, BrowserLog, FileLog } from "./events";
import { settingGet, settingLoad, showMain } from "./lib";

import "./App.css";

function App() {
  const [dates, setDates] = useState<string[]>([]);
  // const [date, setDate] = useState<string>('');
  const [applications, setApplications] = useState<ApplicationLog[]>([]);
  const [browsers, setBrowsers] = useState<BrowserLog[]>([]);
  const [filelogs, setFilelogs] = useState<FileLog[]>([]);
  const [screens, setScreens] = useState<string[]>([]);
  const [globalShortcut, setGlobalShortcut] = useState<string>();
  const [shortcutRegistered, setShortcutRegistered] = useState<boolean>(false);

  async function listDates() {
    setDates(await invoke("plugin:immicdb|list_eventlog_dates"));
  }

  async function listApplicationLogs(date: string) {
    setApplications(await invoke("plugin:application|list_application_logs", { date }));
  }

  async function listBrowserLogs(date: string) {
    setBrowsers(await invoke("plugin:browser|list_browser_logs", { date }));
  }

  async function listFileLogs(date: string) {
    setFilelogs(await invoke("plugin:filelog|list_file_logs", { date }));
  }

  async function listScreenshots(date: string) {
    setScreens(await invoke("plugin:screenshot|list_screenshots", { date }));
  }

  useEffect(() => {
    if (shortcutRegistered) {
      return;
    }

    let isMounted = true;
    const loadGlobalShortcut = async () => {
      await settingLoad();
      const shortcut = await settingGet<string>("global-shortcut") || "Alt+Shift+K";
      if (isMounted) {
        await GS.register(shortcut, () => {
          showMain();
        });
        console.log('registered shortcut: ', shortcut)
        setGlobalShortcut(shortcut);
        setShortcutRegistered(true);
      }
    }
    loadGlobalShortcut();

    return () => {
      isMounted = false;
      globalShortcut && GS.unregister(globalShortcut);
    };
  }, []);

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>

      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          listDates();
        }}
      >
        <Button type="submit">Greet</Button>
      </form>

      <div>
        {dates.map((date) => (
          <Button key={date} onClick={() => {
            // setDate(date);
            listApplicationLogs(date);
            listBrowserLogs(date);
            listFileLogs(date);
            listScreenshots(date);
          }}>
            {date}
          </Button>
        ))}
      </div>
      <div>
        {applications.map((app) => (
          <div key={app.id}>{app.id}: {JSON.stringify(app)}</div>
        ))}
      </div>
      <div>
        {browsers.map((browser) => (
          <div key={browser.id}>{browser.id}: {JSON.stringify(browser)}</div>
        ))}
      </div>
      <div>
        {filelogs.map((filelog) => (
          <div key={filelog.id}>{filelog.id}: {JSON.stringify(filelog)}</div>
        ))}
      </div>
      <div>
        {screens.map((screen) => (
          // <div key={screen}>{screen}</div>
          <img key={screen} src={'https://iss.localhost/' + screen + '-t'} alt={screen} />
        ))}
      </div>
    </div>
  );
}

export default App;
