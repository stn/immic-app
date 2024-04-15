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
  const [timestamp, setTimestamp] = useState<number>();
  const [screenUrl, setScreenUrl] = useState<string>("");
  const [visibleContent, setVisibleContent] = useState<boolean>(true);

  const setTimeframe = useCallback((_timeframe: number, screens: ScreenshotLog[]) => {
    if (screens.length > 0) {
      setTimestamp(screens[0].timestamp);
      setScreenUrl(image_url(screens[0]));
    }
  }, []);

  const toggleVisibleContent = useCallback(() => {
    setVisibleContent(!visibleContent);
  }, [visibleContent]);

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
      <div className="sticky top-0">
        <h1 className="text-4xl font-semibold pb-4 bg-transparent/80">
          <Link to="/">
            {params.year}/{params.month}/{params.day}
          </Link>
          { timestamp && (
            <span className="text-4xl ml-4">
              {timestamp_hhmm(timestamp)}
            </span>
          )}
        </h1>
      </div>
      <div
        className="m-0 p-0"
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
              <div key={hour} className="py-2">
                <h2 className="text-3xl font-semibold mb-4">
                  {hour}
                </h2>
                { zipLogs([screens, applications, browsers, filelogs]).map(([timeframe, [screens, applications, browsers, filelogs]]) => (
                  <div
                    key={timeframe}
                    className="flex gap-6 hover:bg-transparent/80"
                    onMouseEnter={() => setTimeframe(timeframe, screens)}
                  >
                    <div className="flex-none w-4">
                      {("0" + (timeframe % 60)).slice(-2)}
                    </div>
                    <div className="flex-auto w-96">
                      {applications.map((app) => (
                        <div key={app.id}>
                            <TooltipProvider>
                              <Tooltip>
                                <TooltipTrigger>
                                  {/* <div className="text-left">{timestamp_mm(app.timestamp)} {app.name}</div> */}
                                  <div className="text-left">{app.name}</div>
                                  <div className="text-left pl-6">{app.title}</div>
                                </TooltipTrigger>
                                <TooltipContent side="bottom">
                                  {app.path}
                                </TooltipContent>
                              </Tooltip>
                            </TooltipProvider>
                          {/* {JSON.stringify(app)} */}
                        </div>
                      ))}
                    </div>
                    <div className="flex-auto w-96">
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
                    <div className="flex-auto w-96">
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
    ...screens.map((s) => Math.floor(s.timestamp / 60)),
    ...applications.map((a) => Math.floor(a.timestamp / 60)),
    ...browsers.map((b) => Math.floor(b.timestamp / 60)),
    ...filelogs.map((f) => Math.floor(f.timestamp / 60)),
  ])].sort();

  let zipped: [number, [ScreenshotLog[], ApplicationLog[], BrowserLog[], FileLog[]]][] = [];

  let screenIndex = 0;
  let appIndex = 0;
  let browserIndex = 0;
  let filelogIndex = 0;

  for (let t of timeframes) {
    let scrs = [];
    while (screenIndex < screens.length && Math.floor(screens[screenIndex].timestamp / 60) === t) {
      scrs.push(screens[screenIndex]);
      screenIndex++;
    }

    let apps = [];
    while (appIndex < applications.length && Math.floor(applications[appIndex].timestamp / 60) === t) {
      apps.push(applications[appIndex]);
      appIndex++;
    }

    let brs = [];
    while (browserIndex < browsers.length && Math.floor(browsers[browserIndex].timestamp / 60) === t) {
      brs.push(browsers[browserIndex]);
      browserIndex++;
    }

    let fls = [];
    while (filelogIndex < filelogs.length && Math.floor(filelogs[filelogIndex].timestamp / 60) === t) {
      fls.push(filelogs[filelogIndex]);
      filelogIndex++;
    }

    zipped.push([t, [scrs, apps, brs, fls]]);
  }

  return zipped;
}

// function timestamp_mm(timestamp: number): string {
//   let date = new Date(timestamp * 1000);
//   return ("0" + date.toLocaleTimeString("ja-JP", { minute: "numeric" })).slice(-2);
// }

function timestamp_mmss(timestamp: number): string {
  let date = new Date(timestamp * 1000);
  return ("0" + date.toLocaleTimeString("ja-JP", { minute: "2-digit", second: "2-digit" })).slice(-5);
}

function timestamp_hhmm(timestamp: number): string {
  let date = new Date(timestamp * 1000);
  return ("0" + date.toLocaleTimeString("ja-JP", { hour: "2-digit", minute: "2-digit"})).slice(-5);
}

function filename(path: string): string {
  return path.split(/[/\\]/).pop() || "";
}

export default HourlyPage;
