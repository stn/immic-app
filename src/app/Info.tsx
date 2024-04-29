import { useEffect, useState } from "react";
import { useKey } from "react-use";
import { appWindow } from '@tauri-apps/api/window';

import { Label } from "@/components/ui/label";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion"

import { ThemeProvider } from "@/components/theme-provider";
import { ApplicationLogItem } from "@/components/elements/ApplicationLogItem";
import { BrowserLogItem } from "@/components/elements/BrowserLogItem";
import { FileLogItem } from "@/components/elements/FilelogItem";

import { HitsPerDay, SearchHit } from "@/lib/api";
import { ImmicEvent, useTauriEvent } from "@/lib/immic-events";
import { timestamp_hhmm, timestamp_yyyymmss } from "@/lib/utils";


function Info() {
  useKey("Escape", () => { appWindow.hide(); })

  const event = useTauriEvent();
  const [events, setEvents] = useState<ImmicEvent[]>([]);

  useEffect(() => {
    if (event) {
      // console.log("event", event);
      if (event.Application || event.Browser || event.File) {
        const now = new Date().getTime() / 1000.0;
        let new_events = [event, ...events];
        new_events = new_events.filter((e) => (now - event_timestamp(e)) < 60); // TODO setting
        setEvents(new_events);
      }
    }
  }, [event]);

  return (
    <ThemeProvider defaultTheme="dark" storageKey="ui-theme">
      <div>
        {events.map((e) => (
          <div key={event_id(e)}>
            { e.Application && (
              <div>
                <ApplicationLogItem applicationlog={e.Application[0]} />
                <HitsPerDays hitsPerDays={e.Application[1]} item="application_title_hits" />
              </div>
            )}
            { e.Browser && (
              <div>
                <BrowserLogItem browserlog={e.Browser[0]} />
                <HitsPerDays hitsPerDays={e.Browser[1]} item="browser_url_hits" />
              </div>
            )}
            { e.File && (
              <div>
                <FileLogItem filelog={e.File[0]} />
                <HitsPerDays hitsPerDays={e.File[1]} item="file_path_hits" />
              </div>
            )}
          </div>
        ))}
      </div>
    </ThemeProvider>
  );
}

function HitsPerDays({ hitsPerDays, item }: { hitsPerDays: HitsPerDay[], item: keyof HitsPerDay}) {
  return (
    <Accordion
      type="single"
      collapsible
      className="ml-16"
    >
      { hitsPerDays.map((hitsPerDay) => (
        <AccordionItem value={hitsPerDay.date}>
          <AccordionTrigger className="py-1">
            {timestamp_yyyymmss((hitsPerDay[item] as SearchHit[])[0].timestamp)}
          </AccordionTrigger>
          <AccordionContent className="pl-4 flex flex-wrap">
            {(hitsPerDay[item] as SearchHit[]).map((hit: { timestamp: number; }, i: number) => (
              <Label key={i} className="ml-2 mt-1">
                {timestamp_hhmm(hit.timestamp)}
              </Label>
            ))}
          </AccordionContent>
        </AccordionItem>
      ))}
    </Accordion>
  );
}

function event_timestamp(event: ImmicEvent): number {
  if (event.Application) {
    return event.Application[0].timestamp;
  }
  if (event.Browser) {
    return event.Browser[0].timestamp;
  }
  if (event.File) {
    return event.File[0].timestamp;
  }
  return 0;
}

function event_id(event: ImmicEvent): string {
  if (event.Application) {
    return `a${event.Application[0].event_id}`;
  }
  if (event.Browser) {
    return `b${event.Browser[0].event_id}`;
  }
  if (event.File) {
    return `f${event.File[0].event_id}`;
  }
  return "unknown";
}

export default Info;
