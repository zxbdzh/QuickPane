import {
  Globe2,
  History,
  LoaderCircle,
  Moon,
  Pin,
  Plus,
  Search,
  VolumeX,
  X,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";

import type { TabRecord, Workspace } from "../types";
import { cn } from "../lib/utils";
import { INDICATOR_TRANSITION, tabMotion } from "../lib/motion";
import { Button } from "./ui/button";
import { WorkspaceMenu } from "./workspace-menu";

/** 标签栏：总高 32px（与导航栏合计 86px，对应 Rust 侧 CHROME_HEIGHT）。 */
function TabStrip({
  tabs,
  activeId,
  workspaces,
  activeWorkspaceId,
  onSelect,
  onContextMenu,
  onClose,
  onNew,
  onOpenPalette,
  onCreateWorkspace,
  onRenameWorkspace,
  onRemoveWorkspace,
  onSwitchWorkspace,
  onOverlayOpenChange,
}: {
  tabs: TabRecord[];
  activeId: string | null;
  workspaces: Workspace[];
  activeWorkspaceId: string | null;
  onSelect: (tab: TabRecord) => void;
  onContextMenu: (tabId: string, event: React.MouseEvent) => void;
  onClose: (id: string) => void;
  onNew: () => void;
  /** 搜索/最近关闭按钮 → 统一快速切换面板。 */
  onOpenPalette: (mode: "all" | "closed") => void;
  onCreateWorkspace: (name: string) => void;
  onRenameWorkspace: (workspaceId: string, name: string) => void;
  onRemoveWorkspace: (workspaceId: string) => void;
  onSwitchWorkspace: (workspaceId: string) => void;
  /** 工作区菜单开合时上报：驱动 main WebView 扩幅，保证下拉盖在网页上。 */
  onOverlayOpenChange: (open: boolean) => void;
}) {
  return (
    <div className="relative z-50 flex h-8 shrink-0 select-none items-end bg-chrome pr-2 pl-2">
      <WorkspaceMenu
        workspaces={workspaces}
        activeWorkspaceId={activeWorkspaceId}
        activeWorkspaceTabCount={tabs.length}
        onCreate={onCreateWorkspace}
        onRename={onRenameWorkspace}
        onRemove={onRemoveWorkspace}
        onSwitch={onSwitchWorkspace}
        onOpenChange={onOverlayOpenChange}
      />

      <div className="flex min-w-0 flex-1 items-end gap-0.5 self-stretch overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
        <AnimatePresence initial={false}>
          {tabs.map((tab) => {
            const active = tab.id === activeId;
            return (
              <motion.div
                key={tab.id}
                tabIndex={0}
                role="button"
                layout="position"
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
                ) : tab.hibernated ? (
                  <Moon
                    className="size-3.5 shrink-0 text-faint"
                    aria-label="休眠中"
                  />
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
          aria-label="快速切换面板"
          title="快速切换面板 (Ctrl+K)"
          onClick={() => onOpenPalette("all")}
          className="mb-1 text-muted-foreground hover:text-foreground"
        >
          <Search className="size-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="iconSm"
          aria-label="最近关闭的标签页"
          onClick={() => onOpenPalette("closed")}
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
    </div>
  );
}

export { TabStrip };
