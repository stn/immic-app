import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";

import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

import {
  listTimeline,
} from "@/lib/api";
import {
  ApplicationLog,
  BrowserLog,
  FileLog
} from "@/lib/events";

function HourlyPage() {
  const params = useParams();

  const [timeline, setTimeline] = useState<[string, [string, ApplicationLog[], BrowserLog[], FileLog[]]][]>();

  useEffect(() => {
    let isMounted = true;

    try {
      const ts = new Date(`${params.year}-${params.month}-${params.day}T00:00:00`).getTime();
      (async () => {
        let logs = await listTimeline(ts, "Hourly");
        if (isMounted) {
          setTimeline(logs);
        }
      })();
    } catch {
      console.error("Invalid date");
    }

    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <div>
      <h1 className="text-5xl font-semibold mb-6">
        <Link to="/">
          {params.year}/{params.month}/{params.day}
        </Link>
      </h1>
      <div>
        { timeline && timeline.map(([hour, [screen, applications, browsers, filelogs]]) => (
          <div key={hour} className="my-4">
            <div className="flex flex-row">
              <h2 className="text-4xl font-semibold">
                {hour}:00
              </h2>
              <span className="m-4">
                { screen !== "" && (
                  <img key={hour}
                    src={'https://iss.localhost/' + screen + '-t'}
                    alt={`screenshot ${hour}`}
                    />
                )}
              </span>
            </div>
            { zipLogs([applications, browsers, filelogs]).map(([timeframe, [applications, browsers, filelogs]]) => (
              <div key={timeframe} className="grid grid-cols-3 gap-4">
                <div className="w-96 col-start-1">
                  {applications.map((app) => (
                    <div key={app.id}>
                      <div>{app.name}</div>
                      <div className="pl-4">{app.title}</div>
                      {/* {JSON.stringify(app)} */}
                    </div>
                  ))}
                </div>
                <div className="w-96 col-start-2">
                  <ul className="list-disc">
                    {browsers.map((browser) => (
                      <li key={browser.id}>
                        <TooltipProvider>
                          <Tooltip>
                            <TooltipTrigger>
                              <div className="text-left">
                                {/* <img src={browser.fav_icon_url} alt="favicon" /> */}
                                <a href={browser.url} target="_blank" rel="noopener noreferrer"
                                  className="decoration-1 underline-offset-2 hover:underline"
                                >
                                  {browser.title}
                                </a>
                              </div>
                            </TooltipTrigger>
                            <TooltipContent side="bottom">
                              {browser.url}
                            </TooltipContent>
                          </Tooltip>
                        </TooltipProvider>
                        {/* {JSON.stringify(browser)} */}
                      </li>
                    ))}
                  </ul>
                </div>
                  <div className="w-96 col-start-3">
                    {filelogs.map((filelog) => (
                      <div key={filelog.id}>
                        {filelog.kind === "create" ? "C" :
                        filelog.kind === "modify" ? "M" : 
                        filelog.kind === "remove" ? "R" :
                        "?"}&nbsp;{filelog.path}
                        {/* {JSON.stringify(filelog)} */}
                      </div>
                    ))}
                  </div>
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}

function zipLogs(logs: [ApplicationLog[], BrowserLog[], FileLog[]]): [number, [ApplicationLog[], BrowserLog[], FileLog[]]][] {
  const [applications, browsers, filelogs] = logs;

  let timeframes = [...new Set([
    ...applications.map((a) => a.timeframe),
    ...browsers.map((b) => b.timeframe),
    ...filelogs.map((f) => f.timeframe),
  ])].sort();

  let zipped: [number, [ApplicationLog[], BrowserLog[], FileLog[]]][] = [];

  let appIndex = 0;
  let browserIndex = 0;
  let filelogIndex = 0;

  for (let t of timeframes) {
    let apps = [];
    while (appIndex < applications.length && applications[appIndex].timeframe === t) {
      apps.push(applications[appIndex]);
      appIndex++;
    }

    let brs = [];
    while (browserIndex < browsers.length && browsers[browserIndex].timeframe === t) {
      brs.push(browsers[browserIndex]);
      browserIndex++;
    }

    let fls = [];
    while (filelogIndex < filelogs.length && filelogs[filelogIndex].timeframe === t) {
      fls.push(filelogs[filelogIndex]);
      filelogIndex++;
    }

    zipped.push([t, [apps, brs, fls]]);
  }

  return zipped;
}

export default HourlyPage;
