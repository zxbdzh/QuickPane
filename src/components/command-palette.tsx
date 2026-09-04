import {
  Clock3,
  Globe2,
  Layers,
  Moon,
  RotateCcw,
  Search,
  Star,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";

import type {
  Bookmark,
  HistoryEntry,
  TabRecord,
  Workspace,
} from "../types";
import { paletteBackdrop, palettePanel } from "../lib/motion";
import {
  quickSearch,
  type QuickSearchGroupKey,
  type QuickSearchItem,
} from "../lib/quick-search";
import { cn } from "../lib/utils";

/** 统一快速切换面板：标签 / 最近关闭 / 工作区 / 书签 / 历史 五源一屏检索。 */
const GROUP_ORDER: QuickSearchGroupKey[] = [
  "tab",
  "closed",
  "workspace",
  "bookmark",
  "history",
];

const GROUP_LABELS: Record<QuickSearchGroupKey, string> = {
  tab: "标签页",
  closed: "最近关闭",
  workspace: "工作区",
  bookmark: "书签",
  history: "历史",
};

const GROUP_ICONS: Record<QuickSearchGroupKey, typeof Globe2> = {
  tab: Globe2,
  closed: RotateCcw,
  workspace: Layers,
  bookmark: Star,
  history: Clock3,
};

function CommandPalette({
  request,
  tabs,
  recentlyClosed,
  workspaces,
  bookmarks,
  history,
  activeTabId,
  onSelectTab,
  onRestoreClosed,
  onSwitchWorkspace,
  onOpenUrl,
  onOpenChange,
}: {
  /** 快捷键请求：serial 驱动 toggle；mode "closed" 预置最近关闭过滤。 */
  request: { mode: "all" | "closed"; serial: number } | null;
  tabs: TabRecord[];
  recentlyClosed: TabRecord[];
  /** 调用方传入「其它工作区」（排除当前激活）。 */
  workspaces: Workspace[];
  bookmarks: Bookmark[];
  history: HistoryEntry[];
  activeTabId: string | null;
  onSelectTab: (tabId: string) => void;
  onRestoreClosed: (tabId: string) => void;
  onSwitchWorkspace: (workspaceId: string) => void;
  onOpenUrl: (url: string) => void;
  /** 开合上报：驱动 main WebView 扩幅，保证面板盖在网页上。 */
  onOpenChange: (open: boolean) => void;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<QuickSearchGroupKey | null>(null);
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const openRef = useRef(false);
  const modeRef = useRef<"all" | "closed">("all");

  const close = () => {
    setOpen(false);
    setQuery("");
  };

  // 快捷键请求：同模式再按关闭，异模式切换过滤，未开则打开。
  useEffect(() => {
    if (!request) return;
    if (openRef.current && modeRef.current === request.mode) {
      close();
      return;
    }
    modeRef.current = request.mode;
    setFilter(request.mode === "closed" ? "closed" : null);
    setQuery("");
    setActiveIndex(0);
    setOpen(true);
  }, [request]);

  useEffect(() => {
    openRef.current = open;
  }, [open]);

  useEffect(() => {
    onOpenChange(open);
  }, [onOpenChange, open]);

  // 每次呼出（含面板已开时切换过滤模式）都重新聚焦输入框；
  // OS 级键盘焦点由 Rust 侧 set_shell_expanded / 快捷键转发转移到主 WebView。
  useEffect(() => {
    if (!open) return;
    const frame = window.requestAnimationFrame(() =>
      inputRef.current?.focus(),
    );
    return () => window.cancelAnimationFrame(frame);
  }, [open, request]);

  const groups = useMemo(
    () =>
      quickSearch({
        query,
        filter,
        tabs,
        recentlyClosed,
        workspaces,
        bookmarks,
        history,
      }),
    [bookmarks, filter, history, query, recentlyClosed, tabs, workspaces],
  );
  const flatItems = useMemo(
    () => groups.flatMap((group) => group.items),
    [groups],
  );

  // 查询/过滤变化后高亮回到首项，并夹紧到结果范围内。
  useEffect(() => {
    setActiveIndex(0);
  }, [query, filter]);

  useEffect(() => {
    setActiveIndex((index) =>
      Math.min(index, Math.max(flatItems.length - 1, 0)),
    );
  }, [flatItems.length]);

  // 键盘高亮项始终滚入可视区。
  useEffect(() => {
    listRef.current
      ?.querySelector('[data-active="true"]')
      ?.scrollIntoView({ block: "nearest" });
  }, [activeIndex]);

  const execute = (item: QuickSearchItem | undefined) => {
    if (!item) return;
    if (item.group === "tab" && item.tabId) onSelectTab(item.tabId);
    else if (item.group === "closed" && item.tabId)
      onRestoreClosed(item.tabId);
    else if (item.group === "workspace" && item.workspaceId)
      onSwitchWorkspace(item.workspaceId);
    else if (item.url) onOpenUrl(item.url);
    close();
  };

  return createPortal(
    <AnimatePresence initial={false}>
      {open ? (
        <motion.div
          key="command-palette"
          variants={paletteBackdrop}
          initial="initial"
          animate="animate"
          exit="exit"
          className="fixed inset-0 z-[100] grid place-items-center p-6"
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              event.preventDefault();
              close();
            } else if (event.key === "ArrowDown") {
              event.preventDefault();
              setActiveIndex((index) =>
                flatItems.length ? (index + 1) % flatItems.length : 0,
              );
            } else if (event.key === "ArrowUp") {
              event.preventDefault();
              setActiveIndex((index) =>
                flatItems.length
                  ? (index - 1 + flatItems.length) % flatItems.length
                  : 0,
              );
            } else if (event.key === "Enter") {
              event.preventDefault();
              execute(flatItems[activeIndex]);
            }
          }}
        >
        <div
          aria-hidden
          className="absolute inset-0 bg-background/70 backdrop-blur-[2px]"
          onClick={close}
        />
        <motion.div
          role="dialog"
          aria-modal="true"
          aria-label="快速切换"
          variants={palettePanel}
          initial="initial"
          animate="animate"
          exit="exit"
          className="relative flex max-h-[min(520px,calc(100vh-96px))] w-[min(640px,calc(100vw-48px))] flex-col overflow-hidden rounded-lg border bg-popover text-popover-foreground shadow-popover outline-none"
        >
          <div className="flex h-11 shrink-0 items-center gap-2.5 border-b px-3.5">
            <Search className="size-4 shrink-0 text-primary" />
            <input
              ref={inputRef}
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="搜索标签、书签、历史、工作区…"
              aria-label="快速切换搜索"
              className="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
            />
            <span className="shrink-0 font-mono text-[11px] text-faint">
              {flatItems.length}
            </span>
          </div>

          <div className="flex shrink-0 items-center gap-1 border-b px-3 py-1.5">
            {([null, ...GROUP_ORDER] as const).map((key) => (
              <button
                key={key ?? "all"}
                type="button"
                onClick={() => {
                  setFilter(key as QuickSearchGroupKey | null);
                  setActiveIndex(0);
                }}
                className={cn(
                  "rounded-full border px-2 py-0.5 font-mono text-[11px] transition-colors",
                  filter === key
                    ? "border-primary/40 bg-primary/10 text-primary"
                    : "border-border text-muted-foreground hover:border-foreground/20 hover:text-foreground",
                )}
              >
                {key ? GROUP_LABELS[key] : "全部"}
              </button>
            ))}
          </div>

          <div ref={listRef} className="min-h-0 flex-1 overflow-y-auto">
            {flatItems.length ? (
              groups.map((group) => {
                const Icon = GROUP_ICONS[group.key];
                return (
                  <section key={group.key} aria-label={GROUP_LABELS[group.key]}>
                    <header className="sticky top-0 z-10 flex items-center justify-between border-b border-border/60 bg-popover/95 px-3 py-1.5 backdrop-blur-sm">
                      <span className="font-mono text-[10px] tracking-widest text-faint uppercase">
                        {GROUP_LABELS[group.key]}
                      </span>
                      <span className="font-mono text-[10px] text-faint">
                        {group.items.length}
                      </span>
                    </header>
                    {group.items.map((item) => {
                      const flatIndex = flatItems.indexOf(item);
                      const active = flatIndex === activeIndex;
                      const isCurrentTab =
                        item.group === "tab" && item.tabId === activeTabId;
                      return (
                        <button
                          key={item.key}
                          type="button"
                          data-active={active}
                          onClick={() => execute(item)}
                          onMouseEnter={() => setActiveIndex(flatIndex)}
                          className={cn(
                            "flex h-10 w-full items-center gap-2.5 px-3 text-left text-xs transition-colors focus-visible:outline-none",
                            active
                              ? "bg-primary/10"
                              : "hover:bg-muted focus-visible:bg-muted",
                          )}
                        >
                          <Icon
                            className={cn(
                              "size-4 shrink-0",
                              active
                                ? "text-primary"
                                : "text-muted-foreground",
                            )}
                          />
                          <span className="min-w-0 flex-1 truncate">
                            {item.title}
                          </span>
                          {item.hibernated ? (
                            <Moon
                              className="size-3 shrink-0 text-faint"
                              aria-label="休眠中"
                            />
                          ) : null}
                          {isCurrentTab ? (
                            <span className="shrink-0 font-mono text-[10px] text-accent2">
                              当前
                            </span>
                          ) : null}
                          {item.group === "workspace" ? (
                            <span className="shrink-0 font-mono text-[10px] text-faint">
                              {workspaces
                                .find((ws) => ws.id === item.workspaceId)
                                ?.tabs.length ?? 0}{" "}
                              标签
                            </span>
                          ) : item.url ? (
                            <span className="hidden max-w-44 shrink-0 truncate font-mono text-[11px] text-faint min-[480px]:block">
                              {item.url}
                            </span>
                          ) : null}
                        </button>
                      );
                    })}
                  </section>
                );
              })
            ) : (
              <p className="px-4 py-10 text-center text-xs text-muted-foreground">
                没有匹配的结果
              </p>
            )}
          </div>

          <footer className="flex h-7 shrink-0 items-center justify-between border-t px-3.5 font-mono text-[10px] text-faint">
            <span>↑↓ 选择</span>
            <span>Enter 跳转</span>
            <span>Esc 关闭</span>
          </footer>
        </motion.div>
      </motion.div>
      ) : null}
    </AnimatePresence>,
    document.body,
  );
}

export { CommandPalette };
