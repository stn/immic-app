import React from "react";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

import type { FileLog } from "@/lib/events";
import { cn, filename, timestamp_mmss } from "@/lib/utils";

export interface FilelogProps
    extends React.HTMLAttributes<HTMLDivElement> {
    filelog: FileLog
    showTime?: boolean
}

const FileLogItem = React.forwardRef<HTMLDivElement, FilelogProps>(
    ({ filelog, showTime=true, className, ...props }, ref) => {
        return (
            <div className={cn("indent-16", className)} ref={ref} {...props}>
                <TooltipProvider>
                    <Tooltip>
                        <TooltipTrigger>
                            <div className="text-left -indent-16" >
                                { showTime && <span>{timestamp_mmss(filelog.timestamp)}&nbsp;</span>}
                                <span>
                                    {filelog.kind === "create" ? "🗒️" :
                                    filelog.kind === "modify" ? "📝" : 
                                    filelog.kind === "remove" ? "🗑️" :
                                    "?"}
                                    &nbsp;
                                    {filename(filelog.path)}
                                </span>
                                {/* {JSON.stringify(filelog)} */}
                            </div>
                        </TooltipTrigger>
                        <TooltipContent side="bottom">
                            {filelog.path}
                        </TooltipContent>
                    </Tooltip>
                </TooltipProvider>
            </div>
        );
    }
);
FileLogItem.displayName = "FileLogItem";

export { FileLogItem }

