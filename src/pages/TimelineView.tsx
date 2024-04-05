import { useEffect, useState } from "react";

import { ApplicationLog, BrowserLog, FileLog } from "../lib/events";
import { listTimeline } from "../lib/api";
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
      let logs = await listTimeline(timestamp, interval);
      if (isMounted) {
        setTimeline(logs);
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
          <div key={hour}>
            <h2 className="text-4xl font-semibold my-4">
              {hour}:00
            </h2>
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
                    <div>{app.name}</div>
                    <div className="pl-4">{app.title}</div>
                    {/* {JSON.stringify(app)} */}
                  </div>
                ))}
              </div>
              <div className="w-96">
                <ul className="list-disc">
                  {browsers.map((browser) => (
                    <li key={browser.id}>
                      {/* <img src={browser.fav_icon_url} alt="favicon" /> */}
                      <a href={browser.url} target="_blank" rel="noopener noreferrer"
                        className="decoration-1 underline-offset-2 hover:underline"
                      >
                        {browser.title}
                      </a>
                      {/* {JSON.stringify(browser)} */}
                    </li>
                  ))}
                </ul>
              </div>
              <div className="w-96">
                {filelogs.map((filelog) => (
                  <div key={filelog.id}>
                    {filelog.kind === "create" ? "C" :
                     filelog.kind === "modify" ? "M" : 
                     filelog.kind === "remove" ? "R" :
                     "?"} &nbsp;
                     {filelog.path}
                    {/* {JSON.stringify(filelog)} */}
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
