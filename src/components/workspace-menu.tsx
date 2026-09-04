import {
  ChevronDown,
  Layers,
  Pencil,
  Plus,
  Trash2,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

import type { Workspace } from "../types";
import { overlay } from "../lib/motion";
import { cn } from "../lib/utils";

/** 工作区切换菜单：tab-strip 左端 chip + portal 下拉（创建/重命名/切换/删除）。 */
function WorkspaceMenu({
  workspaces,
  activeWorkspaceId,
  activeWorkspaceTabCount,
  onCreate,
  onRename,
  onRemove,
  onSwitch,
  onOpenChange,
}: {
  workspaces: Workspace[];
  activeWorkspaceId: string | null;
  /** 当前工作区的标签数以 data.tabs 为准（暂存模型：切走时才写回记录）。 */
  activeWorkspaceTabCount: number;
  onCreate: (name: string) => void;
  onRename: (workspaceId: string, name: string) => void;
  onRemove: (workspaceId: string) => void;
  onSwitch: (workspaceId: string) => void;
  /** 开合上报：驱动 main WebView 扩幅，保证下拉盖在网页上。 */
  onOpenChange: (open: boolean) => void;
}) {
  const [open, setOpen] = useState(false);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const [creating, setCreating] = useState(false);
  const [createValue, setCreateValue] = useState("");
  const [removeConfirmId, setRemoveConfirmId] = useState<string | null>(null);
  const renameInputRef = useRef<HTMLInputElement>(null);
  const createInputRef = useRef<HTMLInputElement>(null);
  const removeTimer = useRef<number | null>(null);

  const activeWorkspace =
    workspaces.find((workspace) => workspace.id === activeWorkspaceId) ??
    null;
  const canRemoveAny = workspaces.length > 1;

  useEffect(() => {
    onOpenChange(open);
  }, [onOpenChange, open]);

  useEffect(() => {
    if (renamingId) renameInputRef.current?.focus();
  }, [renamingId]);

  useEffect(() => {
    if (creating) createInputRef.current?.focus();
  }, [creating]);

  useEffect(
    () => () => {
      if (removeTimer.current !== null)
        window.clearTimeout(removeTimer.current);
    },
    [],
  );

  const close = () => {
    setOpen(false);
    setRenamingId(null);
    setCreating(false);
    setCreateValue("");
    setRemoveConfirmId(null);
  };

  const commitRename = (workspaceId: string) => {
    const name = renameValue.trim();
    if (name) onRename(workspaceId, name);
    setRenamingId(null);
  };

  const requestRemove = (workspaceId: string) => {
    // 两段式确认：第一次点击进入确认态（2.5s 超时自动取消），再次点击才删除。
    if (removeConfirmId === workspaceId) {
      if (removeTimer.current !== null)
        window.clearTimeout(removeTimer.current);
      removeTimer.current = null;
      setRemoveConfirmId(null);
      onRemove(workspaceId);
      return;
    }
    setRemoveConfirmId(workspaceId);
    if (removeTimer.current !== null)
      window.clearTimeout(removeTimer.current);
    removeTimer.current = window.setTimeout(
      () => setRemoveConfirmId(null),
      2500,
    );
  };

  return (
    <div className="relative shrink-0">
      <button
        type="button"
        aria-label="切换工作区"
        aria-haspopup="menu"
        aria-expanded={open}
        title="工作区"
        onClick={() => setOpen((value) => !value)}
        className={cn(
          "mb-1 flex h-7 max-w-40 items-center gap-1.5 rounded-md px-2 text-xs transition-colors focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none",
          open
            ? "bg-muted text-foreground"
            : "text-muted-foreground hover:bg-muted hover:text-foreground",
        )}
      >
        <Layers className="size-3.5 shrink-0 text-primary" />
        <span className="truncate">
          {activeWorkspace?.name ?? "工作区"}
        </span>
        <ChevronDown className="size-3 shrink-0" />
      </button>

      {createPortal(
        <AnimatePresence initial={false}>
          {open ? (
            <>
              <div
                aria-hidden
                className="fixed inset-0 z-[90]"
                onClick={close}
              />
              <motion.div
                role="menu"
                aria-label="工作区"
                variants={overlay}
                initial="initial"
                animate="animate"
                exit="exit"
                onKeyDown={(event) => {
                  if (event.key === "Escape") {
                    event.preventDefault();
                    close();
                  }
                }}
                className="fixed top-9 left-2 z-[95] w-64 overflow-hidden rounded-lg border bg-popover p-1.5 text-popover-foreground shadow-popover"
              >
                {workspaces.map((workspace) => {
                  const current = workspace.id === activeWorkspaceId;
                  const count = current
                    ? activeWorkspaceTabCount
                    : workspace.tabs.length;
                  if (renamingId === workspace.id) {
                    return (
                      <div
                        key={workspace.id}
                        className="flex h-9 items-center gap-1.5 rounded-md bg-muted px-1.5"
                      >
                        <input
                          ref={renameInputRef}
                          value={renameValue}
                          onChange={(event) =>
                            setRenameValue(event.target.value)
                          }
                          onKeyDown={(event) => {
                            if (event.key === "Enter") {
                              event.preventDefault();
                              commitRename(workspace.id);
                            } else if (event.key === "Escape") {
                              event.preventDefault();
                              event.stopPropagation();
                              setRenamingId(null);
                            }
                          }}
                          onBlur={() => commitRename(workspace.id)}
                          maxLength={24}
                          aria-label="重命名工作区"
                          className="h-7 min-w-0 flex-1 rounded-sm border border-input bg-background px-1.5 text-xs outline-none focus-visible:border-ring"
                        />
                      </div>
                    );
                  }
                  return (
                    <div
                      key={workspace.id}
                      className={cn(
                        "group relative flex h-9 items-center gap-1 rounded-md pl-3 pr-1 transition-colors",
                        current ? "bg-primary/10" : "hover:bg-muted",
                      )}
                    >
                      {current ? (
                        <i
                          aria-hidden
                          className="absolute top-1/2 left-1 h-4 w-0.5 -translate-y-1/2 rounded-full bg-accent2"
                        />
                      ) : null}
                      <button
                        type="button"
                        role="menuitem"
                        onClick={() => {
                          if (current) return;
                          onSwitch(workspace.id);
                          close();
                        }}
                        className="flex min-w-0 flex-1 items-center gap-2 text-left focus-visible:outline-none"
                      >
                        <span
                          className={cn(
                            "truncate text-xs",
                            current
                              ? "font-medium text-foreground"
                              : "text-foreground",
                          )}
                        >
                          {workspace.name}
                        </span>
                      </button>
                      <span className="shrink-0 font-mono text-[10px] text-faint">
                        {count} 标签
                      </span>
                      <button
                        type="button"
                        aria-label={`重命名 ${workspace.name}`}
                        onClick={() => {
                          setRenamingId(workspace.id);
                          setRenameValue(workspace.name);
                        }}
                        className="grid size-6 shrink-0 place-items-center rounded-sm text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100 hover:bg-foreground/10 hover:text-foreground focus-visible:opacity-100"
                      >
                        <Pencil className="size-3" />
                      </button>
                      <button
                        type="button"
                        aria-label={`删除 ${workspace.name}`}
                        disabled={!canRemoveAny || current}
                        onClick={() => requestRemove(workspace.id)}
                        className={cn(
                          "grid size-6 shrink-0 place-items-center rounded-sm transition-opacity",
                          !canRemoveAny || current
                            ? "cursor-not-allowed text-faint opacity-0 group-hover:opacity-30"
                            : removeConfirmId === workspace.id
                              ? "text-destructive opacity-100"
                              : "text-muted-foreground opacity-0 group-hover:opacity-100 hover:text-destructive",
                        )}
                      >
                        <Trash2 className="size-3" />
                      </button>
                    </div>
                  );
                })}

                <div className="mt-1 border-t border-border/60 pt-1">
                  {creating ? (
                    <div className="flex h-9 items-center rounded-md bg-muted px-1.5">
                      <input
                        ref={createInputRef}
                        value={createValue}
                        onChange={(event) =>
                          setCreateValue(event.target.value)
                        }
                        onKeyDown={(event) => {
                          if (event.key === "Enter") {
                            event.preventDefault();
                            const name = createValue.trim();
                            if (name) {
                              onCreate(name);
                              close();
                            }
                          } else if (event.key === "Escape") {
                            event.preventDefault();
                            event.stopPropagation();
                            setCreating(false);
                            setCreateValue("");
                          }
                        }}
                        maxLength={24}
                        placeholder="工作区名称"
                        aria-label="新建工作区名称"
                        className="h-7 min-w-0 flex-1 rounded-sm border border-input bg-background px-1.5 text-xs outline-none placeholder:text-faint focus-visible:border-ring"
                      />
                    </div>
                  ) : (
                    <button
                      type="button"
                      role="menuitem"
                      onClick={() => {
                        setCreating(true);
                        setCreateValue("");
                      }}
                      className="flex h-8 w-full items-center gap-2 rounded-md px-2 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none"
                    >
                      <Plus className="size-3.5" />
                      新建工作区
                    </button>
                  )}
                </div>
              </motion.div>
            </>
          ) : null}
        </AnimatePresence>,
        document.body,
      )}
    </div>
  );
}

export { WorkspaceMenu };
