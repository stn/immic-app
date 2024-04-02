import { useEffect, useState } from "react";
import * as autostart from "tauri-plugin-autostart-api";

import { settingGet, settingLoad, settingSave, settingSet, quitApp } from "./lib";

function Pref() {
  const [dataDir, setDataDir] = useState("");
  const [serverPort, setServerPort] = useState("");
  const [watchPathset, setWatchPathset] = useState<string>("");
  const [autostartEnabled, setAutostartEnabled] = useState(false);
  const [globalShortcut, setGlobalShortcut] = useState<string>("");

  async function storePreferences() {
    await settingSet("data-dir", dataDir);
    await settingSet("server-port", serverPort);
    await settingSet("watch-pathset", watchPathset);
    await settingSet("global-shortcut", globalShortcut);
    await settingSave();
    await quitApp();
  }

  useEffect(() => {
    let isMounted = true;
    (async () => {
      await settingLoad();
      const dataDir = await settingGet<string>("data-dir") || "";
      const serverPort = await settingGet<string>("server-port") || "53294";
      const watchPathset = await settingGet<string>("watch-pathset") || "";
      const autostartEnabled = await autostart.isEnabled();
      const globalShortcut = await settingGet<string>("global-shortcut") || "Alt+Shift+K";
      if (isMounted) {
        setDataDir(dataDir);
        setServerPort(serverPort);
        setWatchPathset(watchPathset);
        setAutostartEnabled(autostartEnabled);
        setGlobalShortcut(globalShortcut);
      }
    })();
    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <div className="container">
      <h1>Preferences</h1>

      <form
        onSubmit={(e) => {
          e.preventDefault();
          (async () => {
            storePreferences();
          })();
        }}
      >
        <label>Data Directory</label>
        <input
          id="data-dir-input"
          onChange={(e) => setDataDir(e.currentTarget.value)}
          placeholder="Data Directory"
          value={dataDir}
        />
        <br />

        <label>Server Port</label>
        <input
          id="server-port-input"
          onChange={(e) => setServerPort(e.currentTarget.value)}
          placeholder="Server Port"
          value={serverPort}
        />
        <br />

        <label>Watch Pathset</label>
        <input
          id="watch-pathset-input"
          onChange={(e) => setWatchPathset(e.currentTarget.value)}
          placeholder="Watch Pathset"
          value={watchPathset}
        />
        <br />

        <label>Autostart</label>
        <input
          id="autostart-input"
          type="checkbox"
          checked={autostartEnabled}
          onChange={(e) => {
            if (e.currentTarget.checked) {
              autostart.enable();
            } else {
              autostart.disable();
            }
            setAutostartEnabled(e.currentTarget.checked);
          }}
        />
        <br />

        <label>Global Shortcut</label>
        <input
          id="global-shortcut-input"
          onChange={(e) => setGlobalShortcut(e.currentTarget.value)}
          placeholder="Global Shortcut"
          value={globalShortcut}
        />
        <br />

        <button type="submit">Save</button>
      </form>
    </div>
  );
}

export default Pref;
