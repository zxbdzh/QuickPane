export type TabRecord = {
  id: string;
  title: string;
  url: string;
  pinned: boolean;
  loading: boolean;
  loaded: boolean;
  muted: boolean;
  /** 休眠中：后台 WebView 已释放，激活时按 url 重建 */
  hibernated: boolean;
  createdAt: string;
  lastActiveAt: string;
};

export type Workspace = {
  id: string;
  name: string;
  tabs: TabRecord[];
  activeTabId: string | null;
};

export type SessionSnapshotTab = {
  url: string;
  title: string;
  pinned: boolean;
};

export type SessionSnapshot = {
  id: string;
  name: string;
  createdAt: string;
  tabs: SessionSnapshotTab[];
  activeIndex: number;
};

export type HistoryEntry = {
  id: string;
  title: string;
  url: string;
  visitedAt: string;
};

export type Bookmark = {
  id: string;
  title: string;
  url: string;
  createdAt: string;
};

export type DownloadRecord = {
  id: string;
  fileName: string;
  url: string;
  path: string | null;
  state: "downloading" | "completed" | "failed";
  startedAt: string;
  finishedAt: string | null;
};

export type QuickLink = {
  id: string;
  title: string;
  url: string;
};

export type ProxyMode = "system" | "direct" | "custom";

export type BrowserExtension = {
  id: string;
  name: string;
  version: string;
  description: string;
  enabled: boolean;
  icon: string | null;
  /** 运行时确认的扩展页面，打开时由后端按 id 再校验。 */
  popupUrl: string | null;
  optionsUrl: string | null;
  builtin: boolean;
  loaded: boolean;
  loadError: string | null;
};

export type Settings = {
  shortcut: string | null;
  paletteShortcut: string;
  autostart: boolean;
  homeUrl: string;
  searchTemplate: string;
  historyDays: number;
  passwordHash: string | null;
  lockOnSystemLock: boolean;
  autoLockAfterHideSeconds: number;
  quickLinks: QuickLink[];
  proxyMode: ProxyMode;
  proxyUrl: string;
  pinnedExtensions: string[];
  /** 标签休眠阈值（分钟）：0 关闭，仅暂停媒体 */
  tabHibernationMinutes: number;
};

export type PersistedData = {
  tabs: TabRecord[];
  activeTabId: string | null;
  recentlyClosed: TabRecord[];
  history: HistoryEntry[];
  bookmarks: Bookmark[];
  downloads: DownloadRecord[];
  settings: Settings;
  workspaces: Workspace[];
  activeWorkspaceId: string | null;
  sessionSnapshots: SessionSnapshot[];
};

export type AppSnapshot = {
  data: PersistedData;
  locked: boolean;
  firstRun: boolean;
  hasPassword: boolean;
  windowVisible: boolean;
  windowAlwaysOnTop: boolean;
  pinnedExtensions: BrowserExtension[];
  recoveryMessage: string | null;
};

export type ShellSection =
  | "newtab"
  | "history"
  | "bookmarks"
  | "downloads"
  | "extensions"
  | "settings"
  | "tabs"
  | "lock";
