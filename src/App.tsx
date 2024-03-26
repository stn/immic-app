import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import * as GS from "@tauri-apps/api/globalShortcut";

import "./App.css";
import { ApplicationLog, BrowserLog, FileLog } from "./events";

async function quitApp() {
  await invoke("quit_app");
}

async function showMain() {
  await invoke("show_main");
}

async function showPreferences() {
  await invoke("show_preferences");
}

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [dates, setDates] = useState<string[]>([]);
  const [date, setDate] = useState<string>('');
  const [applications, setApplications] = useState<ApplicationLog[]>([]);
  const [browsers, setBrowsers] = useState<BrowserLog[]>([]);
  const [filelogs, setFilelogs] = useState<FileLog[]>([]);
  const [screens, setScreens] = useState<string[]>([]);

  async function listDates() {
    setDates(await invoke("list_eventlog_dates"));
  }

  async function listApplicationLogs(date: string) {
    setApplications(await invoke("list_application_logs", { date }));
  }

  async function listBrowserLogs(date: string) {
    setBrowsers(await invoke("list_browser_logs", { date }));
  }

  async function listFileLogs(date: string) {
    setFilelogs(await invoke("list_file_logs", { date }));
  }

  async function listScreens(date: string) {
    setScreens(await invoke("list_screens", { date }));
  }

  async function listScreenDates(date: string) {
    setScreens(await invoke("list_screen_dates", { date }));
  }

  useEffect(() => {
    const registerShortCuts = async () => {
      await GS.register("Alt+Shift+K", () => {
        console.log("Alt+Shift+K pressed");
        showMain();
      });
    };
    registerShortCuts();
    return () => {
      GS.unregister("Alt+Shift+K");
    }
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
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>

      <p>{greetMsg}</p>
      <div>
        {dates.map((date) => (
          <button key={date} onClick={() => {
            setDate(date);
            listApplicationLogs(date);
            listBrowserLogs(date);
            listFileLogs(date);
            listScreens(date);
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
          <img key={screen} src={'https://iss.localhost/' + date + '/' + screen + '-t'} alt={screen} />
        ))}
      </div>
    </div>
  );
}

export default App;
