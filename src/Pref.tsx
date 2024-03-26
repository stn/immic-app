import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

async function settingSet(key: string, value: any) {
  return await invoke("setting_set", { key, value });
}

async function settingGet<T>(key: string) {
  return await invoke("setting_get", { key }) as T;
}

async function settingLoad() {
  return await invoke("setting_load");
}

async function settingSave() {
  return await invoke("setting_save");
}

function Pref() {
  const [dataDir, setDataDir] = useState("");

  async function storePreferences() {
    await settingSet("data-dir", dataDir);
    await settingSave();
  }

  useEffect(() => {
    let isMounted = true;
    (async () => {
      await settingLoad();
      const dataDir = await settingGet<string>("data-dir") || "";
      if (isMounted) {
        setDataDir(dataDir);
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
        <input
          id="data-dir-input"
          onChange={(e) => setDataDir(e.currentTarget.value)}
          placeholder="Data Directory"
          value={dataDir}
        />
        <br />
        <br />
        <button type="submit">Save</button>
      </form>
    </div>
  );
}

export default Pref;
