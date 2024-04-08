import { useEffect } from "react";
import { useLocation } from "react-router-dom";

import {
  searchBrowserLogs,
} from "@/lib/api";

export interface SearchPageProps {
}

export function SearchPage(_props: SearchPageProps) {
  let { state } = useLocation();
  
  useEffect(() => {
    (async () => {
     const result = await searchBrowserLogs(state.query);
     console.log(result);
    })();
  }, [state.query]);

  return (
    <div className="">
      <h1>Search</h1>
      <p>Query: {state.query}</p>
    </div>
  );
}
