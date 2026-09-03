import { Globe2, Pin, Volume2, VolumeX, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { api } from "../api";
import type { AppSnapshot, TabRecord } from "../types";

function TabMenuWindow() {
  const [tab, setTab] = useState<TabRecord | null>(null);
  const [active, setActive] = useState(false);
  const menuGeneration = useRef(0);

  useEffect(() => {
    document.documentElement.style.background = "transparent";
    document.body.style.background = "transparent";
    const applyState = (payload: { tab: TabRecord; active: boolean }) => {
      menuGeneration.current += 1;
      setTab(payload.tab);
      setActive(payload.active);
    };
    const cleanup = listen<{ tab: TabRecord; active: boolean }>(
      "tab-menu-state",
      ({ payload }) => applyState(payload),
    ).then((dispose) => {
      void api
        .getTabMenuState()
        .then((payload) => {
          if (payload) applyState(payload);
        })
        .catch(() => {});
      return dispose;
    });
    return () => void cleanup.then((dispose) => dispose());
  }, []);

  useEffect(() => {
    const handler = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape") void getCurrentWindow().hide();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, []);

  const run = async (action: () => Promise<AppSnapshot>, keepOpen = false) => {
    const generation = menuGeneration.current;
    try {
      const snapshot = await action();
      if (keepOpen) {
        const nextTab = snapshot.data.tabs.find((item) => item.id === tab?.id);
        if (nextTab) setTab(nextTab);
      }
    } catch (reason) {
      void emit(
        "shortcut-error",
        reason instanceof Error ? reason.message : String(reason),
      );
    } finally {
      if (!keepOpen && menuGeneration.current === generation) {
        await getCurrentWindow().hide();
      }
    }
  };

  if (!tab) return null;

  const items = [
    {
      label: active ? "当前标签页" : "激活标签页",
      icon: Globe2,
      disabled: active,
      action: () => api.selectTab(tab.id),
    },
    {
      label: tab.pinned ? "取消固定" : "固定标签页",
      icon: Pin,
      action: () => api.setTabPinned(tab.id, !tab.pinned),
    },
    {
      label: tab.muted ? "取消静音" : "静音标签页",
      icon: tab.muted ? Volume2 : VolumeX,
      action: () => api.setTabMuted(tab.id, !tab.muted),
      keepOpen: true,
    },
    {
      label: "关闭标签页",
      icon: X,
      action: () => api.removeTab(tab.id),
      separated: true,
    },
  ];

  return (
    <div className="m-1.5 overflow-hidden rounded-lg border border-border bg-popover p-1 text-popover-foreground shadow-lg">
      {items.map((item) => {
        const Icon = item.icon;
        return (
          <div
            key={item.label}
            className={
              item.separated ? "border-t border-border pt-1 mt-1" : undefined
            }
          >
            <button
              type="button"
              disabled={item.disabled}
              onClick={() =>
                void run(
                  item.action,
                  "keepOpen" in item ? item.keepOpen : false,
                )
              }
              className="flex h-8 w-full items-center gap-2 rounded-sm px-2 text-sm outline-none transition-colors hover:bg-muted focus-visible:bg-muted disabled:pointer-events-none disabled:opacity-50"
            >
              <Icon className="size-4 text-muted-foreground" />
              {item.label}
            </button>
          </div>
        );
      })}
    </div>
  );
}

export { TabMenuWindow };
