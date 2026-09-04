import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import test from "node:test";
import ts from "typescript";

const source = await readFile(new URL("../src/lib/address-suggestions.ts", import.meta.url), "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2020 },
}).outputText;
const tempDir = await mkdtemp(join(dirname(fileURLToPath(import.meta.url)), ".address-suggestions-"));
const modulePath = join(tempDir, "address-suggestions.mjs");
await writeFile(modulePath, compiled);
const { getAddressSuggestions, getSourceSuggestions } = await import(pathToFileURL(modulePath).href);
await rm(tempDir, { recursive: true, force: true });

const now = "2026-08-31T12:00:00.000Z";

test("优先保留快捷站点或书签并按历史访问时间排序", () => {
  const suggestions = getAddressSuggestions({
    query: "example",
    quickLinks: [{ id: "q1", title: "Example 快捷站点", url: "https://example.com" }],
    bookmarks: [{ id: "b1", title: "Example 书签", url: "https://example.com/", createdAt: now }],
    history: [
      { id: "h1", title: "Example 较早记录", url: "https://old.example.net", visitedAt: "2026-08-30T12:00:00.000Z" },
      { id: "h2", title: "Example 最近记录", url: "https://new.example.net", visitedAt: now },
      { id: "h3", title: "Example 重复记录", url: "https://example.com/", visitedAt: now },
    ],
  });

  assert.equal(suggestions.filter((item) => item.url.includes("example.com")).length, 1);
  assert.equal(suggestions[0].source, "quickLink");
  assert.deepEqual(suggestions.slice(1).map((item) => item.url), ["https://new.example.net", "https://old.example.net"]);
});

test("当前打开标签优先并返回标签 id", () => {
  const suggestions = getAddressSuggestions({
    query: "docs",
    tabs: [
      { id: "tab-1", title: "开发文档", url: "https://docs.example", pinned: true, lastActiveAt: now },
      { id: "tab-2", title: "其他页面", url: "https://other.example", pinned: false, lastActiveAt: now },
    ],
    quickLinks: [],
    bookmarks: [{ id: "b1", title: "开发文档书签", url: "https://docs.example/", createdAt: now }],
    history: [],
  });

  assert.deepEqual(suggestions[0], {
    title: "开发文档",
    url: "https://docs.example",
    host: "docs.example",
    source: "tab",
    tabId: "tab-1",
  });
});

test("支持标题、主机名和网址子串匹配，并优先展示书签", () => {
  const suggestions = getAddressSuggestions({
    query: "docs",
    quickLinks: [],
    bookmarks: [{ id: "b1", title: "开发文档", url: "https://example.com/guide/docs", createdAt: now }],
    history: [{ id: "h1", title: "其他页面", url: "https://docs.example.net", visitedAt: now }],
    limit: 2,
  });

  assert.deepEqual(suggestions.map((item) => item.url), [
    "https://example.com/guide/docs",
    "https://docs.example.net",
  ]);
});

test("支持拼音全拼检索中文标题", () => {
  const suggestions = getAddressSuggestions({
    query: "bianyi",
    quickLinks: [{ id: "q1", title: "便宜", url: "https://cheap.example" }],
    bookmarks: [],
    history: [],
  });

  assert.deepEqual(suggestions.map((item) => item.url), ["https://cheap.example"]);
});

test("拼音查询忽略大小写和全角分隔空格", () => {
  const suggestions = getAddressSuggestions({
    query: "BIAN　YI",
    quickLinks: [{ id: "q1", title: "便宜", url: "https://cheap.example" }],
    bookmarks: [],
    history: [],
  });

  assert.deepEqual(suggestions.map((item) => item.url), ["https://cheap.example"]);
});

test("支持拼音首字母检索中文标题", () => {
  const suggestions = getAddressSuggestions({
    query: "py",
    quickLinks: [
      { id: "q1", title: "拼音", url: "https://pinyin.example" },
      { id: "q2", title: "便宜", url: "https://cheap.example" },
    ],
    bookmarks: [],
    history: [],
  });

  assert.deepEqual(suggestions.map((item) => item.url), [
    "https://pinyin.example",
    "https://cheap.example",
  ]);
});

test("原生标题命中优先于拼音命中", () => {
  const suggestions = getAddressSuggestions({
    query: "py",
    quickLinks: [
      { id: "q1", title: "便宜", url: "https://cheap.example" },
      { id: "q2", title: "Python 文档", url: "https://python.example" },
    ],
    bookmarks: [],
    history: [],
  });

  assert.deepEqual(suggestions.map((item) => item.url), [
    "https://python.example",
    "https://cheap.example",
  ]);
});

test("拼音可以匹配中英文混合标题和多音词", () => {
  const suggestions = getAddressSuggestions({
    query: "yinhang",
    quickLinks: [{ id: "q1", title: "银行 GitHub", url: "https://bank.example" }],
    bookmarks: [],
    history: [],
  });

  assert.deepEqual(suggestions.map((item) => item.url), ["https://bank.example"]);
});

test("拼音匹配仍沿用规范化网址去重", () => {
  const suggestions = getAddressSuggestions({
    query: "py",
    quickLinks: [{ id: "q1", title: "便宜入口", url: "https://cheap.example" }],
    bookmarks: [{ id: "b1", title: "便宜书签", url: "https://cheap.example/", createdAt: now }],
    history: [{ id: "h1", title: "便宜历史", url: "https://cheap.example/", visitedAt: now }],
  });

  assert.deepEqual(suggestions.map((item) => item.url), ["https://cheap.example"]);
});

test("空白查询不返回建议", () => {
  assert.deepEqual(getAddressSuggestions({ query: "  ", quickLinks: [], bookmarks: [], history: [] }), []);
});

test("无效历史时间不会破坏最近访问排序", () => {
  const suggestions = getAddressSuggestions({
    query: "example",
    quickLinks: [],
    bookmarks: [],
    history: [
      { id: "bad", title: "时间损坏", url: "https://invalid.example", visitedAt: "not-a-date" },
      { id: "good", title: "最近页面", url: "https://recent.example", visitedAt: now },
    ],
  });

  assert.deepEqual(suggestions.map((item) => item.url), [
    "https://recent.example",
    "https://invalid.example",
  ]);
});

test("同一来源重复网址保留匹配等级更高的记录", () => {
  const suggestions = getAddressSuggestions({
    query: "example",
    quickLinks: [
      { id: "q1", title: "访问 example 文档", url: "https://same.example" },
      { id: "q2", title: "example", url: "https://same.example/" },
    ],
    bookmarks: [],
    history: [],
  });

  assert.deepEqual(suggestions, [{
    title: "example",
    url: "https://same.example/",
    host: "same.example",
    source: "quickLink",
  }]);
});

test("坏记录不会阻断建议，并使用主机名降级标题", () => {
  const suggestions = getAddressSuggestions({
    query: "example",
    quickLinks: [],
    bookmarks: [],
    history: [
      { id: "bad-url", title: "坏地址", url: "not-a-url", visitedAt: now },
      { id: "missing-title", url: "https://fallback.example/docs", visitedAt: now },
    ],
  });

  assert.deepEqual(suggestions, [{
    title: "fallback.example",
    url: "https://fallback.example/docs",
    host: "fallback.example",
    source: "history",
  }]);
});

test("建议结果最多返回八条", () => {
  const suggestions = getAddressSuggestions({
    query: "site",
    quickLinks: Array.from({ length: 10 }, (_, index) => ({
      id: `q${index}`,
      title: `Site ${index + 1}`,
      url: `https://site-${index + 1}.example`,
    })),
    bookmarks: [],
    history: [],
    limit: 50,
  });

  assert.equal(suggestions.length, 8);
  assert.equal(suggestions[0].url, "https://site-1.example");
  assert.equal(suggestions[7].url, "https://site-8.example");
});

test("单源建议：空词列出全部标签且同 URL 多标签不去重", () => {
  const suggestions = getSourceSuggestions({
    source: "tab",
    query: "",
    tabs: [
      { id: "t1", title: "文档", url: "https://docs.example", lastActiveAt: "2026-08-30T12:00:00.000Z" },
      { id: "t2", title: "文档副本", url: "https://docs.example", lastActiveAt: now },
    ],
  });

  assert.deepEqual(suggestions.map((item) => item.tabId), ["t2", "t1"]);
  assert.equal(suggestions.every((item) => item.source === "tab"), true);
});

test("单源建议：历史按访问时间降序且 URL 去重，上限 20", () => {
  const suggestions = getSourceSuggestions({
    source: "history",
    query: "",
    history: [
      { id: "h1", title: "旧记录", url: "https://same.example", visitedAt: "2026-08-01T00:00:00.000Z" },
      { id: "h2", title: "新记录", url: "https://same.example/", visitedAt: now },
      ...Array.from({ length: 20 }, (_, index) => ({
        id: `x${index}`,
        title: `条目 ${index}`,
        url: `https://entry-${index}.example`,
        visitedAt: `2026-08-2${index % 10}T00:00:00.000Z`,
      })),
    ],
  });

  assert.equal(suggestions.length, 20);
  assert.equal(suggestions[0].title, "新记录");
  assert.ok(!suggestions.some((item) => item.title === "旧记录"));
});

test("单源建议：按关键词过滤并支持拼音", () => {
  const suggestions = getSourceSuggestions({
    source: "bookmark",
    query: "kaifa",
    bookmarks: [
      { id: "b1", title: "开发文档", url: "https://dev.example", createdAt: now },
      { id: "b2", title: "购物清单", url: "https://shop.example", createdAt: now },
    ],
  });

  assert.deepEqual(suggestions.map((item) => item.url), ["https://dev.example"]);
});

test("单源建议：newtab 标签不被协议过滤", () => {
  const suggestions = getSourceSuggestions({
    source: "tab",
    query: "",
    tabs: [{ id: "t1", title: "新标签页", url: "quickpane://newtab", lastActiveAt: now }],
  });

  assert.equal(suggestions.length, 1);
  assert.equal(suggestions[0].tabId, "t1");
});
