import type { AppSnapshot, DownloadRecord, QuickLink, Settings } from "../types";

/**
 * 仅供 UI 验证使用的后端模拟（/mock.html 入口）。
 * 在 window.__TAURI_INTERNALS__ 上模拟 invoke / 事件，让真实 App 组件跑通完整事件流。
 * 不会进入生产构建（vite build 仅以 index.html 为入口）。
 */

type SnapshotListener = (event: { event: string; id: number; payload: unknown }) => void;

const listeners = new Map<string, Set<SnapshotListener>>();

const QUICK_LINKS: QuickLink[] = [
  { id: "ql-1", title: "GitHub", url: "https://github.com" },
  { id: "ql-2", title: "哔哩哔哩", url: "https://www.bilibili.com" },
  { id: "ql-3", title: "知乎", url: "https://www.zhihu.com" },
  { id: "ql-4", title: "V2EX", url: "https://www.v2ex.com" },
];

const DEFAULT_SETTINGS: Settings = {
  shortcut: "Alt+Q",
  autostart: false,
  homeUrl: "https://kaodes.com",
  searchTemplate: "https://cn.bing.com/search?q={query}",
  historyDays: 90,
  passwordHash: null,
  lockOnSystemLock: true,
  quickLinks: QUICK_LINKS,
  proxyMode: "system",
  proxyUrl: "",
  pinnedExtensions: [],
};

const DOWNLOADS: DownloadRecord[] = [];

function makeSnapshot(): AppSnapshot {
  const bootLocked = new URLSearchParams(window.location.search).get("locked") === "1";
  return {
    data: {
      tabs: [
        {
          id: "tab-newtab",
          title: "新标签页",
          url: "quickpane://newtab",
          pinned: false,
          loading: false,
          loaded: true,
          muted: false,
          createdAt: new Date().toISOString(),
          lastActiveAt: new Date().toISOString(),
        },
      ],
      activeTabId: "tab-newtab",
      recentlyClosed: [],
      history: [
        { id: "h-1", title: "QuickPane · GitHub", url: "https://github.com/topics/browser", visitedAt: new Date().toISOString() },
        { id: "h-2", title: "V2EX - 创意工作者社区", url: "https://www.v2ex.com", visitedAt: new Date(Date.now() - 3600_000).toISOString() },
      ],
      bookmarks: [],
      downloads: DOWNLOADS,
      settings: { ...DEFAULT_SETTINGS, passwordHash: bootLocked ? "mock-hash" : null },
    },
    locked: bootLocked,
    firstRun: false,
    windowVisible: true,
    pinnedExtensions: [],
  };
}

let snapshot = makeSnapshot();
let mockPassword: string | null = null;
let eventSeq = 1;

function emitEvent(name: string, payload: unknown) {
  listeners.get(name)?.forEach((listener) => listener({ event: name, id: eventSeq++, payload }));
}

function emitSnapshot() {
  emitEvent("app-snapshot", structuredClone(snapshot));
}

function normalizeInput(input: string) {
  const value = input.trim();
  if (/^https?:\/\//i.test(value)) return value;
  if (/^[\w.-]+\.[a-z]{2,}(\/.*)?$/i.test(value)) return `https://${value}`;
  return DEFAULT_SETTINGS.searchTemplate.replace("{query}", encodeURIComponent(value));
}

function titleFromUrl(url: string) {
  try {
    return new URL(url).hostname;
  } catch {
    return url;
  }
}

async function invoke(command: string, args: Record<string, unknown> = {}): Promise<unknown> {
  switch (command) {
    case "plugin:event|listen": {
      const event = args.event as string;
      const handler = args.handler as number;
      const callback = registeredCallbacks.get(handler);
      if (callback) {
        if (!listeners.has(event)) listeners.set(event, new Set());
        listeners.get(event)!.add(callback as SnapshotListener);
      }
      return handler;
    }
    case "plugin:event|unlisten": {
      const event = args.event as string;
      const handler = args.eventId as number;
      const callback = registeredCallbacks.get(handler);
      if (callback) listeners.get(event)?.delete(callback as SnapshotListener);
      return null;
    }
    case "get_snapshot":
      return structuredClone(snapshot);
    case "new_tab": {
      const url = (args.url as string | null) ?? "quickpane://newtab";
      const finalUrl = url === "quickpane://newtab" ? url : normalizeInput(url);
      const tab = {
        id: `tab-${Math.random().toString(36).slice(2, 8)}`,
        title: finalUrl === "quickpane://newtab" ? "新标签页" : titleFromUrl(finalUrl),
        url: finalUrl,
        pinned: false,
        loading: false,
        loaded: true,
        muted: false,
        createdAt: new Date().toISOString(),
        lastActiveAt: new Date().toISOString(),
      };
      snapshot.data.tabs.push(tab);
      snapshot.data.activeTabId = tab.id;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "select_tab": {
      snapshot.data.activeTabId = args.tabId as string;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "remove_tab": {
      const id = args.tabId as string;
      const index = snapshot.data.tabs.findIndex((tab) => tab.id === id);
      if (index >= 0) {
        const [removed] = snapshot.data.tabs.splice(index, 1);
        snapshot.data.recentlyClosed.unshift(removed);
      }
      if (snapshot.data.tabs.length === 0) {
        snapshot.data.tabs.push({
          id: `tab-${Math.random().toString(36).slice(2, 8)}`,
          title: "新标签页",
          url: "quickpane://newtab",
          pinned: false,
          loading: false,
          loaded: true,
          muted: false,
          createdAt: new Date().toISOString(),
          lastActiveAt: new Date().toISOString(),
        });
      }
      if (!snapshot.data.tabs.some((tab) => tab.id === snapshot.data.activeTabId)) {
        snapshot.data.activeTabId = snapshot.data.tabs[snapshot.data.tabs.length - 1].id;
      }
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "navigate": {
      const tab = snapshot.data.tabs.find((item) => item.id === args.tabId);
      if (tab) {
        tab.url = normalizeInput(args.input as string);
        tab.title = titleFromUrl(tab.url);
      }
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "reload":
    case "go_back":
    case "go_forward":
    case "find_in_page":
    case "zoom_page":
    case "hide_to_tray":
    case "exit_app":
    case "open_download":
    case "clear_site_data":
      return null;
    case "show_shell": {
      snapshot.windowVisible = Boolean(args.visible);
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "set_global_shortcut": {
      snapshot.data.settings.shortcut = args.shortcut as string;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "update_settings": {
      Object.assign(snapshot.data.settings, args.update as Record<string, unknown>);
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "add_bookmark": {
      snapshot.data.bookmarks.unshift({
        id: `bm-${Math.random().toString(36).slice(2, 8)}`,
        title: args.title as string,
        url: args.url as string,
        createdAt: new Date().toISOString(),
      });
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "remove_bookmark": {
      snapshot.data.bookmarks = snapshot.data.bookmarks.filter((item) => item.id !== args.bookmarkId);
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "clear_history":
      snapshot.data.history = [];
      emitSnapshot();
      return structuredClone(snapshot);
    case "clear_downloads":
      snapshot.data.downloads = [];
      emitSnapshot();
      return structuredClone(snapshot);
    case "set_app_password": {
      mockPassword = args.newPassword as string;
      snapshot.data.settings.passwordHash = "mock-argon2-hash";
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "disable_app_password": {
      if (args.currentPassword !== mockPassword) throw new Error("密码错误");
      mockPassword = null;
      snapshot.data.settings.passwordHash = null;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "unlock_app": {
      if ((args.password as string) !== mockPassword) throw new Error("应用密码错误");
      snapshot.locked = false;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "skip_password_setup": {
      snapshot.firstRun = false;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "lock_now": {
      snapshot.locked = true;
      emitSnapshot();
      return null;
    }
    default:
      console.warn("[mock] 未实现的命令:", command, args);
      return null;
  }
}

const registeredCallbacks = new Map<number, (response: unknown) => void>();

declare global {
  interface Window {
    __TAURI_INTERNALS__: {
      invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
      transformCallback: (callback: (response: unknown) => void, once?: boolean) => number;
      metadata?: { currentWindow?: { label: string } };
    };
    __mock: {
      emit: typeof emitSnapshot;
      patch: (mutate: (draft: AppSnapshot) => void) => void;
      error: (message: string) => void;
      theme: (value: "light" | "dark") => void;
      openSection: (section: string) => void;
      snapshot: () => AppSnapshot;
    };
  }
}

export function installMockBackend() {
  window.__TAURI_INTERNALS__ = {
    invoke,
    transformCallback: (callback) => {
      const id = eventSeq++;
      registeredCallbacks.set(id, callback);
      return id;
    },
    metadata: { currentWindow: { label: "main" } },
  };

  // ?section=history|bookmarks|downloads|settings：App 挂载后自动切入对应页面（截图辅助）。
  const initialSection = new URLSearchParams(window.location.search).get("section");
  if (initialSection) {
    let tries = 0;
    const timer = window.setInterval(() => {
      emitEvent("open-section", initialSection);
      if (++tries >= 40) window.clearInterval(timer);
    }, 250);
  }

  window.__mock = {
    emit: emitSnapshot,
    patch: (mutate) => {
      mutate(snapshot);
      emitSnapshot();
    },
    error: (message) => emitEvent("shortcut-error", message),
    openSection: (section) => emitEvent("open-section", section),
    theme: (value) => {
      localStorage.setItem("quickpane.theme", value);
      document.documentElement.classList.toggle("dark", value === "dark");
    },
    snapshot: () => structuredClone(snapshot),
  };
}
