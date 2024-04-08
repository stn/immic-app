import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";

import {
  SearchLogsResults,
  searchLogs,
} from "@/lib/api";

export interface SearchPageProps {
}

export function SearchPage(_props: SearchPageProps) {
  let { state } = useLocation();

  let [hits, setHits] = useState<SearchLogsResults>();
  
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
              <span className="text-xl font-semibold mr-4">
                {hit.date.slice(0, 4)}/{hit.date.slice(4, 6)}/{hit.date.slice(6, 8)}
              </span>
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
