import { useCallback, useEffect, useState } from "react";
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
  const [visibleContent, setVisibleContent] = useState<boolean>(true);

  const setTimeframe = useCallback((_timeframe: number, screens: ScreenshotLog[]) => {
    if (screens.length > 0) {
      setScreenUrl(image_url(screens[0]));
    }
  }, [setScreenUrl]);

  const toggleVisibleContent = useCallback(() => {
    setVisibleContent(!visibleContent);
  }, [visibleContent, setVisibleContent]);

  useEffect(() => {
    let isMounted = true;

    try {
      (async () => {
        let logs = await listTimelineOn(`${params.year}${params.month}${params.day}`);
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
      <div
        className="mt-20"
        onClick={() => { toggleVisibleContent(); }}
      >
        { !visibleContent && (
          <div className="h-screen bg-transparent">
            &nbsp;
          </div>
        )}
        { visibleContent && (
          <div className="bg-transparent/60">
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
                          <div>{timestamp_mm(app.timestamp)} {app.name}</div>
                          <div className="pl-6">{app.title}</div>
                          {/* {JSON.stringify(app)} */}
                        </div>
                      ))}
                    </div>
                    <div className="w-96 col-start-2">
                      <ul className="list-none ml-10">
                        {browsers.map((browser) => (
                          <li key={browser.id}>
                            <TooltipProvider>
                              <Tooltip>
                                <TooltipTrigger>
                                  <div className="text-left -indent-10">
                                    {/* <img src={browser.fav_icon_url} alt="favicon" /> */}
                                    {timestamp_mmss(browser.timestamp)}&nbsp;
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
                      <div className="ml-10">
                        {filelogs.map((filelog) => (
                          <div key={filelog.id}>
                            <TooltipProvider>
                              <Tooltip>
                                <TooltipTrigger>
                                  <div className="text-left -indent-10">
                                    {timestamp_mmss(filelog.timestamp)}
                                    {filelog.kind === "create" ? "🗒️" :
                                    filelog.kind === "modify" ? "📝" : 
                                    filelog.kind === "remove" ? "🗑️" :
                                    "?"}&nbsp;{filename(filelog.path)}
                                    {/* {JSON.stringify(filelog)} */}
                                  </div>
                                </TooltipTrigger>
                                <TooltipContent side="bottom">
                                  {filelog.path}
                                </TooltipContent>
                              </Tooltip>
                            </TooltipProvider>
                          </div>
                        ))}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function zipLogs(logs: [ScreenshotLog[], ApplicationLog[], BrowserLog[], FileLog[]]): [number, [ScreenshotLog[], ApplicationLog[], BrowserLog[], FileLog[]]][] {
  const [screens, applications, browsers, filelogs] = logs;

  let timeframes = [...new Set([
    ...screens.map((s) => s.timestamp / 60 | 0),
    ...applications.map((a) => a.timestamp / 60 | 0 + 1),
    ...browsers.map((b) => b.timestamp / 60 | 0 + 1),
    ...filelogs.map((f) => f.timestamp / 60 | 0 + 1),
  ])].sort();

  let zipped: [number, [ScreenshotLog[], ApplicationLog[], BrowserLog[], FileLog[]]][] = [];

  let screenIndex = 0;
  let appIndex = 0;
  let browserIndex = 0;
  let filelogIndex = 0;

  for (let t of timeframes) {
    let scrs = [];
    while (screenIndex < screens.length && (screens[screenIndex].timestamp / 60 | 0) === t) {
      scrs.push(screens[screenIndex]);
      screenIndex++;
    }

    let apps = [];
    while (appIndex < applications.length && (applications[appIndex].timestamp / 60 | 0 + 1) === t) {
      apps.push(applications[appIndex]);
      appIndex++;
    }

    let brs = [];
    while (browserIndex < browsers.length && (browsers[browserIndex].timestamp / 60 | 0 + 1) === t) {
      brs.push(browsers[browserIndex]);
      browserIndex++;
    }

    let fls = [];
    while (filelogIndex < filelogs.length && (filelogs[filelogIndex].timestamp / 60 | 0 + 1) === t) {
      fls.push(filelogs[filelogIndex]);
      filelogIndex++;
    }

    zipped.push([t, [scrs, apps, brs, fls]]);
  }

  return zipped;
}

function timestamp_mm(timestamp: number): string {
  let date = new Date(timestamp * 1000);
  return ("0" + date.toLocaleString("en-US", { minute: "numeric" })).slice(-2);
}

function timestamp_mmss(timestamp: number): string {
  let date = new Date(timestamp * 1000);
  return ("0" + date.toLocaleString("en-US", { minute: "2-digit", second: "2-digit" })).slice(-5);
}

function filename(path: string): string {
  return path.split(/[/\\]/).pop() || "";
}

export default HourlyPage;
