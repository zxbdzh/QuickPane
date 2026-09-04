import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import test from "node:test";
import ts from "typescript";

const source = await readFile(
  new URL("../src/lib/browser-shortcuts.ts", import.meta.url),
  "utf8",
);
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ESNext,
    target: ts.ScriptTarget.ES2020,
  },
}).outputText;
const tempDir = await mkdtemp(
  join(dirname(fileURLToPath(import.meta.url)), ".browser-shortcuts-"),
);
const modulePath = join(tempDir, "browser-shortcuts.mjs");
await writeFile(modulePath, compiled);
const { browserShortcutFromKey, findShortcutConflict } = await import(
  pathToFileURL(modulePath).href
);
await rm(tempDir, { recursive: true, force: true });

const key = (
  key,
  ctrlKey = false,
  shiftKey = false,
  altKey = false,
  metaKey = false,
) => ({
  key,
  ctrlKey,
  shiftKey,
  altKey,
  metaKey,
});

test("识别地址栏、标签恢复和缩放快捷键", () => {
  assert.equal(browserShortcutFromKey(key("l", true)), "focus-address");
  assert.equal(browserShortcutFromKey(key("T", true, true)), "restore-tab");
  assert.equal(browserShortcutFromKey(key("+", true)), "zoom-in");
});

test("识别可配置的标签面板快捷键并拒绝冲突", () => {
  assert.equal(
    browserShortcutFromKey(key("k", true, true), {
      tabSearch: "Ctrl+Shift+K",
      recentlyClosed: "Alt+Y",
    }),
    "tab-search",
  );
  assert.equal(
    browserShortcutFromKey(key("y", false, false, true), {
      tabSearch: "Ctrl+Shift+K",
      recentlyClosed: "Alt+Y",
    }),
    "recently-closed",
  );
  assert.match(
    findShortcutConflict({
      showHide: "Alt+Q",
      tabSearch: "Ctrl+Shift+K",
      recentlyClosed: "Ctrl+Shift+K",
    }),
    /相同快捷键/,
  );
  assert.match(
    findShortcutConflict({
      showHide: "Alt+Q",
      tabSearch: "Ctrl+H",
      recentlyClosed: "Ctrl+Shift+Y",
    }),
    /历史记录/,
  );
});

test("Escape 不依赖 Ctrl，普通输入不触发快捷键", () => {
  assert.equal(browserShortcutFromKey(key("Escape")), "escape");
  assert.equal(browserShortcutFromKey(key("a")), null);
});
