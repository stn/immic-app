import React from "react";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

import type { BrowserLog } from "@/lib/events";
import { cn, timestamp_mmss } from "@/lib/utils";

export interface BrowserlogProps
    extends React.HTMLAttributes<HTMLDivElement> {
    browserlog: BrowserLog
    showTime?: boolean
}

const BrowserLogItem = React.forwardRef<HTMLDivElement, BrowserlogProps>(
    ({ browserlog, showTime=true, className, ...props }, ref) => {
        const url = new URL(browserlog.url);

        return (
            <div className={cn("indent-11", className)} ref={ref} {...props}>
                <TooltipProvider>
                    <Tooltip>
                        <TooltipTrigger>
                            <div className="text-left -indent-11">
                                { showTime && <span>{timestamp_mmss(browserlog.timestamp)}&nbsp;</span>}
                                <img src={`https://s2.googleusercontent.com/s2/favicons?domain=${url.hostname}`} alt="" className="inline-block" width="16" height="16" />
                                &nbsp;
                                <a href={browserlog.url} target="_blank" rel="noopener noreferrer"
                                    className="decoration-1 underline-offset-2 hover:underline"
                                >
                                    {browserlog.title}
                                </a>
                            </div>
                            {/* {JSON.stringify(browserlog)} */}
                        </TooltipTrigger>
                        <TooltipContent side="bottom">
                            {browserlog.url}
                        </TooltipContent>
                    </Tooltip>
                </TooltipProvider>
            </div>
        );
    }
);
BrowserLogItem.displayName = "BrowserLogItem";

export { BrowserLogItem }
