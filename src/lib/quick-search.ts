import type { Bookmark, HistoryEntry, TabRecord, Workspace } from "../types";
import { matchesTextQuery } from "./text-search";

export type QuickSearchGroupKey =
  | "tab"
  | "closed"
  | "workspace"
  | "bookmark"
  | "history";

export type QuickSearchItem = {
  key: string;
  group: QuickSearchGroupKey;
  title: string;
  url?: string;
  tabId?: string;
  workspaceId?: string;
  hibernated?: boolean;
};

export type QuickSearchGroup = {
  key: QuickSearchGroupKey;
  items: QuickSearchItem[];
};

export type QuickSearchSource =
  | "tab"
  | "bookmark"
  | "history"
  | null;

const GROUP_ORDER: QuickSearchGroupKey[] = [
  "tab",
  "closed",
  "workspace",
  "bookmark",
  "history",
];

const CLOSED_LIMIT = 8;
const CLOSED_RECENT_LIMIT = 5;
const BOOKMARK_LIMIT = 6;
const HISTORY_LIMIT = 8;

/** 地址栏动作关键字 → 检索源（t 标签 / b 书签 / h 历史）。 */
const KEYWORD_SOURCES: Record<string, Exclude<QuickSearchSource, null>> = {
  t: "tab",
  b: "bookmark",
  h: "history",
};

const KEYWORD_SEPARATORS = /^[\s:：]+/;

function timestamp(value: string) {
  const time = Date.parse(value);
  return Number.isFinite(time) ? time : Number.MIN_SAFE_INTEGER;
}

function normalizedUrlKey(url: string) {
  const trimmed = url.trim();
  if (!trimmed) return trimmed;
  try {
    return new URL(trimmed).href;
  } catch {
    return trimmed;
  }
}

/**
 * 识别地址栏动作关键字（大小写不敏感，支持空格 / 英文冒号 / 中文冒号分隔）。
 * 单独的 "t"/"b"/"h"（无分隔符）视为普通搜索词，避免误伤单字母查询。
 */
export function parseSearchKeyword(value: string): {
  source: QuickSearchSource;
  term: string;
} {
  const raw = value.trimStart();
  const first = raw[0]?.toLowerCase();
  const source = first ? (KEYWORD_SOURCES[first] ?? null) : null;
  if (!source) return { source: null, term: value.trim() };

  const rest = raw.slice(1);
  if (!rest || !KEYWORD_SEPARATORS.test(rest)) {
    return { source: null, term: value.trim() };
  }
  return { source, term: rest.replace(KEYWORD_SEPARATORS, "").trim() };
}

/**
 * 统一快速切换面板的检索：
 * - 空查询（无过滤）：只出「最近上下文」——标签（按最近活跃降序）、最近关闭（≤5）、其它工作区；
 * - 有查询：五组全搜（子串 + 拼音），history 按访问时间降序且 URL 去重；
 * - filter 指定组时仅返回该组（空词列出该源全部）。
 */
export function quickSearch({
  query,
  filter = null,
  tabs,
  recentlyClosed,
  workspaces,
  bookmarks,
  history,
}: {
  query: string;
  filter?: QuickSearchGroupKey | null;
  tabs: TabRecord[];
  recentlyClosed: TabRecord[];
  workspaces: Workspace[];
  bookmarks: Bookmark[];
  history: HistoryEntry[];
}): QuickSearchGroup[] {
  const term = query.trim();
  const emptyQuery = !term;
  const activeGroups = emptyQuery && !filter
    ? (["tab", "closed", "workspace"] as const)
    : GROUP_ORDER.filter((key) => !filter || key === filter);

  const groups: QuickSearchGroup[] = [];
  for (const key of activeGroups) {
    if (key === "tab") {
      const items = tabs
        .filter((tab) => matchesTextQuery(term, tab.title, tab.url))
        .sort((left, right) => timestamp(right.lastActiveAt) - timestamp(left.lastActiveAt))
        .map((tab) => ({
          key: `tab:${tab.id}`,
          group: key,
          title: tab.title || "新标签页",
          url: tab.url,
          tabId: tab.id,
          hibernated: tab.hibernated,
        }));
      groups.push({ key, items });
      continue;
    }

    if (key === "closed") {
      const limit = emptyQuery ? CLOSED_RECENT_LIMIT : CLOSED_LIMIT;
      const items = recentlyClosed
        .filter((tab) => matchesTextQuery(term, tab.title, tab.url))
        .slice(0, limit)
        .map((tab) => ({
          key: `closed:${tab.id}`,
          group: key,
          title: tab.title || "新标签页",
          url: tab.url,
          tabId: tab.id,
        }));
      groups.push({ key, items });
      continue;
    }

    if (key === "workspace") {
      const items = workspaces
        .filter((workspace) => matchesTextQuery(term, workspace.name))
        .map((workspace) => ({
          key: `workspace:${workspace.id}`,
          group: key,
          title: workspace.name,
          workspaceId: workspace.id,
        }));
      groups.push({ key, items });
      continue;
    }

    if (key === "bookmark") {
      const items = bookmarks
        .filter((bookmark) => matchesTextQuery(term, bookmark.title, bookmark.url))
        .slice(0, BOOKMARK_LIMIT)
        .map((bookmark) => ({
          key: `bookmark:${bookmark.id}`,
          group: key,
          title: bookmark.title || bookmark.url,
          url: bookmark.url,
        }));
      groups.push({ key, items });
      continue;
    }

    const seen = new Set<string>();
    const items: QuickSearchItem[] = [];
    for (const entry of history
      .filter((item) => matchesTextQuery(term, item.title, item.url))
      .sort((left, right) => timestamp(right.visitedAt) - timestamp(left.visitedAt))) {
      const urlKey = normalizedUrlKey(entry.url);
      if (seen.has(urlKey)) continue;
      seen.add(urlKey);
      items.push({
        key: `history:${entry.id}`,
        group: key,
        title: entry.title || entry.url,
        url: entry.url,
      });
      if (items.length >= HISTORY_LIMIT) break;
    }
    groups.push({ key, items });
  }
  return groups;
}
