import {
  Check as CheckIcon,
  FolderInput,
  Globe2,
  Layers,
  Moon,
  Search,
  Star,
  VolumeX,
  X,
} from "lucide-react";
import { useMemo, useState } from "react";

import type { TabBatchAction } from "../api";
import type { AppSnapshot, TabRecord, Workspace } from "../types";
import { hostOf } from "../lib/format";
import { matchesTextQuery } from "../lib/text-search";
import { cn } from "../lib/utils";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogTitle,
} from "./ui/alert-dialog";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from "./ui/select";

function Check({
  checked,
  label,
  onChange,
}: {
  checked: boolean;
  label: string;
  onChange: () => void;
}) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={checked}
      aria-label={label}
      onClick={(event) => {
        event.stopPropagation();
        onChange();
      }}
      className={cn(
        "grid size-4 shrink-0 place-items-center rounded-[4px] border transition-colors focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none",
        checked
          ? "border-primary bg-primary text-on-soft"
          : "border-input bg-background hover:border-foreground/30",
      )}
    >
      {checked ? <CheckIcon className="size-3" /> : null}
    </button>
  );
}

/** 标签批量管理页：按域名/标题筛选，批量收藏、静音、移入工作区、关闭。 */
function TabsManagerPage({
  snapshot,
  workspaces,
  onBatch,
  onSelectTab,
}: {
  snapshot: AppSnapshot;
  /** 「移入工作区」目标：调用方传入当前工作区之外的选项。 */
  workspaces: Workspace[];
  onBatch: (
    action: TabBatchAction,
    tabIds: string[],
    workspaceId?: string,
  ) => void;
  onSelectTab: (tabId: string) => void;
}) {
  const [query, setQuery] = useState("");
  const [domain, setDomain] = useState<string | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [confirmClose, setConfirmClose] = useState(false);
  const tabs = snapshot.data.tabs;

  const domains = useMemo(() => {
    const counts = new Map<string, number>();
    for (const tab of tabs) {
      const host = hostOf(tab.url);
      if (!host) continue;
      counts.set(host, (counts.get(host) ?? 0) + 1);
    }
    return [...counts.entries()]
      .sort((left, right) => right[1] - left[1])
      .slice(0, 6)
      .map(([host]) => host);
  }, [tabs]);

  const filtered = useMemo(
    () =>
      tabs.filter(
        (tab) =>
          (!domain || hostOf(tab.url) === domain) &&
          matchesTextQuery(query, tab.title, tab.url),
      ),
    [domain, query, tabs],
  );

  const selected = filtered.filter((tab) => selectedIds.has(tab.id));
  const allSelected =
    filtered.length > 0 && filtered.every((tab) => selectedIds.has(tab.id));
  const hasUnmuted = selected.some((tab) => !tab.muted);

  const toggle = (tabId: string) => {
    setSelectedIds((previous) => {
      const next = new Set(previous);
      if (next.has(tabId)) next.delete(tabId);
      else next.add(tabId);
      return next;
    });
  };

  const toggleAll = () => {
    setSelectedIds(
      allSelected ? new Set() : new Set(filtered.map((tab) => tab.id)),
    );
  };

  const runBatch = (action: TabBatchAction, workspaceId?: string) => {
    const tabIds = selected.map((tab) => tab.id);
    if (!tabIds.length) return;
    onBatch(action, tabIds, workspaceId);
    setSelectedIds(new Set());
  };

  return (
    <div className="mx-auto w-full max-w-[880px] px-6 pt-7 pb-16">
      <header className="flex items-end justify-between gap-4 pb-4">
        <div>
          <h1 className="text-xl font-semibold">标签管理</h1>
          <p className="mt-0.5 text-xs text-muted-foreground">
            <span className="font-mono">{tabs.length}</span> 个标签
          </p>
        </div>
      </header>

      <label className="relative block pb-3">
        <Search className="pointer-events-none absolute top-1/2 left-3 size-3.5 -translate-y-1/2 text-muted-foreground" />
        <Input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="搜索标题或网址"
          aria-label="搜索标签"
          className="h-8 pl-8 text-xs"
        />
      </label>

      {domains.length ? (
        <div className="flex flex-wrap items-center gap-1 pb-3">
          {[null, ...domains].map((host) => (
            <button
              key={host ?? "all"}
              type="button"
              onClick={() => setDomain(host)}
              className={cn(
                "rounded-full border px-2 py-0.5 font-mono text-[11px] transition-colors",
                domain === host
                  ? "border-primary/40 bg-primary/10 text-primary"
                  : "border-border text-muted-foreground hover:border-foreground/20 hover:text-foreground",
              )}
            >
              {host ?? "全部"}
            </button>
          ))}
        </div>
      ) : null}

      {filtered.length === 0 ? (
        <div className="flex flex-col items-center justify-center gap-3 rounded-lg border border-dashed py-20 text-center">
          <span className="grid size-12 place-items-center rounded-md bg-soft text-on-soft [&_svg]:size-5">
            <Layers />
          </span>
          <div>
            <p className="text-sm font-medium">
              {tabs.length ? "没有匹配的标签" : "当前工作区还没有标签"}
            </p>
            <p className="mt-1 text-xs text-muted-foreground">
              {tabs.length
                ? "换个关键词或域名筛选试试。"
                : "打开几个网页后，可以在这里批量管理它们。"}
            </p>
          </div>
        </div>
      ) : (
        <div className="overflow-hidden rounded-lg border bg-surface">
          {selected.length > 0 ? (
            <div className="sticky top-0 z-10 flex flex-wrap items-center gap-1.5 border-b bg-popover/95 px-3 py-2 backdrop-blur-sm">
              <span className="mr-1 font-mono text-xs text-primary">
                已选 {selected.length}
              </span>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => runBatch("bookmark")}
              >
                <Star className="size-3.5" />
                收藏
              </Button>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => runBatch(hasUnmuted ? "mute" : "unmute")}
              >
                <VolumeX className="size-3.5" />
                {hasUnmuted ? "静音" : "取消静音"}
              </Button>
              <Select
                onValueChange={(workspaceId) =>
                  runBatch("move", workspaceId)
                }
              >
                <SelectTrigger size="sm" className="w-auto gap-1.5">
                  <FolderInput className="size-3.5" />
                  移入工作区
                </SelectTrigger>
                <SelectContent>
                  {workspaces.length ? (
                    workspaces.map((workspace) => (
                      <SelectItem key={workspace.id} value={workspace.id}>
                        {workspace.name}
                      </SelectItem>
                    ))
                  ) : (
                    <div className="px-2 py-1.5 text-xs text-muted-foreground">
                      没有其它工作区
                    </div>
                  )}
                </SelectContent>
              </Select>
              <Button
                variant="destructive"
                size="sm"
                onClick={() => setConfirmClose(true)}
              >
                <X className="size-3.5" />
                关闭
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="ml-auto"
                onClick={() => setSelectedIds(new Set())}
              >
                取消选择
              </Button>
            </div>
          ) : null}

          <div className="grid grid-cols-[28px_24px_minmax(0,1fr)_auto] items-center gap-3 border-b border-border/70 bg-muted/40 px-3 py-1.5">
            <Check
              checked={allSelected}
              label="全选"
              onChange={toggleAll}
            />
            <span className="font-mono text-[10px] tracking-widest text-faint uppercase">
              标签
            </span>
            <span className="font-mono text-[10px] tracking-widest text-faint uppercase">
              网址
            </span>
            <span className="font-mono text-[10px] tracking-widest text-faint uppercase">
              状态
            </span>
          </div>

          {filtered.map((tab) => (
            <TabRow
              key={tab.id}
              tab={tab}
              active={tab.id === snapshot.data.activeTabId}
              checked={selectedIds.has(tab.id)}
              onToggle={() => toggle(tab.id)}
              onSelect={() => onSelectTab(tab.id)}
            />
          ))}
        </div>
      )}

      <AlertDialog
        open={confirmClose}
        onOpenChange={setConfirmClose}
      >
        <AlertDialogContent>
          <AlertDialogTitle>关闭选中的标签？</AlertDialogTitle>
          <AlertDialogDescription>
            将关闭 {selected.length} 个标签，固定标签不会被跳过，此操作不可撤销。
          </AlertDialogDescription>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => {
                runBatch("close");
                setConfirmClose(false);
              }}
            >
              关闭
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

function TabRow({
  tab,
  active,
  checked,
  onToggle,
  onSelect,
}: {
  tab: TabRecord;
  active: boolean;
  checked: boolean;
  onToggle: () => void;
  onSelect: () => void;
}) {
  return (
    <div
      onClick={onSelect}
      onKeyDown={(event) => {
        if (event.target !== event.currentTarget) return;
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect();
        }
      }}
      tabIndex={0}
      role="button"
      aria-label={`切换到 ${tab.title || "新标签页"}`}
      className={cn(
        "grid cursor-default grid-cols-[28px_24px_minmax(0,1fr)_auto] items-center gap-3 border-b border-border/70 px-3 py-2 transition-colors last:border-b-0 hover:bg-muted/70",
        active && "bg-primary/5",
        "outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:ring-inset",
      )}
    >
      <Check
        checked={checked}
        label={`选择 ${tab.title || "新标签页"}`}
        onChange={onToggle}
      />
      {tab.muted ? (
        <VolumeX className="size-4 shrink-0 text-primary" />
      ) : tab.hibernated ? (
        <Moon className="size-4 shrink-0 text-faint" />
      ) : (
        <Globe2 className="size-4 shrink-0 text-muted-foreground" />
      )}
      <div className="min-w-0">
        <p className="flex items-center gap-1.5 truncate text-sm font-medium">
          <span className="truncate">{tab.title || "新标签页"}</span>
          {active ? (
            <span className="shrink-0 font-mono text-[10px] text-accent2">
              当前
            </span>
          ) : null}
        </p>
        <p className="truncate font-mono text-xs text-muted-foreground">
          {tab.url}
        </p>
      </div>
      <div className="flex items-center gap-1.5">
        {tab.pinned ? (
          <span className="font-mono text-[10px] text-faint">固定</span>
        ) : null}
        {tab.muted ? (
          <span className="font-mono text-[10px] text-primary">静音</span>
        ) : null}
        {tab.hibernated ? (
          <span className="font-mono text-[10px] text-faint">休眠</span>
        ) : null}
        <span className="max-w-24 truncate font-mono text-[10px] text-faint">
          {hostOf(tab.url)}
        </span>
      </div>
    </div>
  );
}

export { TabsManagerPage };
