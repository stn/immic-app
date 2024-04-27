import React from "react";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

import type { ApplicationLog } from "@/lib/events";
import { timestamp_mmss } from "@/lib/utils";

export interface ApplicationlogProps
    extends React.HTMLAttributes<HTMLDivElement> {
    applicationlog: ApplicationLog
    showTime?: boolean
}

const ApplicationLogItem = React.forwardRef<HTMLDivElement, ApplicationlogProps>(
    ({ applicationlog, showTime=true, className, ...props }, ref) => {
        return (
            <div className={className} ref={ref} {...props}>
                <TooltipProvider>
                    <Tooltip>
                        <TooltipTrigger>
                            <div className="text-left">
                                { showTime && <span>{timestamp_mmss(applicationlog.timestamp)}&nbsp;</span>}
                                <span>{applicationlog.name}</span>
                            </div>
                            <div className="text-left pl-4">
                                {applicationlog.title}
                            </div>
                            {/* {JSON.stringify(applicationlog)} */}
                        </TooltipTrigger>
                        <TooltipContent side="bottom">
                            {applicationlog.path}
                        </TooltipContent>
                    </Tooltip>
                </TooltipProvider>
            </div>
        );
    }
);
ApplicationLogItem.displayName = "ApplicationLogItem";

export { ApplicationLogItem }
