import type {
  AppSnapshot,
  DownloadRecord,
  QuickLink,
  Settings,
  TabRecord,
} from "../types";

/**
 * 仅供 UI 验证使用的后端模拟（/mock.html 入口）。
 * 在 window.__TAURI_INTERNALS__ 上模拟 invoke / 事件，让真实 App 组件跑通完整事件流。
 * 不会进入生产构建（vite build 仅以 index.html 为入口）。
 */

type SnapshotListener = (event: {
  event: string;
  id: number;
  payload: unknown;
}) => void;

const listeners = new Map<string, Set<SnapshotListener>>();

const QUICK_LINKS: QuickLink[] = [
  { id: "ql-1", title: "GitHub", url: "https://github.com" },
  { id: "ql-2", title: "哔哩哔哩", url: "https://www.bilibili.com" },
  { id: "ql-3", title: "知乎", url: "https://www.zhihu.com" },
  { id: "ql-4", title: "V2EX", url: "https://www.v2ex.com" },
];

const DEFAULT_SETTINGS: Settings = {
  shortcut: "Alt+Q",
  paletteShortcut: "Ctrl+Shift+A",
  autostart: false,
  homeUrl: "https://kaodes.com",
  searchTemplate: "https://cn.bing.com/search?q={query}",
  historyDays: 90,
  passwordHash: null,
  lockOnSystemLock: true,
  autoLockAfterHideSeconds: 0,
  quickLinks: QUICK_LINKS,
  proxyMode: "system",
  proxyUrl: "",
  pinnedExtensions: [],
  tabHibernationMinutes: 15,
};

const DOWNLOADS: DownloadRecord[] = [];

function makeTab(url = "quickpane://newtab", title?: string): TabRecord {
  const finalUrl =
    url === "quickpane://newtab" ? url : normalizeInput(url);
  return {
    id: `tab-${Math.random().toString(36).slice(2, 8)}`,
    title: title ?? (finalUrl === "quickpane://newtab" ? "新标签页" : titleFromUrl(finalUrl)),
    url: finalUrl,
    pinned: false,
    loading: false,
    loaded: true,
    muted: false,
    hibernated: false,
    createdAt: new Date().toISOString(),
    lastActiveAt: new Date().toISOString(),
  };
}

/** 暂存模型：data.tabs 始终是当前工作区的标签，切走时写回工作区记录。 */
function stashCurrentTabs() {
  const active = snapshot.data.workspaces.find(
    (workspace) => workspace.id === snapshot.data.activeWorkspaceId,
  );
  if (active) {
    active.tabs = structuredClone(snapshot.data.tabs);
    active.activeTabId = snapshot.data.activeTabId;
  }
}

function loadWorkspaceTabs(workspaceId: string) {
  const target = snapshot.data.workspaces.find(
    (workspace) => workspace.id === workspaceId,
  );
  if (!target) throw new Error("工作区不存在");
  if (target.tabs.length === 0) target.tabs = [makeTab()];
  snapshot.data.tabs = structuredClone(target.tabs);
  snapshot.data.activeTabId = target.activeTabId ?? target.tabs[0].id;
  snapshot.data.activeWorkspaceId = workspaceId;
}

function makeSnapshot(): AppSnapshot {
  const bootLocked =
    new URLSearchParams(window.location.search).get("locked") === "1";
  const initialTabs = [makeTab()];
  return {
    data: {
      tabs: initialTabs,
      activeTabId: initialTabs[0].id,
      recentlyClosed: [],
      history: [
        {
          id: "h-1",
          title: "QuickPane · GitHub",
          url: "https://github.com/topics/browser",
          visitedAt: new Date().toISOString(),
        },
        {
          id: "h-2",
          title: "V2EX - 创意工作者社区",
          url: "https://www.v2ex.com",
          visitedAt: new Date(Date.now() - 3600_000).toISOString(),
        },
      ],
      bookmarks: [],
      downloads: DOWNLOADS,
      settings: { ...DEFAULT_SETTINGS, passwordHash: null },
      workspaces: [
        { id: "ws-default", name: "默认工作区", tabs: [], activeTabId: null },
        {
          id: "ws-study",
          name: "学习",
          tabs: [
            {
              id: "tab-study-1",
              title: "Rust 文档",
              url: "https://doc.rust-lang.org",
              pinned: false,
              loading: false,
              loaded: false,
              muted: false,
              hibernated: true,
              createdAt: new Date().toISOString(),
              lastActiveAt: new Date(Date.now() - 7200_000).toISOString(),
            },
            {
              id: "tab-study-2",
              title: "MDN Web Docs",
              url: "https://developer.mozilla.org",
              pinned: false,
              loading: false,
              loaded: true,
              muted: false,
              hibernated: false,
              createdAt: new Date().toISOString(),
              lastActiveAt: new Date(Date.now() - 3600_000).toISOString(),
            },
          ],
          activeTabId: "tab-study-2",
        },
      ],
      activeWorkspaceId: "ws-default",
    },
    locked: bootLocked,
    firstRun: false,
    hasPassword: bootLocked,
    windowVisible: true,
    pinnedExtensions: [],
    recoveryMessage: null,
  };
}

let snapshot = makeSnapshot();
let mockPassword: string | null = null;
if (new URLSearchParams(window.location.search).get("locked") === "1")
  mockPassword = "password";
let eventSeq = 1;

function emitEvent(name: string, payload: unknown) {
  listeners
    .get(name)
    ?.forEach((listener) => listener({ event: name, id: eventSeq++, payload }));
}

function emitSnapshot() {
  emitEvent("app-snapshot", structuredClone(snapshot));
}

function normalizeInput(input: string) {
  const value = input.trim();
  if (/^https?:\/\//i.test(value)) return value;
  if (/^[\w.-]+\.[a-z]{2,}(\/.*)?$/i.test(value)) return `https://${value}`;
  return DEFAULT_SETTINGS.searchTemplate.replace(
    "{query}",
    encodeURIComponent(value),
  );
}

function titleFromUrl(url: string) {
  try {
    return new URL(url).hostname;
  } catch {
    return url;
  }
}

async function invoke(
  command: string,
  args: Record<string, unknown> = {},
): Promise<unknown> {
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
      const tab = makeTab(url);
      snapshot.data.tabs.push(tab);
      snapshot.data.tabs.sort(
        (left, right) => Number(right.pinned) - Number(left.pinned),
      );
      snapshot.data.activeTabId = tab.id;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "select_tab": {
      snapshot.data.activeTabId = args.tabId as string;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "set_tab_pinned": {
      const tab = snapshot.data.tabs.find((item) => item.id === args.tabId);
      if (tab) {
        tab.pinned = Boolean(args.pinned);
        snapshot.data.tabs.sort(
          (left, right) => Number(right.pinned) - Number(left.pinned),
        );
      }
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "set_tab_muted": {
      const tab = snapshot.data.tabs.find((item) => item.id === args.tabId);
      if (tab) tab.muted = Boolean(args.muted);
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "restore_closed_tab": {
      const requestedId = args.tabId as string | null | undefined;
      const index = requestedId
        ? snapshot.data.recentlyClosed.findIndex(
            (item) => item.id === requestedId,
          )
        : 0;
      const restored =
        index >= 0
          ? snapshot.data.recentlyClosed.splice(index, 1)[0]
          : undefined;
      if (!restored) throw new Error("没有可恢复的标签页");
      restored.loading = false;
      restored.loaded = true;
      restored.muted = false;
      snapshot.data.tabs.push(restored);
      snapshot.data.tabs.sort(
        (left, right) => Number(right.pinned) - Number(left.pinned),
      );
      snapshot.data.activeTabId = restored.id;
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
        snapshot.data.tabs.push(makeTab());
      }
      if (
        !snapshot.data.tabs.some((tab) => tab.id === snapshot.data.activeTabId)
      ) {
        snapshot.data.activeTabId =
          snapshot.data.tabs[snapshot.data.tabs.length - 1].id;
      }
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "create_workspace": {
      const name = (args.name as string).trim();
      if (!name) throw new Error("工作区名称不能为空");
      stashCurrentTabs();
      const workspace = {
        id: `ws-${Math.random().toString(36).slice(2, 8)}`,
        name,
        tabs: [makeTab()],
        activeTabId: null as string | null,
      };
      workspace.activeTabId = workspace.tabs[0].id;
      snapshot.data.workspaces.push(workspace);
      snapshot.data.tabs = structuredClone(workspace.tabs);
      snapshot.data.activeTabId = workspace.activeTabId;
      snapshot.data.activeWorkspaceId = workspace.id;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "rename_workspace": {
      const workspace = snapshot.data.workspaces.find(
        (item) => item.id === (args.workspaceId as string),
      );
      if (!workspace) throw new Error("工作区不存在");
      const name = (args.name as string).trim();
      if (!name) throw new Error("工作区名称不能为空");
      workspace.name = name;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "remove_workspace": {
      const workspaceId = args.workspaceId as string;
      if (snapshot.data.workspaces.length <= 1)
        throw new Error("至少保留一个工作区");
      if (workspaceId === snapshot.data.activeWorkspaceId)
        throw new Error("不能删除当前工作区");
      snapshot.data.workspaces = snapshot.data.workspaces.filter(
        (item) => item.id !== workspaceId,
      );
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "switch_workspace": {
      const workspaceId = args.workspaceId as string;
      if (workspaceId === snapshot.data.activeWorkspaceId)
        return structuredClone(snapshot);
      stashCurrentTabs();
      loadWorkspaceTabs(workspaceId);
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "move_tab_to_workspace": {
      const tabId = args.tabId as string;
      const workspaceId = args.workspaceId as string;
      if (workspaceId === snapshot.data.activeWorkspaceId)
        throw new Error("标签已在当前工作区");
      const target = snapshot.data.workspaces.find(
        (item) => item.id === workspaceId,
      );
      if (!target) throw new Error("工作区不存在");
      const index = snapshot.data.tabs.findIndex((tab) => tab.id === tabId);
      if (index >= 0) {
        const [moved] = snapshot.data.tabs.splice(index, 1);
        moved.hibernated = false;
        moved.loaded = false;
        target.tabs.push(moved);
      }
      if (
        !snapshot.data.tabs.some((tab) => tab.id === snapshot.data.activeTabId)
      ) {
        snapshot.data.activeTabId =
          snapshot.data.tabs[Math.max(0, index - 1)]?.id ??
          snapshot.data.tabs[0]?.id ??
          null;
      }
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "apply_tab_batch": {
      const update = args.update as {
        action: string;
        tabIds: string[];
        workspaceId?: string | null;
      };
      const ids = new Set(update.tabIds);
      if (update.action === "close") {
        const closed = snapshot.data.tabs.filter((tab) => ids.has(tab.id));
        snapshot.data.tabs = snapshot.data.tabs.filter(
          (tab) => !ids.has(tab.id),
        );
        snapshot.data.recentlyClosed = [
          ...closed,
          ...snapshot.data.recentlyClosed,
        ].slice(0, 20);
        if (snapshot.data.tabs.length === 0)
          snapshot.data.tabs.push(makeTab());
        if (
          !snapshot.data.tabs.some(
            (tab) => tab.id === snapshot.data.activeTabId,
          )
        ) {
          snapshot.data.activeTabId = snapshot.data.tabs[0].id;
        }
      } else if (update.action === "bookmark") {
        for (const tab of snapshot.data.tabs.filter(
          (tab) => ids.has(tab.id) && /^https?:\/\//.test(tab.url),
        )) {
          if (snapshot.data.bookmarks.some((item) => item.url === tab.url))
            continue;
          snapshot.data.bookmarks.unshift({
            id: `bm-${Math.random().toString(36).slice(2, 8)}`,
            title: tab.title || tab.url,
            url: tab.url,
            createdAt: new Date().toISOString(),
          });
        }
      } else if (update.action === "mute" || update.action === "unmute") {
        for (const tab of snapshot.data.tabs) {
          if (ids.has(tab.id)) tab.muted = update.action === "mute";
        }
      } else if (update.action === "move") {
        const target = snapshot.data.workspaces.find(
          (item) => item.id === update.workspaceId,
        );
        if (!target) throw new Error("工作区不存在");
        const moved = snapshot.data.tabs.filter((tab) => ids.has(tab.id));
        snapshot.data.tabs = snapshot.data.tabs.filter(
          (tab) => !ids.has(tab.id),
        );
        for (const tab of moved) {
          tab.hibernated = false;
          tab.loaded = false;
          target.tabs.push(tab);
        }
        if (snapshot.data.tabs.length === 0)
          snapshot.data.tabs.push(makeTab());
        if (
          !snapshot.data.tabs.some(
            (tab) => tab.id === snapshot.data.activeTabId,
          )
        ) {
          snapshot.data.activeTabId = snapshot.data.tabs[0].id;
        }
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
    case "exit_app":
    case "open_download":
    case "clear_site_data":
      return null;
    case "hide_to_tray": {
      snapshot.windowVisible = false;
      emitSnapshot();
      return null;
    }
    case "show_shell": {
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "set_global_shortcut": {
      snapshot.data.settings.shortcut = args.shortcut as string;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "update_settings": {
      Object.assign(
        snapshot.data.settings,
        args.update as Record<string, unknown>,
      );
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
      snapshot.data.bookmarks = snapshot.data.bookmarks.filter(
        (item) => item.id !== args.bookmarkId,
      );
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
      snapshot.hasPassword = true;
      snapshot.locked = false;
      snapshot.firstRun = false;
      snapshot.data.settings.passwordHash = null;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "disable_app_password": {
      if (args.currentPassword !== mockPassword) throw new Error("密码错误");
      mockPassword = null;
      snapshot.hasPassword = false;
      snapshot.data.settings.passwordHash = null;
      emitSnapshot();
      return structuredClone(snapshot);
    }
    case "unlock_app": {
      if ((args.password as string) !== mockPassword)
        throw new Error("应用密码错误");
      snapshot.locked = false;
      snapshot.firstRun = false;
      snapshot.hasPassword = true;
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
      invoke: (
        command: string,
        args?: Record<string, unknown>,
      ) => Promise<unknown>;
      transformCallback: (
        callback: (response: unknown) => void,
        once?: boolean,
      ) => number;
      metadata?: { currentWindow?: { label: string } };
    };
    __mock: {
      emit: typeof emitSnapshot;
      patch: (mutate: (draft: AppSnapshot) => void) => void;
      error: (message: string) => void;
      theme: (value: "light" | "dark") => void;
      openSection: (section: string) => void;
      focusAddress: () => void;
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
  const initialSection = new URLSearchParams(window.location.search).get(
    "section",
  );
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
    focusAddress: () => emitEvent("focus-address", null),
    theme: (value) => {
      localStorage.setItem("quickpane.theme", value);
      document.documentElement.classList.toggle("dark", value === "dark");
    },
    snapshot: () => structuredClone(snapshot),
  };
}
