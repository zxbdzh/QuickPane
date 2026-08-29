import { Bookmark, Download, History, Lock, Puzzle, Settings as SettingsIcon } from "lucide-react";
import type { ReactNode } from "react";
import { useEffect, useState } from "react";
import { emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { api } from "../api";
import { cn } from "../lib/utils";
import { Kbd } from "./ui/kbd";

type MenuSection = "history" | "bookmarks" | "downloads" | "extensions" | "settings";

/**
 * 菜单弹层窗口的内容：独立于主窗口，避免被标签 WebView 盖住。
 * 由主窗口通过 show_menu_window 按按钮锚点调起；失焦由 Rust 侧隐藏。
 */
function MenuWindow() {
  const [hasPassword, setHasPassword] = useState(false);

  useEffect(() => {
    document.documentElement.style.background = "transparent";
    document.body.style.background = "transparent";
    void api
      .snapshot()
      .then((snapshot) => setHasPassword(Boolean(snapshot.data.settings.passwordHash)))
      .catch(() => {});
  }, []);

  useEffect(() => {
    const handler = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape") void getCurrentWindow().hide();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, []);

  // 点击窗口外部即收起。创建瞬间焦点可能抖动（false→true），用短防抖避免误隐藏。
  useEffect(() => {
    const menuWindow = getCurrentWindow();
    let timer: number | undefined;
    const unlisten = menuWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        window.clearTimeout(timer);
      } else {
        timer = window.setTimeout(() => void menuWindow.hide(), 150);
      }
    });
    return () => {
      window.clearTimeout(timer);
      void unlisten.then((dispose) => dispose());
    };
  }, []);

  const close = () => void getCurrentWindow().hide();

  const openSection = (section: MenuSection) => {
    void emit("open-section", section);
    close();
  };

  const lock = () => {
    api.lockNow().catch((reason) => {
      void emit("shortcut-error", reason instanceof Error ? reason.message : String(reason));
    });
    close();
  };

  const items: Array<{ key: string; icon: ReactNode; label: string; hint?: string; action: () => void }> = [
    { key: "history", icon: <History className="size-4" />, label: "历史记录", hint: "Ctrl+H", action: () => openSection("history") },
    { key: "bookmarks", icon: <Bookmark className="size-4" />, label: "书签", action: () => openSection("bookmarks") },
    { key: "downloads", icon: <Download className="size-4" />, label: "下载", hint: "Ctrl+J", action: () => openSection("downloads") },
  ];
  if (hasPassword) {
    items.push({ key: "lock", icon: <Lock className="size-4" />, label: "立即锁定", action: lock });
  }
  items.push({ key: "extensions", icon: <Puzzle className="size-4" />, label: "扩展", action: () => openSection("extensions") });
  items.push({ key: "settings", icon: <SettingsIcon className="size-4" />, label: "设置", action: () => openSection("settings") });

  return (
    <div className="m-1.5 overflow-hidden rounded-lg border border-border bg-popover p-1.5 text-popover-foreground shadow-lg">
      {items.map((item) => (
        <button
          type="button"
          key={item.key}
          onClick={item.action}
          className={cn(
            "flex h-9 w-full items-center gap-2.5 rounded-md px-2.5 text-sm outline-none transition-colors",
            "hover:bg-muted focus-visible:bg-muted",
          )}
        >
          {item.icon}
          {item.label}
          {item.hint ? <Kbd className="ml-auto">{item.hint}</Kbd> : null}
        </button>
      ))}
    </div>
  );
}

export { MenuWindow };
