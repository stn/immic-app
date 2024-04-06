import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";

import * as GS from "@tauri-apps/api/globalShortcut";

import {
  settingGet,
  settingLoad,
  showMain
} from "@/lib/api";

import { TimelineView } from "./TimelineView";

function MainPanel() {
  const navigate = useNavigate();

  const [dataDir, setDataDir] = useState("");
  const [globalShortcut, setGlobalShortcut] = useState<string>();
  const [shortcutRegistered, setShortcutRegistered] = useState<boolean>(false);

  useEffect(() => {
    let isMounted = true;

    // ここの処理は一見Appにあるべきにみえるが、global-shortcutの設定はsettingsに依存し、
    // settingsがない場合はsettingsにリダイレクトするため、ここに書いている。

    (async () => {
      // Check if data-dir is set
      if (dataDir === "") {
        await settingLoad();
        const dataDir = await settingGet<string>("data-dir") || "";
        if (isMounted) {
          setDataDir(dataDir);
          if (dataDir === "") {
            navigate("/settings");
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
    <>
      <TimelineView />
    </>
  );
}

export default MainPanel;
