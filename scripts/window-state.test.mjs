import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const app = await readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

test("重新打开扩展弹窗必须销毁旧窗口，避免 WebView 白屏旧状态", () => {
  const popup = app.slice(
    app.indexOf("async fn show_extension_popup"),
    app.indexOf("fn toggle_extension_pin"),
  );
  const reuse = popup.slice(
    popup.indexOf('if let Some(popup) = app.get_webview_window("extension-popup")'),
    popup.indexOf("// 必须与标签页 WebView"),
  );
  assert.match(reuse, /popup\.destroy\(\)/);
  assert.doesNotMatch(reuse, /popup\.show\(\)/);
  assert.doesNotMatch(reuse, /popup\.navigate\(/);
});

test("窗口状态只恢复主窗口，不能显示隐藏的扩展宿主", () => {
  const registration = app.slice(
    app.indexOf("tauri_plugin_window_state::Builder"),
    app.indexOf("tauri_plugin_global_shortcut::Builder"),
  );
  assert.match(registration, /\.with_filter\(\|label\| label == "main"\)/);
});

test("新建扩展面板必须等扩展页加载完成后再显示", () => {
  const popup = app.slice(
    app.indexOf("async fn show_extension_popup"),
    app.indexOf("fn toggle_extension_pin"),
  );
  assert.match(popup, /\.visible\(false\)\.on_page_load/);
  assert.match(popup, /PageLoadEvent::Finished/);
  assert.match(popup, /payload\.url\(\)\.as_str\(\) == "about:blank"/);
});
