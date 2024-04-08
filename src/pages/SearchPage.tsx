import { useEffect, useState } from "react";
import { useLocation, Link } from "react-router-dom";

import {
  SearchLogsResults,
  searchLogs,
} from "@/lib/api";

export interface SearchPageProps {
}

export function SearchPage(_props: SearchPageProps) {
  const { state } = useLocation();

  const [hits, setHits] = useState<SearchLogsResults>();
  
  useEffect(() => {
    (async () => {
     const result = await searchLogs(state.query);
     setHits(result);
    })();
  }, [state.query]);

  return (
    <div className="">
      {
        hits?.hits.map((hit) => (
          <div key={hit.date} className="mb-4">
            <div className="mb-1">
              <Link to={`/${dateToPath(hit.date)}`}>
                <span className="text-xl font-semibold mr-4">
                  {dateToPath(hit.date)}
                </span>
              </Link>
              <span className="">({hit.hits} hits)</span>
            </div>
            <ul>
              { hit.application_path && <li>Application Path: {hit.application_path}</li> }
              { hit.application_title && <li>Application Title: {hit.application_title}</li> }
              { hit.browser_title && <li>Browser Title: {hit.browser_title}</li> }
              { hit.browser_url && <li>Browser URL: {hit.browser_url}</li> }
              { hit.file_path && <li>File Path: {hit.file_path}</li> }
            </ul>
          </div>
        ))
      }
    </div>
  );
}

function dateToPath(date: string) {
  return `${date.slice(0, 4)}/${date.slice(4, 6)}/${date.slice(6, 8)}`;
}
