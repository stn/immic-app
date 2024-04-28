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
  const [events, setEvents] = useState<ImmicEvent[]>([]);

  useEffect(() => {
    if (event) {
      // console.log("event", event);
      const now = new Date().getTime() / 1000.0;
      let new_events = [event, ...events];
      new_events = new_events.filter((e) => (now - event_timestamp(e)) < 60); // TODO setting
      setEvents(new_events);
    }
  }, [event]);

  return (
    <ThemeProvider defaultTheme="dark" storageKey="ui-theme">
      <div>
        {/* <div>{lastEvent && JSON.stringify(lastEvent, null, 2)}</div> */}
        {events.map((e) => (
          <div key={event_id(e)}>
            { e.Application && (
              <div>
                <ApplicationLogItem applicationlog={e.Application[0]} />
                <div>
                  { e.Application[1].map((h) => (
                    <span key={h.id}>
                      ({h.id}, {h.timestamp})
                    </span>
                  ))}
                </div>
              </div>
            )}
            { e.Browser && <BrowserLogItem browserlog={e.Browser} /> }
            { e.File && <FileLogItem filelog={e.File} /> }
          </div>
        ))}
      </div>
    </ThemeProvider>
  );
}

function event_timestamp(event: ImmicEvent): number {
  if (event.Application) {
    return event.Application[0].timestamp;
  }
  if (event.Browser) {
    return event.Browser.timestamp;
  }
  if (event.File) {
    return event.File.timestamp;
  }
  return 0;
}

function event_id(event: ImmicEvent): string {
  if (event.Application) {
    return `a${event.Application[0].id}`;
  }
  if (event.Browser) {
    return `b${event.Browser.id}`;
  }
  if (event.File) {
    return `f${event.File.id}`;
  }
  return "unknown";
}

export default Info;
