import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";

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
import {
  timestamp_hhmm,
} from "@/lib/utils";
import { FileLogItem } from "@/components/elements/FilelogItem";
import { BrowserLogItem } from "@/components/elements/BrowserLogItem";
import { ApplicationLogItem } from "@/components/elements/ApplicationLogItem";

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
                        <ApplicationLogItem key={app.id} applicationlog={app} showTime={false} />
                      ))}
                    </div>
                    <div className="flex-auto w-96">
                      {browsers.map((browser) => (
                        <BrowserLogItem key={browser.id} browserlog={browser} />
                      ))}
                    </div>
                    <div className="flex-auto w-96">
                      <div className="ml-10">
                        {filelogs.map((filelog) => (
                          <FileLogItem
                            key={filelog.id}
                            filelog={filelog}
                          />
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

export default HourlyPage;
