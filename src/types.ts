export type TabRecord = {
  id: string;
  title: string;
  url: string;
  pinned: boolean;
  loading: boolean;
  loaded: boolean;
  muted: boolean;
  createdAt: string;
  lastActiveAt: string;
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
  /** 声明了 default_popup 时的面板页 URL（在标签页中打开，WebView2 无弹出宿主） */
  popupUrl: string | null;
};

export type Settings = {
  shortcut: string | null;
  tabSearchShortcut: string;
  recentlyClosedShortcut: string;
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
};

export type PersistedData = {
  tabs: TabRecord[];
  activeTabId: string | null;
  recentlyClosed: TabRecord[];
  history: HistoryEntry[];
  bookmarks: Bookmark[];
  downloads: DownloadRecord[];
  settings: Settings;
};

export type AppSnapshot = {
  data: PersistedData;
  locked: boolean;
  firstRun: boolean;
  hasPassword: boolean;
  windowVisible: boolean;
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
  | "lock";
