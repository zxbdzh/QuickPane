import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const nav = await readFile(
  new URL("../src/components/navigation-bar.tsx", import.meta.url),
  "utf8",
);
const browser = await readFile(
  new URL("../src-tauri/src/browser.rs", import.meta.url),
  "utf8",
);

test("导航栏本身建立堆叠层，汉堡菜单才能盖住浏览内容区", () => {
  assert.match(
    nav,
    /<nav className="relative z-40 flex h-\[54px\]/,
    "浏览态内容区在导航栏之后绘制；没有 z-index 时菜单会画在网页下面",
  );
});

test("扩幅后必须把标签 WebView 压到 UI 层之下", () => {
  const fn = browser.slice(
    browser.indexOf("pub fn set_shell_expanded"),
    browser.indexOf("pub fn show_active_tab"),
  );
  assert.match(fn, /if expanded \{\s*sink_tab_below_shell\(app\);/);
  assert.match(
    fn,
    /if !changed \{\s*if expanded \{\s*sink_tab_below_shell\(app\);/,
  );
});
