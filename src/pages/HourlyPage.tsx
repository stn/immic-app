import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";

import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

import {
  image_url,
  listTimelineOn,
} from "@/lib/api";
import {
  ApplicationLog,
  BrowserLog,
  FileLog,
  ScreenshotLog
} from "@/lib/events";

function HourlyPage() {
  const params = useParams();

  const [timeline, setTimeline] = useState<[string, [ScreenshotLog[], ApplicationLog[], BrowserLog[], FileLog[]]][]>();
  const [screenUrl, setScreenUrl] = useState<string>("");

  const setTimeframe = (_timeframe: number, screens: ScreenshotLog[]) => {
    if (screens.length > 0) {
      setScreenUrl(image_url(screens[0]));
    }
  };

  useEffect(() => {
    let isMounted = true;

    try {
      // const ts = new Date(`${params.year}-${params.month}-${params.day}T00:00:00`).getTime();
      (async () => {
        let logs = await listTimelineOn(`${params.year}${params.month}${params.day}`);
        // let logs = await listTimeline(ts, "Hourly");
        if (isMounted) {
          setTimeline(logs);
          if (logs.length > 0 && logs[0][1][0].length > 0) {
            setScreenUrl(image_url(logs[0][1][0][0]));
          }
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
    <div
      style={{ "backgroundImage": `url(${screenUrl})` } as React.CSSProperties}
      className="bg-fixed bg-contain bg-center bg-no-repeat"
      >
      <div className="sticky top-0 pb-2 bg-transparent/20">
        <h1 className="text-4xl font-semibold mb-6 bg-black">
          <Link to="/">
            {params.year}/{params.month}/{params.day}
          </Link>
        </h1>
      </div>
      <div className="mt-20 bg-transparent/20">
        { timeline && timeline.map(([hour, [screens, applications, browsers, filelogs]]) => (
          <div key={hour} className="my-4">
            <h2 className="text-4xl font-semibold mb-2">
              {hour}:00
            </h2>
            { zipLogs([screens, applications, browsers, filelogs]).map(([timeframe, [screens, applications, browsers, filelogs]]) => (
              <div
               key={timeframe}
               className="grid grid-cols-3 gap-4 hover:bg-transparent/80"
               onMouseEnter={() => setTimeframe(timeframe, screens)}
               >
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

function zipLogs(logs: [ScreenshotLog[], ApplicationLog[], BrowserLog[], FileLog[]]): [number, [ScreenshotLog[], ApplicationLog[], BrowserLog[], FileLog[]]][] {
  const [screens, applications, browsers, filelogs] = logs;

  let timeframes = [...new Set([
    ...screens.map((s) => s.timeframe),
    ...applications.map((a) => a.timeframe),
    ...browsers.map((b) => b.timeframe),
    ...filelogs.map((f) => f.timeframe),
  ])].sort();

  let zipped: [number, [ScreenshotLog[], ApplicationLog[], BrowserLog[], FileLog[]]][] = [];

  let screenIndex = 0;
  let appIndex = 0;
  let browserIndex = 0;
  let filelogIndex = 0;

  for (let t of timeframes) {
    let scrs = [];
    while (screenIndex < screens.length && screens[screenIndex].timeframe === t) {
      scrs.push(screens[screenIndex]);
      screenIndex++;
    }

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

    zipped.push([t, [scrs, apps, brs, fls]]);
  }

  return zipped;
}

export default HourlyPage;
