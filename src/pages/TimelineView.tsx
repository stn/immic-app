import { useEffect, useState } from "react";

import { ApplicationLog, BrowserLog, FileLog } from "../lib/events";
import {
  listApplicationLogs,
  listBrowserLogs,
  listFileLogs,
  listScreenshots,
} from "../lib/api";
import type { Interval } from "../lib/api";

export interface TimelineViewProps {
  timestamp?: number;
}

export function TimelineView(props: TimelineViewProps) {
  const [timestamp, setTimestamp] = useState<number>(props.timestamp || Date.now());
  const [interval, setInterval] = useState<Interval>("Hourly");
  const [date, setDate] = useState<string>("");
  const [timeline, setTimeline] = useState<[string, [string, ApplicationLog[], BrowserLog[], FileLog[]]][]>();

  useEffect(() => {
    if (date === "") {
      let d = new Date(timestamp).toISOString().split('T')[0];
      d = d.replace(/-/g, '');
      setDate(d);
    }
  }, []);

  useEffect(() => {
    let isMounted = true;
    
    (async () => {
      let applicationLogs = new Map(await listApplicationLogs(timestamp, interval));
      let browserLogs = new Map(await listBrowserLogs(timestamp, interval));
      let fileLogs = new Map(await listFileLogs(timestamp, interval));
      let screenshots = new Map(await listScreenshots(timestamp, interval));
      if (isMounted) {
        let hours = Array.from(new Set([...applicationLogs.keys(), ...browserLogs.keys(), ...fileLogs.keys(), ...screenshots.keys()])).sort();
        setTimeline(hours.map((hour) => {
          let apps = applicationLogs.get(hour) || [];
          let brs = browserLogs.get(hour) || [];
          let fls = fileLogs.get(hour) || [];
          let scr = screenshots.get(hour) || "";
          return [hour, [scr, apps, brs, fls]];
        }));
      }
    })();

    return () => {
      isMounted = false;
    };
  }, [date]);

  return (
    <div className="m-4">
      <h1 className="text-5xl font-semibold my-6">
        {date}
      </h1>
      <div>
        { timeline && timeline.map(([hour, [screen, applications, browsers, filelogs]]) => (
          <div>
            <div key={hour}>
              <h2 className="text-4xl font-semibold my-4">
                {hour}:00
              </h2>
            </div>
            <div className="grid grid-cols-4 gap-4">
              <div className="mr-4">
                <img key={hour}
                  src={'https://iss.localhost/' + screen + '-t'}
                  alt={`screenshot ${hour}`}
                  />
              </div>
              <div className="w-96">
                {applications.map((app) => (
                  <div key={app.id}>
                    {app.title}
                    {/* {app.id}: {JSON.stringify(app)} */}
                  </div>
                ))}
              </div>
              <div className="w-96">
                {browsers.map((browser) => (
                  <div key={browser.id}>
                    {browser.title}
                    {/* {browser.id}: {JSON.stringify(browser)} */}
                  </div>
                ))}
              </div>
              <div className="w-96">
                {filelogs.map((filelog) => (
                  <div key={filelog.id}>
                    {filelog.id}: {JSON.stringify(filelog)}
                  </div>
                ))}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
