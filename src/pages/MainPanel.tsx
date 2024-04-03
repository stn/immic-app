import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";

import * as GS from "@tauri-apps/api/globalShortcut";

import { settingGet, settingLoad, showMain } from "../lib/api";
import { DailyView } from "./DailyView";

function MainPanel() {
  const navigate = useNavigate();

  const [dataDir, setDataDir] = useState("");
  const [date, setDate] = useState<string>();
  const [globalShortcut, setGlobalShortcut] = useState<string>();
  const [shortcutRegistered, setShortcutRegistered] = useState<boolean>(false);

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

      let date = new Date().toISOString().split('T')[0];
      date = date.replace(/-/g, '');
      setDate(date);
    })();

    return () => {
      isMounted = false;
      globalShortcut && GS.unregister(globalShortcut);
    };
  }, []);

  return (
    <div>
      { date && <DailyView date={date} /> }
    </div>
  );
}

export default MainPanel;
