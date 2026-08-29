import type { ComponentProps } from "react";

import { cn } from "../../lib/utils";

/** 快捷键提示，放在菜单项右侧（配合 ml-auto 右对齐） */
function Kbd({ className, ...props }: ComponentProps<"kbd">) {
  return (
    <kbd
      className={cn(
        "inline-flex h-[18px] min-w-[18px] items-center justify-center rounded-sm border bg-muted px-1 font-sans text-[11px] font-normal tracking-wide text-muted-foreground",
        className,
      )}
      {...props}
    />
  );
}

export { Kbd };
