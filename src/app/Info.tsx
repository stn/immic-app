import { useEffect, useState } from "react";
import { useKey } from "react-use";
import { appWindow } from '@tauri-apps/api/window';

import { useTauriEvent } from "@/lib/immic-events";
import { ThemeProvider } from "@/components/theme-provider";

function Info() {
  useKey("Escape", () => { appWindow.hide(); })

  const event = useTauriEvent();
  const [log, setLog] = useState<string>("");

  useEffect(() => {
    if (event) {
      console.log('tauri-event:', event);
      setLog(JSON.stringify(event, null, 2));
    }
  }, [event]);

  return (
    <ThemeProvider defaultTheme="dark" storageKey="ui-theme">
      <div>
        <h1>Info</h1>
        <div>{log}</div>
      </div>
    </ThemeProvider>
  );
}

export default Info;
