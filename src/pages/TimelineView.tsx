import { useEffect, useState } from "react";

import { ApplicationLog, BrowserLog, FileLog } from "../lib/events";
import {
  listApplicationLogs,
  listBrowserLogs,
  listFileLogs,
  listScreenshots,
} from "../lib/api";

export interface TimelineViewProps {
  timestamp?: number;
}

export function TimelineView(props: TimelineViewProps) {
  const [timestamp, setTimestamp] = useState<number>(props.timestamp || Date.now());
  const [date, setDate] = useState<string>("");
  const [applications, setApplications] = useState<[string, ApplicationLog[]][]>();
  const [browsers, setBrowsers] = useState<[string, BrowserLog[]][]>();
  const [filelogs, setFilelogs] = useState<[string, FileLog[]][]>();
  const [screens, setScreens] = useState<[string, string][]>();

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
      let applicationLogs = await listApplicationLogs(timestamp, "Hourly");
      let browserLogs = await listBrowserLogs(timestamp, "Hourly");
      let fileLogs = await listFileLogs(timestamp, "Hourly");
      let screenshots = await listScreenshots(timestamp, "Hourly");
      if (isMounted) {
        setApplications(applicationLogs);
        setBrowsers(browserLogs);
        setFilelogs(fileLogs);
        setScreens(screenshots);
      }
    })();

    return () => {
      isMounted = false;
    };
  }, [date]);

  return (
    <div className="mx-auto grid w-full items-start gap-6">
      <h1 className="text-5xl font-semibold">
        {date}
      </h1>
      <div>
        {applications && applications.map(([hour, apps]) => (
          <div key={hour}>
            <h2>{hour}</h2>
            <div>
              {apps.map((app) => (
                <div key={app.id}>{app.id}: {JSON.stringify(app)}</div>
              ))}
            </div>
          </div>
        ))}
      </div>
      <div>
        {browsers && browsers.map(([hour, bs]) => (
          <div key={hour}>
            <h2>{hour}</h2>
            <div>
              {bs.map((browser) => (
                <div key={browser.id}>{browser.id}: {JSON.stringify(browser)}</div>
              ))}
            </div>
          </div>
        ))}
      </div>
      <div>
        {filelogs && filelogs.map(([hour, logs]) => (
          <div key={hour}>
            <h2>{hour}</h2>
            <div>
              {logs.map((filelog) => (
                <div key={filelog.id}>{filelog.id}: {JSON.stringify(filelog)}</div>
              ))}
            </div>
          </div>
        ))}
      </div>
      <div>
        {screens && screens.map(([hour, screen]) => (
          <div key={hour}>
            <h2>{hour}</h2>
            <img key={hour} src={'https://iss.localhost/' + screen + '-t'} alt={screen} />
          </div>
        ))}
      </div>
    </div>
  );
}
