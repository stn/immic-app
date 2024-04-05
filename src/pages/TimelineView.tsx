import { useEffect, useState } from "react";

import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

import { ApplicationLog, BrowserLog, FileLog } from "../lib/events";
import {
  listDates,
  listTimeline,
} from "../lib/api";
import type { Interval } from "../lib/api";

export interface TimelineViewProps {
  timestamp?: number;
}

export function TimelineView(props: TimelineViewProps) {
  const [timestamp, setTimestamp] = useState<number>(props.timestamp || Date.now());
  const [interval, setInterval] = useState<Interval>("Hourly");
  const [date, setDate] = useState<string>("");
  const [dates, setDates] = useState<string[]>([]);
  const [year, setYear] = useState<string>("");
  const [month, setMonth] = useState<string>("");
  const [day, setDay] = useState<string>("");
  const [timeline, setTimeline] = useState<[string, [string, ApplicationLog[], BrowserLog[], FileLog[]]][]>();

  const dailyView = () => {
    setInterval("Daily");
  }

  const hourlyView = (d: string) => {
    const ts = new Date(d.slice(0, 4) + "-" + d.slice(4, 6) + "-" + d.slice(6, 8)).getTime();
    setTimestamp(ts);
    setDate(d);
    setInterval("Hourly");
  }

  useEffect(() => {
    if (date === "") {
      let d = new Date(timestamp).toISOString().split('T')[0];
      d = d.replace(/-/g, '');
      setDate(d);
    }
  }, []);

  useEffect(() => {
    let isMounted = true;

    setYear(date.slice(0, 4));
    setMonth(date.slice(4, 6));
    setDay(date.slice(6, 8));
    
    if (interval === "Daily") {
      (async () => {
        let ds = await listDates();
        if (isMounted) {
          setDates(ds);
        }
      })();
    } else if (interval === "Hourly") {
      (async () => {
        let logs = await listTimeline(timestamp, interval);
        if (isMounted) {
          setTimeline(logs);
        }
      })();
    }

    return () => {
      isMounted = false;
    };
  }, [interval, date]);

  return (
    <div className="m-4">
      { interval === "Daily" && (
        <div>
          { dates && dates.map((d) => (
            <div key={d}
              className="text-5xl font-semibold my-6"
            >
              <button onClick={() => hourlyView(d)}>
                {d.slice(0, 4)} / {d.slice(4, 6)} / {d.slice(6, 8)}
              </button>
            </div>
          ))}
        </div>
      )}
      { interval === "Hourly" && (
        <div>
          <h1 className="text-5xl font-semibold my-6">
            <button onClick={dailyView}>
              {year} / {month} / {day}
            </button>
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
      )}
    </div>
  );
}
