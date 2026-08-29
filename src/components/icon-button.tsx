import type { ComponentProps } from "react";

import { Button } from "./ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip";
import { cn } from "../lib/utils";

/** 工具栏图标按钮：自带 Tooltip 与 aria-label，保证键盘可见焦点态 */
function IconButton({
  label,
  shortcut,
  className,
  children,
  ...props
}: ComponentProps<typeof Button> & { label: string; shortcut?: string }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          aria-label={label}
          className={cn("text-muted-foreground hover:text-foreground", className)}
          {...props}
        >
          {children}
        </Button>
      </TooltipTrigger>
      <TooltipContent>
        {label}
        {shortcut ? <span className="ml-1.5 text-background/55">{shortcut}</span> : null}
      </TooltipContent>
    </Tooltip>
  );
}

export { IconButton };
