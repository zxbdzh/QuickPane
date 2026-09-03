import { match as matchPinyin } from "pinyin-pro";
import type { Bookmark, HistoryEntry, QuickLink, TabRecord } from "../types";

export type AddressSuggestionSource = "tab" | "quickLink" | "bookmark" | "history";

export type AddressSuggestion = {
  title: string;
  url: string;
  host: string;
  source: AddressSuggestionSource;
  tabId?: string;
};

type SuggestionCandidate = AddressSuggestion & {
  key: string;
  matchRank: number;
  sourceRank: number;
  visitedAt: number;
  order: number;
};

function parseSuggestionUrl(value: unknown) {
  if (typeof value !== "string") return null;
  const trimmed = value.trim();
  if (!trimmed) return null;

  try {
    const parsed = new URL(trimmed);
    if (!["http:", "https:", "chrome-extension:"].includes(parsed.protocol)) return null;
    return { value: trimmed, host: parsed.hostname, key: parsed.href };
  } catch {
    return null;
  }
}

function visitedTimestamp(value: string) {
  const timestamp = Date.parse(value);
  return Number.isFinite(timestamp) ? timestamp : Number.MIN_SAFE_INTEGER;
}

function matchRank(query: string, title: string, url: string, host: string) {
  const values = [title, host, url].map((value) => value.toLowerCase());
  if (values.some((value) => value === query)) return 0;
  if (values.some((value) => value.startsWith(query))) return 1;
  if (values.some((value) => value.includes(query))) return 2;
  return null;
}

const HAN_CHARACTERS = /[\u3400-\u9fff]/;
const ASCII_PINYIN_QUERY = /^[a-z]+$/i;

function pinyinMatchRank(query: string, title: string, cache: Map<string, number | null>) {
  const normalizedQuery = query.replace(/\s/gu, "").toLowerCase();
  if (!ASCII_PINYIN_QUERY.test(normalizedQuery) || !HAN_CHARACTERS.test(title)) return null;
  if (cache.has(title)) return cache.get(title) ?? null;

  const matches = matchPinyin(title, normalizedQuery);
  // 原生字段占用 0-2 级，拼音命中统一放到其后，避免改变既有排序契约。
  const rank = matches ? (matches[0] === 0 ? 3 : 4) : null;
  cache.set(title, rank);
  return rank;
}

export function getAddressSuggestions({
  query,
  quickLinks,
  bookmarks,
  history,
  tabs = [],
  limit = 8,
}: {
  query: string;
  quickLinks: QuickLink[];
  bookmarks: Bookmark[];
  history: HistoryEntry[];
  tabs?: TabRecord[];
  limit?: number;
}): AddressSuggestion[] {
  const value = query.trim().toLowerCase();
  if (!value) return [];

  const candidates = [
    ...tabs.map((item, index) => ({
      title: item.title,
      url: item.url,
      tabId: item.id,
      source: "tab" as const,
      sourceRank: item.pinned ? -2 : -1,
      visitedAt: visitedTimestamp(item.lastActiveAt),
      order: index,
    })),
    ...quickLinks.map((item, index) => ({ ...item, tabId: undefined, source: "quickLink" as const, sourceRank: 0, visitedAt: 0, order: tabs.length + index })),
    ...bookmarks.map((item, index) => ({ ...item, tabId: undefined, source: "bookmark" as const, sourceRank: 0, visitedAt: 0, order: tabs.length + quickLinks.length + index })),
    ...history
      .slice()
      .sort((left, right) => visitedTimestamp(right.visitedAt) - visitedTimestamp(left.visitedAt))
      .map((item, index) => ({
        ...item,
        tabId: undefined,
        source: "history" as const,
        sourceRank: 1,
        visitedAt: visitedTimestamp(item.visitedAt),
        order: tabs.length + quickLinks.length + bookmarks.length + index,
      })),
  ];
  const matching: SuggestionCandidate[] = [];
  const pinyinCache = new Map<string, number | null>();

  for (const candidate of candidates) {
    const parsed = parseSuggestionUrl(candidate.url);
    if (!parsed) continue;
    const title = typeof candidate.title === "string" ? candidate.title.trim() : "";
    const rank = matchRank(value, title, parsed.value, parsed.host) ?? pinyinMatchRank(value, title, pinyinCache);
    if (rank === null) continue;
    matching.push({
      key: parsed.key,
      title: title || parsed.host,
      url: parsed.value,
      host: parsed.host,
      source: candidate.source,
      tabId: candidate.tabId,
      sourceRank: candidate.sourceRank,
      matchRank: rank,
      visitedAt: candidate.visitedAt,
      order: candidate.order,
    });
  }

  const unique = new Map<string, SuggestionCandidate>();
  for (const candidate of matching.sort((left, right) =>
    left.sourceRank - right.sourceRank
    || left.matchRank - right.matchRank
    || right.visitedAt - left.visitedAt
    || left.order - right.order)) {
    if (!unique.has(candidate.key)) unique.set(candidate.key, candidate);
  }

  return [...unique.values()]
    .slice(0, Math.max(0, Math.min(8, limit)))
    .map(({ title, url, host, source, tabId }) => ({
      title,
      url,
      host,
      source,
      ...(tabId === undefined ? {} : { tabId }),
    }));
}
