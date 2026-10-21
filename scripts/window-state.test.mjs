import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const app = await readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

test("窗口状态只恢复主窗口，不能显示隐藏的扩展宿主", () => {
  const registration = app.slice(
    app.indexOf("tauri_plugin_window_state::Builder"),
    app.indexOf("tauri_plugin_global_shortcut::Builder"),
  );
  // 插件默认 VISIBLE 恢复会调用 show；宿主 visible(false) 本身无法阻止它。
  assert.match(registration, /\.with_filter\(\|label\| label == "main"\)/);
});

test("扩展面板必须等扩展页加载完成后再显示", () => {
  const popup = app.slice(
    app.indexOf("async fn show_extension_popup"),
    app.indexOf("fn toggle_extension_pin"),
  );
  assert.match(popup, /\.visible\(false\)\.on_page_load/);
  assert.match(popup, /PageLoadEvent::Finished/);
  assert.match(popup, /payload\.url\(\)\.as_str\(\) == "about:blank"/);
});
