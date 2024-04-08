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
      <h1>Search</h1>
      <p>Query: {state.query}</p>
      <div>
        {
          hits?.hits.map((hit) => (
            <div key={hit.date}>
              <h2>{hit.date}</h2>
              <ul>
                <li>Application Path: {hit.application_path}</li>
                <li>Application Title: {hit.application_title}</li>
                <li>Browser Title: {hit.browser_title}</li>
                <li>Browser URL: {hit.browser_url}</li>
                <li>File Path: {hit.file_path}</li>
              </ul>
            </div>
          ))
        }
      </div>
    </div>
  );
}
