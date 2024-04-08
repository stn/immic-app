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
  );
}

export default HourlyPage;
