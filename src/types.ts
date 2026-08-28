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

export type Settings = {
  shortcut: string | null;
  autostart: boolean;
  homeUrl: string;
  searchTemplate: string;
  historyDays: number;
  passwordHash: string | null;
  lockOnSystemLock: boolean;
  quickLinks: QuickLink[];
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
  windowVisible: boolean;
};

export type ShellSection = "newtab" | "history" | "bookmarks" | "downloads" | "settings" | "lock";
