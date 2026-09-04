import {
  Globe2,
  History,
  LoaderCircle,
  Pin,
  Plus,
  RotateCcw,
  Search,
  VolumeX,
  X,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState } from "react";

import type { TabRecord } from "../types";
import { cn } from "../lib/utils";
import { matchesTextQuery } from "../lib/text-search";
import { INDICATOR_TRANSITION, overlay, tabMotion } from "../lib/motion";
import { Button } from "./ui/button";

/** 标签栏：总高 32px（与导航栏合计 86px，对应 Rust 侧 CHROME_HEIGHT）。 */
function TabStrip({
  tabs,
  activeId,
  recentlyClosed,
  onSelect,
  onContextMenu,
  onClose,
  onRestoreClosed,
  onNew,
  onOverlayOpenChange,
}: {
  tabs: TabRecord[];
  activeId: string | null;
  recentlyClosed: TabRecord[];
  onSelect: (tab: TabRecord) => void;
  onContextMenu: (tabId: string, event: React.MouseEvent) => void;
  onClose: (id: string) => void;
  onRestoreClosed: (id?: string) => void;
  onNew: () => void;
  /** 搜索/最近关闭面板开合时上报：驱动 main WebView 扩幅，保证面板盖在网页上。 */
  onOverlayOpenChange: (open: boolean) => void;
}) {
  const [panel, setPanel] = useState<"search" | "closed" | null>(null);
  const [query, setQuery] = useState("");
  const searchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (panel === "search") searchRef.current?.focus();
  }, [panel]);

  // 面板向下弹出会超出收缩态的 chrome 高度，打开时必须扩幅才能完整显示。
  useEffect(() => {
    onOverlayOpenChange(panel !== null);
  }, [onOverlayOpenChange, panel]);

  const filteredTabs = tabs.filter((tab) =>
    matchesTextQuery(query, tab.title, tab.url),
  );

  return (
    <div className="relative z-50 flex h-8 shrink-0 select-none items-end bg-chrome pr-2 pl-2">
      <div className="flex min-w-0 flex-1 items-end gap-0.5 self-stretch overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
        <AnimatePresence initial={false}>
          {tabs.map((tab) => {
            const active = tab.id === activeId;
            return (
              <motion.div
                key={tab.id}
                tabIndex={0}
                role="button"
                layout
                variants={tabMotion}
                initial="initial"
                animate="animate"
                exit="exit"
                onKeyDown={(event) => {
                  if (event.target !== event.currentTarget) return;
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    onSelect(tab);
                  }
                }}
                onClick={() => onSelect(tab)}
                onContextMenu={(event) => onContextMenu(tab.id, event)}
                onAuxClick={(event) => {
                  if (event.button !== 1) return;
                  event.preventDefault();
                  onClose(tab.id);
                }}
                aria-label={
                  active
                    ? `${tab.title || "新标签页"}（当前标签页）`
                    : `切换到 ${tab.title || "新标签页"}`
                }
                className={cn(
                  "group relative grid h-8 w-[180px] min-w-[108px] max-w-[220px] shrink-0 cursor-default grid-cols-[16px_minmax(0,1fr)_18px] items-center gap-1.5 overflow-hidden rounded-t-md px-2.5 text-xs outline-none transition-colors",
                  "focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:ring-inset",
                  active
                    ? "bg-surface text-foreground"
                    : "text-muted-foreground hover:bg-muted",
                )}
              >
                {tab.muted ? (
                  <VolumeX className="size-3.5 shrink-0 text-primary" />
                ) : tab.pinned ? (
                  <Pin className="size-3.5 shrink-0" />
                ) : tab.loading ? (
                  <LoaderCircle className="size-3.5 animate-spin text-accent2" />
                ) : (
                  <Globe2 className="size-3.5 shrink-0" />
                )}
                <span className="truncate text-left">
                  {tab.title || "新标签页"}
                </span>
                <button
                  type="button"
                  aria-label={`关闭 ${tab.title || "新标签页"}`}
                  onClick={(event) => {
                    event.stopPropagation();
                    onClose(tab.id);
                  }}
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
                  <motion.i
                    aria-hidden
                    layoutId="qp-tab-indicator"
                    transition={INDICATOR_TRANSITION}
                    className="absolute inset-x-2.5 bottom-0 h-0.5 rounded-full bg-accent2"
                  />
                ) : null}
              </motion.div>
            );
          })}
        </AnimatePresence>
      </div>

      <div className="flex shrink-0 items-center gap-0.5">
        <Button
          variant="ghost"
          size="iconSm"
          aria-label="搜索标签页"
          title="搜索标签页"
          onClick={() => {
            setPanel(panel === "search" ? null : "search");
            setQuery("");
          }}
          className="mb-1 text-muted-foreground hover:text-foreground"
        >
          <Search className="size-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="iconSm"
          aria-label="最近关闭的标签页"
          title="最近关闭的标签页"
          onClick={() => setPanel(panel === "closed" ? null : "closed")}
          className="mb-1 text-muted-foreground hover:text-foreground"
        >
          <History className="size-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="iconSm"
          aria-label="新建标签页 (Ctrl+T)"
          onClick={onNew}
          className="mb-1 text-muted-foreground hover:text-foreground"
        >
          <Plus className="size-4" />
        </Button>
      </div>

      <AnimatePresence initial={false}>
        {panel ? (
          <motion.div
            variants={overlay}
            initial="initial"
            animate="animate"
            exit="exit"
            className="absolute top-[calc(100%+4px)] right-2 w-[min(360px,calc(100vw-16px))] overflow-hidden rounded-md border bg-popover p-1.5 text-popover-foreground shadow-popover"
          >
            {panel === "search" ? (
              <>
                <div className="flex h-8 items-center gap-2 rounded-sm border border-input bg-background px-2">
                  <Search className="size-3.5 text-muted-foreground" />
                  <input
                    ref={searchRef}
                    value={query}
                    onChange={(event) => setQuery(event.target.value)}
                    onKeyDown={(event) => {
                      if (event.key === "Escape") setPanel(null);
                    }}
                    placeholder="搜索标签页"
                    aria-label="搜索标签页"
                    className="min-w-0 flex-1 bg-transparent text-xs outline-none placeholder:text-muted-foreground"
                  />
                  <span className="font-mono text-[11px] text-faint">
                    {filteredTabs.length}
                  </span>
                </div>
                <div className="mt-1 max-h-64 overflow-y-auto">
                  {filteredTabs.length ? (
                    filteredTabs.map((tab) => (
                      <button
                        key={tab.id}
                        type="button"
                        onClick={() => {
                          onSelect(tab);
                          setPanel(null);
                        }}
                        className="flex h-9 w-full items-center gap-2 rounded-sm px-2 text-left text-xs hover:bg-muted focus-visible:bg-muted focus-visible:outline-none"
                      >
                        {tab.muted ? (
                          <VolumeX className="size-3.5 text-primary" />
                        ) : (
                          <Globe2 className="size-3.5 text-muted-foreground" />
                        )}
                        <span className="min-w-0 flex-1 truncate">
                          {tab.title || "新标签页"}
                        </span>
                        <span className="max-w-36 truncate text-[11px] text-muted-foreground">
                          {tab.url}
                        </span>
                      </button>
                    ))
                  ) : (
                    <p className="px-2 py-4 text-center text-xs text-muted-foreground">
                      没有匹配的标签页
                    </p>
                  )}
                </div>
              </>
            ) : (
              <>
                <div className="flex items-center gap-2 px-2 py-1.5 text-xs font-medium">
                  <RotateCcw className="size-3.5 text-muted-foreground" />
                  最近关闭
                </div>
                {recentlyClosed.length ? (
                  <div className="max-h-64 overflow-y-auto">
                    {recentlyClosed.map((tab) => (
                      <button
                        key={tab.id}
                        type="button"
                        onClick={() => {
                          onRestoreClosed(tab.id);
                          setPanel(null);
                        }}
                        className="flex w-full items-center gap-2 rounded-sm px-2 py-2 text-left hover:bg-muted focus-visible:bg-muted focus-visible:outline-none"
                      >
                        <Globe2 className="size-3.5 shrink-0 text-muted-foreground" />
                        <span className="min-w-0">
                          <strong className="block truncate text-xs font-medium">
                            {tab.title || "新标签页"}
                          </strong>
                          <small className="block truncate text-[11px] text-muted-foreground">
                            {tab.url}
                          </small>
                        </span>
                      </button>
                    ))}
                  </div>
                ) : (
                  <p className="px-2 py-4 text-center text-xs text-muted-foreground">
                    没有最近关闭的标签页
                  </p>
                )}
              </>
            )}
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}

export { TabStrip };
