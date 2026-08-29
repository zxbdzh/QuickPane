import { Pin, PinOff, Plus, Puzzle, SquareArrowOutUpRight, X } from "lucide-react";
import { useEffect, useState } from "react";

import { api } from "../api";
import type { AppSnapshot, BrowserExtension } from "../types";
import { cn } from "../lib/utils";
import { Button } from "./ui/button";
import { Switch } from "./ui/switch";

/** 扩展管理页：安装/启停/卸载/固定未打包的 Chrome/Edge 扩展。 */
function ExtensionsPage({ run, onOpen, pinnedIds, applySnapshot }: {
  run: <T>(action: () => Promise<T>) => Promise<T | undefined>;
  onOpen: (url: string) => void;
  pinnedIds: string[];
  applySnapshot: (value: AppSnapshot) => void;
}) {
  const [extensions, setExtensions] = useState<BrowserExtension[] | null>(null);

  useEffect(() => {
    void run(api.listExtensions).then((items) => {
      if (items) setExtensions(items);
    });
    // 仅在进入页面时加载一次；后续操作直接用返回值刷新。
  }, [run]);

  const install = () => {
    void run(api.installExtension).then((items) => {
      if (items) setExtensions(items);
    });
  };

  return (
    <div className="mx-auto w-full max-w-[880px] px-6 pt-7 pb-16">
      <header className="flex items-end justify-between gap-4 pb-4">
        <div>
          <h1 className="text-xl font-semibold">扩展</h1>
          <p className="mt-0.5 text-xs text-muted-foreground">
            安装未打包的 Chrome/Edge 扩展文件夹，更改后已打开的页面会重新加载。
          </p>
        </div>
        <Button variant="ghost" size="sm" className="text-primary hover:bg-soft hover:text-primary" onClick={install}>
          <Plus className="size-4" />添加扩展
        </Button>
      </header>

      {extensions === null ? null : extensions.length === 0 ? (
        <div className="grid place-items-center gap-3 rounded-lg border border-dashed border-border py-20 text-center">
          <div className="grid size-12 place-items-center rounded-full bg-muted">
            <Puzzle className="size-5 text-muted-foreground" />
          </div>
          <div>
            <p className="text-sm font-medium">尚未安装扩展</p>
            <p className="mt-1 text-xs text-muted-foreground">
              选择一个包含 manifest.json 的文件夹即可完成安装。
            </p>
          </div>
          <Button variant="secondary" size="sm" onClick={install}>
            <Plus className="size-4" />添加扩展
          </Button>
        </div>
      ) : (
        <div className="flex flex-col gap-2">
          {extensions.map((extension) => (
            <div
              className={cn(
                "flex items-center gap-3 rounded-lg border border-border bg-surface px-4 py-3",
                !extension.enabled && "opacity-50",
              )}
              key={extension.id}
            >
              {extension.icon ? (
                <img src={extension.icon} alt="" className="size-8 shrink-0 rounded-sm" />
              ) : (
                <div className="grid size-8 shrink-0 place-items-center rounded-sm bg-muted">
                  <Puzzle className="size-4 text-muted-foreground" />
                </div>
              )}
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">
                  {extension.name}
                  <span className="ml-2 text-xs text-muted-foreground">{extension.version}</span>
                </p>
                {extension.description ? (
                  <p className="truncate text-xs text-muted-foreground">{extension.description}</p>
                ) : null}
              </div>
              {extension.popupUrl && extension.enabled ? (
                <Button
                  variant="ghost"
                  size="sm"
                  aria-label={`打开 ${extension.name} 面板`}
                  title="在置顶小窗中打开扩展面板"
                  className="shrink-0 text-primary hover:bg-soft hover:text-primary"
                  onClick={() => onOpen(extension.popupUrl as string)}
                >
                  <SquareArrowOutUpRight className="size-4" />打开面板
                </Button>
              ) : null}
              <Button
                variant="ghost"
                size="iconSm"
                aria-label={pinnedIds.includes(extension.id) ? `从导航栏取消固定 ${extension.name}` : `固定 ${extension.name} 到导航栏`}
                title={extension.popupUrl ? undefined : "该扩展没有面板，固定后点击无效果"}
                disabled={!extension.popupUrl}
                className={cn(
                  "shrink-0",
                  pinnedIds.includes(extension.id)
                    ? "text-primary hover:bg-soft hover:text-primary"
                    : "text-muted-foreground hover:bg-muted hover:text-foreground",
                )}
                onClick={() => {
                  const pinned = !pinnedIds.includes(extension.id);
                  void run(() => api.toggleExtensionPin(extension.id, pinned)).then((next) => {
                    if (next) applySnapshot(next);
                  });
                }}
              >
                {pinnedIds.includes(extension.id) ? <PinOff className="size-3.5" /> : <Pin className="size-3.5" />}
              </Button>
              <Switch
                checked={extension.enabled}
                aria-label={`启用或禁用 ${extension.name}`}
                onCheckedChange={(enabled) => {
                  void run(() => api.setExtensionEnabled(extension.id, enabled)).then((items) => {
                    if (items) setExtensions(items);
                  });
                }}
              />
              <Button
                variant="ghost"
                size="iconSm"
                aria-label={`卸载 ${extension.name}`}
                className="text-muted-foreground hover:bg-destructive-soft hover:text-destructive"
                onClick={() => {
                  void run(() => api.removeExtension(extension.id)).then((items) => {
                    if (items) setExtensions(items);
                  });
                }}
              >
                <X className="size-3.5" />
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export { ExtensionsPage };
