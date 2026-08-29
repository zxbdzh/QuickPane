import type { ComponentProps } from "react";

import { cn } from "../../lib/utils";

function Input({ className, type, ...props }: ComponentProps<"input">) {
  return (
    <input
      type={type}
      className={cn(
        "flex h-9 w-full min-w-0 rounded-md border border-input bg-surface px-3 py-1 text-sm text-foreground transition-[border-color,box-shadow] outline-none",
        "placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground",
        "focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50",
        "disabled:cursor-not-allowed disabled:bg-muted disabled:text-faint",
        className,
      )}
      {...props}
    />
  );
}

export { Input };
