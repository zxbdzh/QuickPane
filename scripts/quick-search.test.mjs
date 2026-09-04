import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import test from "node:test";
import ts from "typescript";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const compile = async (sourcePath, outputPath) => {
  const source = await readFile(new URL(sourcePath, import.meta.url), "utf8");
  const compiled = ts.transpileModule(source, {
    compilerOptions: {
      module: ts.ModuleKind.ESNext,
      target: ts.ScriptTarget.ES2020,
    },
  }).outputText;
  await writeFile(outputPath, compiled);
};

const tempDir = await mkdtemp(join(scriptDir, ".quick-search-"));
try {
  // quick-search 依赖同目录的 text-search：补齐 .mjs 扩展名并一起转译。
  await compile("../src/lib/text-search.ts", join(tempDir, "text-search.mjs"));
  const quickSource = await readFile(
    new URL("../src/lib/quick-search.ts", import.meta.url),
    "utf8",
  );
  const quickCompiled = ts.transpileModule(quickSource, {
    compilerOptions: {
      module: ts.ModuleKind.ESNext,
      target: ts.ScriptTarget.ES2020,
    },
  }).outputText.replace(/from "\.\/text-search"/u, 'from "./text-search.mjs"');
  await writeFile(join(tempDir, "quick-search.mjs"), quickCompiled);
  var { quickSearch, parseSearchKeyword } = await import(
    pathToFileURL(join(tempDir, "quick-search.mjs")).href
  );
} finally {
  await rm(tempDir, { recursive: true, force: true });
}

const now = "2026-09-01T12:00:00.000Z";
const tab = (id, title, url, extra = {}) => ({
  id,
  title,
  url,
  pinned: false,
  loading: false,
  loaded: true,
  muted: false,
  hibernated: false,
  createdAt: now,
  lastActiveAt: now,
  ...extra,
});
const source = {
  tabs: [
    tab("t1", "GitHub", "https://github.com", { lastActiveAt: "2026-09-01T11:00:00.000Z" }),
    tab("t2", "开发文档", "https://docs.example", { lastActiveAt: "2026-09-01T11:30:00.000Z" }),
  ],
  recentlyClosed: [tab("c1", "关闭的购物车", "https://shop.example")],
  workspaces: [{ id: "w1", name: "学习", tabs: [], activeTabId: null }],
  bookmarks: [{ id: "b1", title: "MDN", url: "https://developer.mozilla.org", createdAt: now }],
  history: [{ id: "h1", title: "新闻", url: "https://news.example", visitedAt: now }],
};

test("空查询只出最近上下文：标签、最近关闭、其它工作区", () => {
  const groups = quickSearch({ query: "", ...source });
  assert.deepEqual(groups.map((group) => group.key), ["tab", "closed", "workspace"]);
  assert.deepEqual(groups[0].items.map((item) => item.tabId), ["t2", "t1"]);
});

test("有查询时五组全搜并按拼音匹配", () => {
  const groups = quickSearch({ query: "kaifa", ...source });
  assert.deepEqual(groups.map((group) => group.key), [
    "tab",
    "closed",
    "workspace",
    "bookmark",
    "history",
  ]);
  assert.deepEqual(groups[0].items.map((item) => item.tabId), ["t2"]);
});

test("标签按最近活跃降序，休眠标志随行", () => {
  const groups = quickSearch({
    query: "",
    ...source,
    tabs: [
      tab("t1", "旧标签", "https://old.example", {
        hibernated: true,
        lastActiveAt: "2026-09-01T10:00:00.000Z",
      }),
      tab("t2", "新标签", "https://new.example", {
        lastActiveAt: "2026-09-01T11:59:00.000Z",
      }),
    ],
  });
  const items = groups[0].items;
  assert.equal(items[0].tabId, "t2");
  assert.equal(items[1].tabId, "t1");
  assert.equal(items[1].hibernated, true);
  assert.equal(items[0].hibernated, false);
});

test("最近关闭限额：空查询 5 条，有查询 8 条", () => {
  const closed = Array.from({ length: 10 }, (_, index) =>
    tab(`c${index}`, `页面 ${index}`, `https://closed-${index}.example`),
  );
  const idle = quickSearch({ query: "", ...source, recentlyClosed: closed });
  const active = quickSearch({ query: "页面", ...source, recentlyClosed: closed });
  assert.equal(idle.find((group) => group.key === "closed").items.length, 5);
  assert.equal(active.find((group) => group.key === "closed").items.length, 8);
});

test("历史按访问时间降序且 URL 去重，限额 8", () => {
  const history = [
    { id: "h1", title: "较旧", url: "https://same.example", visitedAt: "2026-08-01T00:00:00.000Z" },
    { id: "h2", title: "较新", url: "https://same.example/", visitedAt: "2026-09-01T00:00:00.000Z" },
    ...Array.from({ length: 9 }, (_, index) => ({
      id: `x${index}`,
      title: `条目 ${index}`,
      url: `https://entry-${index}.example`,
      visitedAt: `2026-08-2${index}T00:00:00.000Z`,
    })),
  ];
  const groups = quickSearch({ query: "", filter: "history", ...source, history });
  const items = groups[0].items;
  assert.equal(items.length, 8);
  assert.equal(items[0].title, "较新");
  assert.ok(!items.some((item) => item.title === "较旧"));
});

test("filter 指定组时空词列出该源全部", () => {
  const groups = quickSearch({ query: "", filter: "bookmark", ...source });
  assert.deepEqual(groups.map((group) => group.key), ["bookmark"]);
  assert.deepEqual(groups[0].items.map((item) => item.url), [
    "https://developer.mozilla.org",
  ]);
});

test("书签限额 6 条", () => {
  const bookmarks = Array.from({ length: 10 }, (_, index) => ({
    id: `b${index}`,
    title: `书签 ${index}`,
    url: `https://bookmark-${index}.example`,
    createdAt: now,
  }));
  const groups = quickSearch({ query: "书签", ...source, bookmarks });
  assert.equal(groups.find((group) => group.key === "bookmark").items.length, 6);
});

test("工作区按名称匹配", () => {
  const hit = quickSearch({ query: "学习", ...source });
  const miss = quickSearch({ query: "工作", ...source });
  assert.equal(hit.find((group) => group.key === "workspace").items.length, 1);
  assert.equal(miss.find((group) => group.key === "workspace").items.length, 0);
});

test("parseSearchKeyword 识别关键字与边界", () => {
  assert.deepEqual(parseSearchKeyword("t "), { source: "tab", term: "" });
  assert.deepEqual(parseSearchKeyword("t:"), { source: "tab", term: "" });
  assert.deepEqual(parseSearchKeyword("T github"), { source: "tab", term: "github" });
  assert.deepEqual(parseSearchKeyword("b：MDN"), { source: "bookmark", term: "MDN" });
  assert.deepEqual(parseSearchKeyword("h  新闻 "), { source: "history", term: "新闻" });
  assert.deepEqual(parseSearchKeyword("t"), { source: null, term: "t" });
  assert.deepEqual(parseSearchKeyword("typescript"), { source: null, term: "typescript" });
  assert.deepEqual(parseSearchKeyword("  "), { source: null, term: "" });
});
