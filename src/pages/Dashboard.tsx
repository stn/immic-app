import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";

import { invoke } from "@tauri-apps/api/tauri";
import * as GS from "@tauri-apps/api/globalShortcut";

import { Button } from "@/components/ui/button";

import { ApplicationLog, BrowserLog, FileLog } from "../lib/events";
import { settingGet, settingLoad, showMain } from "../lib/api";

function Dashboard() {
  const navigate = useNavigate();

  const [dataDir, setDataDir] = useState("");
  const [dates, setDates] = useState<string[]>([]);
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
    let isMounted = true;

    (async () => {
      // Check if data-dir is set
      if (dataDir === "") {
        await settingLoad();
        const dataDir = await settingGet<string>("data-dir") || "";
        if (isMounted) {
          setDataDir(dataDir);
          if (dataDir === "") {
            navigate("/setting");
            return;
          }
        }
      }

      // Register global shortcut
      if (!shortcutRegistered) {
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
    })();

    return () => {
      isMounted = false;
      globalShortcut && GS.unregister(globalShortcut);
    };
  }, []);

  return (
    <div className="container">
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
          <img key={screen} src={'https://iss.localhost/' + screen + '-t'} alt={screen} />
        ))}
      </div>
    </div>
  );
}

export default Dashboard;
