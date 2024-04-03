import { useEffect, useState } from "react";

import { ApplicationLog, BrowserLog, FileLog } from "../lib/events";
import {
  listApplicationLogs,
  listBrowserLogs,
  listFileLogs,
  listScreenshots,
} from "../lib/api";

export interface DailyViewProps {
  date: string;
}

export function DailyView(props: DailyViewProps) {
  const [applications, setApplications] = useState<ApplicationLog[]>([]);
  const [browsers, setBrowsers] = useState<BrowserLog[]>([]);
  const [filelogs, setFilelogs] = useState<FileLog[]>([]);
  const [screens, setScreens] = useState<string[]>([]);

  useEffect(() => {
    let isMounted = true;
    
    (async () => {
      let applicationLogs = await listApplicationLogs(props.date);
      let browserLogs = await listBrowserLogs(props.date);
      let fileLogs = await listFileLogs(props.date);
      let screenshots = await listScreenshots(props.date);
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
  }, []);

  return (
    <div>
      <h1>
        {props.date}
      </h1>
      <div>
        {applications.map((app) => (
          <div key={app.id}>{app.id}: {JSON.stringify(app)}</div>
        ))}
      </div>
      <div>
        {browsers.map((browser) => (
          <div key={browser.id}>{browser.id}: {JSON.stringify(browser)}</div>
        ))}
      </div>
      <div>
        {filelogs.map((filelog) => (
          <div key={filelog.id}>{filelog.id}: {JSON.stringify(filelog)}</div>
        ))}
      </div>
      <div>
        {screens.map((screen) => (
          <img key={screen} src={'https://iss.localhost/' + screen + '-t'} alt={screen} />
        ))}
      </div>
    </div>
  );
}
