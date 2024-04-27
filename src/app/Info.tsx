import { useEffect, useState } from "react";
import { useKey } from "react-use";
import { appWindow } from '@tauri-apps/api/window';

import { ImmicEvent, useTauriEvent } from "@/lib/immic-events";
import { ThemeProvider } from "@/components/theme-provider";
import { ApplicationLogItem } from "@/components/elements/ApplicationLogItem";
import { BrowserLogItem } from "@/components/elements/BrowserLogItem";
import { FileLogItem } from "@/components/elements/FilelogItem";


function Info() {
  useKey("Escape", () => { appWindow.hide(); })

  const event = useTauriEvent();
  const [lastEvent, setLastEvent] = useState<ImmicEvent | null>(null);

  useEffect(() => {
    if (event) {
      console.log('tauri-event:', event);
      setLastEvent(event);
    }
  }, [event]);

  return (
    <ThemeProvider defaultTheme="dark" storageKey="ui-theme">
      <div>
        <h1>Info</h1>
        {/* <div>{lastEvent && JSON.stringify(lastEvent, null, 2)}</div> */}
        { event?.Application && <ApplicationLogItem applicationlog={event.Application} /> }
        { event?.Browser && <BrowserLogItem browserlog={event.Browser} /> }
        { event?.File && <FileLogItem filelog={event.File} /> }
      </div>
    </ThemeProvider>
  );
}

export default Info;
