import { useEffect, useState } from "react";
import { useLocation, Link } from "react-router-dom";

import { Label } from "@/components/ui/label";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion"

import {
  SearchHit,
  SearchLogsResults,
  searchLogs,
  openHourly,
} from "@/lib/api";
import { timestamp_hhmm } from "@/lib/utils";

export interface SearchPageProps {
}

export function SearchPage(_props: SearchPageProps) {
  const { state } = useLocation();

  const [hits, setHits] = useState<SearchLogsResults>();
  
  useEffect(() => {
    (async () => {
     const result = await searchLogs(state.query);
     console.log(result);
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
              <span className="">({hit.count})</span>
            </div>
            <div>
              { hit.application_title_count && (
                <SearchHits
                  label={`Application Title (${hit.application_title_count})`}
                  hits={hit.application_title_hits!}
                />
              )}
              { hit.application_name_count && (
                <SearchHits
                  label={`Application Name (${hit.application_name_count})`}
                  hits={hit.application_name_hits!}
                />
              )}
              { hit.browser_title_count && (
                <SearchHits
                  label={`Browser Title (${hit.browser_title_count})`}
                  hits={hit.browser_title_hits!}
                />
              )}
              { hit.browser_url_count && (
                <SearchHits
                  label={`Browser URL (${hit.browser_url_count})`}
                  hits={hit.browser_url_hits!}
                />
              )}
              { hit.file_path_count && (
                <SearchHits
                  label={`File Path (${hit.file_path_count})`}
                  hits={hit.file_path_hits!}
                />
              )}
            </div>
          </div>
        ))
      }
    </div>
  );
}

function dateToPath(date: string) {
  return `${date.slice(0, 4)}/${date.slice(4, 6)}/${date.slice(6, 8)}`;
}

function SearchHits({ label, hits }: { label: string, hits: SearchHit[] }) {
  return (
    <Accordion
      type="single"
      collapsible
      className="ml-4"
    >
      <AccordionItem value={label}>
        <AccordionTrigger className="py-1">
          {label}
        </AccordionTrigger>
        <AccordionContent className="pl-4">
          {hits.map((hit: SearchHit, i: number) => (
            <div>
              <Label
                key={i}
                className="ml-2 mt-1"
                onClick={async () => {await openHourly(hit.timestamp)}}
              >
                {timestamp_hhmm(hit.timestamp)}
                &nbsp;
                {hit.text || ""}
              </Label>
            </div>
          ))}
        </AccordionContent>
      </AccordionItem>
    </Accordion>
  );
}
