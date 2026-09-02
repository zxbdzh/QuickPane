import { Globe2, LoaderCircle, Plus, X } from "lucide-react";

import type { TabRecord } from "../types";
import { cn } from "../lib/utils";
import { Button } from "./ui/button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "./ui/context-menu";

/** 标签栏：总高 32px（与导航栏合计 86px，对应 Rust 侧 CHROME_HEIGHT） */
function TabStrip({ tabs, activeId, onSelect, onClose, onNew }: {
  tabs: TabRecord[];
  activeId: string | null;
  onSelect: (tab: TabRecord) => void;
  onClose: (id: string) => void;
  onNew: () => void;
}) {
  return (
    <div className="flex h-8 shrink-0 select-none items-end bg-chrome pr-2 pl-2">
      <div className="flex min-w-0 flex-1 items-end gap-0.5 self-stretch overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
        {tabs.map((tab) => {
          const active = tab.id === activeId;
          return (
            <ContextMenu key={tab.id}>
              <ContextMenuTrigger asChild>
                <div
                  tabIndex={0}
                  role="button"
                  onKeyDown={(event) => {
                    if (event.target !== event.currentTarget) return;
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      onSelect(tab);
                    }
                  }}
                  onClick={() => onSelect(tab)}
                  aria-label={active ? `${tab.title || "新标签页"}（当前标签页）` : `切换到 ${tab.title || "新标签页"}`}
                  className={cn(
                    "group relative grid h-8 w-[180px] min-w-[108px] max-w-[220px] shrink-0 cursor-default grid-cols-[16px_minmax(0,1fr)_18px] items-center gap-1.5 rounded-t-md px-2.5 text-xs outline-none transition-colors",
                    "focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:ring-inset",
                    active ? "bg-surface text-foreground" : "text-muted-foreground hover:bg-muted",
                  )}
                >
                  {tab.loading ? (
                    <LoaderCircle className="size-3.5 animate-spin" />
                  ) : (
                    <Globe2 className="size-3.5 shrink-0" />
                  )}
                  <span className="truncate text-left">{tab.title || "新标签页"}</span>
                  <button
                    type="button"
                    aria-label={`关闭 ${tab.title || "新标签页"}`}
                    onClick={(event) => { event.stopPropagation(); onClose(tab.id); }}
                    className={cn(
                      "grid size-[18px] place-items-center rounded-full transition-opacity hover:bg-foreground/10 focus-visible:opacity-100 focus-visible:ring-[3px] focus-visible:ring-ring/50",
                      active
                        ? "opacity-80 hover:opacity-100"
                        : "opacity-0 group-focus-within:opacity-80 group-hover:opacity-80",
                    )}
                  >
                    <X className="size-3" />
                  </button>
                  {active ? (
                    <i aria-hidden className="absolute inset-x-2.5 bottom-0 h-0.5 rounded-full bg-primary" />
                  ) : null}
                </div>
              </ContextMenuTrigger>
              <ContextMenuContent>
                <ContextMenuItem onSelect={() => onSelect(tab)}>
                  <Globe2 />激活标签页
                </ContextMenuItem>
                <ContextMenuSeparator />
                <ContextMenuItem onSelect={() => onClose(tab.id)}>
                  <X />关闭标签页
                </ContextMenuItem>
              </ContextMenuContent>
            </ContextMenu>
          );
        })}
      </div>
      <Button
        variant="ghost"
        size="iconSm"
        aria-label="新建标签页 (Ctrl+T)"
        onClick={onNew}
        className="mb-1 ml-0.5 text-muted-foreground hover:text-foreground"
      >
        <Plus className="size-4" />
      </Button>
      <div aria-hidden className="mt-1.5 ml-1 grid size-[18px] place-items-center rounded-sm bg-primary text-[10px] font-bold text-primary-foreground">
        Q
      </div>
    </div>
  );
}

export { TabStrip };
