import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Settings } from "lucide-react";

import { Input } from "@/components/ui/input"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

import {
  Interval,
  listDates,
  listTimeline,
  searchBrowserLogs,
} from "@/lib/api";
import {
  ApplicationLog,
  BrowserLog,
  FileLog
} from "@/lib/events";

import { useForm, SubmitHandler } from "react-hook-form";

type SearchInputs = {
  query: string;
}

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

  const { register, handleSubmit } = useForm<SearchInputs>();
  const onSubmit: SubmitHandler<SearchInputs> = async (data) => {
     console.log(data);
     const result = await searchBrowserLogs(data.query);
     console.log(result);
  };

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
    <>
      <header className="sticky top-0 h-16 items-center bg-transparent px-4">
        <nav className="flex gap-6 text-lg font-medium mt-2">
          <div className="ml-[550px] w-[1000px]">
            <form onSubmit={handleSubmit(onSubmit)}>
              <Input
                type="search"
                className="bg-transparent focus:bg-background pl-8"
                {...register("query")}
              />
              <input type="submit" className="hidden" />
            </form>
          </div>
          <div className="ml-auto h-12 pt-2">
            <Link
              to="/settings"
              className="text-muted-foreground transition-colors hover:text-foreground ml-auto"
            >
              <Settings className="h-6 w-6 text-muted-foreground" />
            </Link>
          </div>
        </nav>
      </header>
      <main className="flex flex-1 flex-col gap-4 bg-background pl-4 pr-4">
        <div className="">
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
              <h1 className="text-5xl font-semibold mb-6">
                <button onClick={dailyView}>
                  {year} / {month} / {day}
                </button>
              </h1>
              <div>
                { timeline && timeline.map(([hour, [screen, applications, browsers, filelogs]]) => (
                  <div key={hour} className="my-4">
                    <h2 className="text-4xl font-semibold">
                      {hour}:00
                    </h2>
                    <div className="grid grid-cols-4 gap-4">
                      <div className="m-4">
                        { screen !== "" && (
                          <img key={hour}
                            src={'https://iss.localhost/' + screen + '-t'}
                            alt={`screenshot ${hour}`}
                            />
                        )}
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
      </main>
    </>
  );
}
