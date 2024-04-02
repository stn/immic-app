import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import * as GS from "@tauri-apps/api/globalShortcut";

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

      <div className="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>

      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          listDates();
        }}
      >
        <button type="submit">Greet</button>
      </form>

      <div>
        {dates.map((date) => (
          <button key={date} onClick={() => {
            // setDate(date);
            listApplicationLogs(date);
            listBrowserLogs(date);
            listFileLogs(date);
            listScreenshots(date);
          }}>
            {date}
          </button>
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
