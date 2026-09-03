use std::{path::PathBuf, time::SystemTime};

use chrono::Utc;
use tauri::{
    webview::{DownloadEvent, PageLoadEvent, WebviewBuilder},
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl,
};
use tauri_plugin_dialog::DialogExt;
use url::Url;
use uuid::Uuid;

use crate::state::{
    display_title_from_url, AppSnapshot, AppState, DownloadRecord, HistoryEntry, TabRecord,
};

#[cfg(windows)]
use webview2_com::{
    AcceleratorKeyPressedEventHandler,
    Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_KEY_EVENT_KIND, COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN,
        COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN,
    },
};
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VK_0, VK_ADD, VK_CONTROL, VK_D, VK_ESCAPE, VK_F, VK_H, VK_J, VK_L, VK_OEM_MINUS,
    VK_OEM_PLUS, VK_SHIFT, VK_SUBTRACT, VK_T, VK_TAB, VK_W,
};

pub const CHROME_HEIGHT: f64 = 86.0;

/// WebView2 的默认附加参数（与 Tauri 默认值保持一致），注入代理时必须一并带上。
const WEBVIEW2_DEFAULT_ARGS: &str =
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

const MUTE_TAB_SCRIPT: &str = r#"
window.__qpMuteObserver?.disconnect();
const muteQuickPaneMedia = () => document.querySelectorAll('audio,video').forEach((media) => {
  if (!('qpMuted' in media.dataset)) media.dataset.qpMuted = media.muted ? '1' : '0';
  media.muted = true;
});
muteQuickPaneMedia();
window.__qpMuteObserver = new MutationObserver(muteQuickPaneMedia);
window.__qpMuteObserver.observe(document.documentElement, { childList: true, subtree: true });
"#;

const UNMUTE_TAB_SCRIPT: &str = r#"
window.__qpMuteObserver?.disconnect();
document.querySelectorAll('audio,video').forEach((media) => {
  media.muted = media.dataset.qpMuted === '1';
  delete media.dataset.qpMuted;
});
delete window.__qpMuteObserver;
"#;

const OPEN_TARGET_BLANK_IN_CURRENT_TAB: &str = r#"
document.addEventListener("click", (event) => {
  if (
    event.defaultPrevented ||
    event.button !== 0 ||
    event.metaKey ||
    event.ctrlKey ||
    event.shiftKey ||
    event.altKey
  ) {
    return;
  }

  const target = event.target;
  const link = target instanceof Element
    ? target.closest('a[target="_blank"][href]')
    : null;
  if (!link) {
    return;
  }

  event.preventDefault();
  event.stopImmediatePropagation();
  window.location.assign(link.href);
}, true);
"#;

/// 代理设置 → WebView2 附加浏览器参数；system 模式返回 None（使用默认行为）。
pub fn proxy_browser_args(proxy_mode: &str, proxy_url: &str) -> Option<String> {
    match proxy_mode {
        "direct" => Some(format!("{WEBVIEW2_DEFAULT_ARGS} --no-proxy-server")),
        "custom" => {
            let url = proxy_url.trim();
            (!url.is_empty()).then(|| format!("{WEBVIEW2_DEFAULT_ARGS} --proxy-server={url}"))
        }
        _ => None,
    }
}

/// 代理是 WebView2 环境级参数，修改后必须关闭全部标签 WebView 再重建才生效。
pub async fn recreate_tab_webviews(app: &AppHandle) -> Result<(), String> {
    let labels: Vec<String> = app
        .webviews()
        .keys()
        .filter(|label| label.starts_with("tab-"))
        .cloned()
        .collect();
    for label in labels {
        if let Some(webview) = app.get_webview(&label) {
            let _ = webview.close();
        }
    }
    let active = app.state::<AppState>().inner.lock().ok().and_then(|guard| {
        if guard.locked || guard.shell_mode {
            return None;
        }
        let tab = guard
            .data
            .tabs
            .iter()
            .find(|tab| Some(&tab.id) == guard.data.active_tab_id.as_ref())?;
        (tab.url != "quickpane://newtab").then(|| tab.id.clone())
    });
    if let Some(tab_id) = active {
        ensure_tab_webview(app, &tab_id)?;
    }
    Ok(())
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct TabBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn emit_snapshot(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        let _ = app.emit("app-snapshot", state.snapshot());
    }
}

pub fn normalize_input(input: &str, search_template: &str) -> Result<String, String> {
    let value = input.trim();
    if value.is_empty() || value == "quickpane://newtab" {
        return Ok("quickpane://newtab".into());
    }

    if let Ok(url) = Url::parse(value) {
        return match url.scheme() {
            // chrome-extension:// 用于在标签页中打开扩展面板页（WebView2 无弹出宿主）。
            "http" | "https" | "chrome-extension" => Ok(url.to_string()),
            _ => Err("仅支持 HTTP 和 HTTPS 地址".into()),
        };
    }

    let looks_like_host = !value.contains(char::is_whitespace)
        && (value.contains('.')
            || value.eq_ignore_ascii_case("localhost")
            || value.starts_with("localhost:")
            || value.parse::<std::net::IpAddr>().is_ok());

    if looks_like_host {
        let candidate = format!("https://{value}");
        return Url::parse(&candidate)
            .map(|url| url.to_string())
            .map_err(|_| "网址格式无效".into());
    }

    let encoded: String = url::form_urlencoded::byte_serialize(value.as_bytes()).collect();
    let target = search_template.replace("{query}", &encoded);
    Url::parse(&target)
        .map(|url| url.to_string())
        .map_err(|_| "搜索引擎地址无效".into())
}

pub fn browser_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("WebView2");
    std::fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

/// 未打包浏览器扩展的安装目录，每个子文件夹一个扩展。
/// wry 创建 WebView 时会读取该目录（不存在会导致创建失败），因此这里保证目录存在。
pub fn extensions_dir(app: &AppHandle) -> PathBuf {
    let path = app
        .path()
        .app_data_dir()
        .map(|dir| dir.join("Extensions"))
        .unwrap_or_else(|_| PathBuf::from("Extensions"));
    let _ = std::fs::create_dir_all(&path);
    path
}

fn browser_bounds(app: &AppHandle) -> Result<(LogicalPosition<f64>, LogicalSize<f64>), String> {
    let window = app
        .get_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    let size = window.inner_size().map_err(|error| error.to_string())?;
    let scale = window.scale_factor().map_err(|error| error.to_string())?;
    let logical = size.to_logical::<f64>(scale);
    Ok((
        LogicalPosition::new(0.0, CHROME_HEIGHT),
        LogicalSize::new(
            logical.width.max(1.0),
            (logical.height - CHROME_HEIGHT).max(1.0),
        ),
    ))
}

pub fn ensure_tab_webview(app: &AppHandle, tab_id: &str) -> Result<(), String> {
    let label = tab_label(tab_id);
    if app.get_webview(&label).is_some() {
        return Ok(());
    }

    let (url, title, is_active, muted, locked, shell_mode, proxy_args) = {
        let state = app.state::<AppState>();
        let guard = state.inner.lock().map_err(|_| "应用状态无法读取")?;
        let tab = guard
            .data
            .tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .ok_or_else(|| "标签页不存在".to_string())?;
        (
            tab.url.clone(),
            tab.title.clone(),
            guard.data.active_tab_id.as_deref() == Some(tab_id),
            tab.muted,
            guard.locked,
            guard.shell_mode,
            proxy_browser_args(
                &guard.data.settings.proxy_mode,
                &guard.data.settings.proxy_url,
            ),
        )
    };

    if locked || shell_mode || url == "quickpane://newtab" {
        return Ok(());
    }

    let external = Url::parse(&url).map_err(|_| "标签页网址无效".to_string())?;
    let data_dir = browser_data_dir(app)?;
    let app_for_title = app.clone();
    let app_for_load = app.clone();
    let app_for_download = app.clone();
    let app_for_new_window = app.clone();
    let tab_id_for_title = tab_id.to_string();
    let tab_id_for_load = tab_id.to_string();

    let builder = WebviewBuilder::new(label.clone(), WebviewUrl::External(external))
        .data_directory(data_dir)
        .focused(is_active)
        .enable_clipboard_access()
        .zoom_hotkeys_enabled(true)
        .devtools(cfg!(debug_assertions))
        .initialization_script(OPEN_TARGET_BLANK_IN_CURRENT_TAB)
        .initialization_script(if muted { MUTE_TAB_SCRIPT } else { "" })
        // 扩展加载进标签共享的 WebView2 Profile；目录不存在时 wry 的加载器安全跳过。
        .browser_extensions_enabled(true)
        .extensions_path(extensions_dir(app));
    let builder = match &proxy_args {
        Some(args) => builder.additional_browser_args(args),
        None => builder,
    };
    let builder = builder
        .on_document_title_changed(move |_webview, next_title| {
            let state = app_for_title.state::<AppState>();
            let _ = state.mutate(|runtime| {
                if let Some(tab) = runtime
                    .data
                    .tabs
                    .iter_mut()
                    .find(|tab| tab.id == tab_id_for_title)
                {
                    tab.title = if next_title.trim().is_empty() {
                        display_title_from_url(&tab.url)
                    } else {
                        next_title
                    };
                }
            });
            emit_snapshot(&app_for_title);
        })
        .on_page_load(move |_webview, payload| {
            let next_url = payload.url().to_string();
            let loading = payload.event() == PageLoadEvent::Started;
            let state = app_for_load.state::<AppState>();
            let _ = state.mutate(|runtime| {
                let title = if let Some(tab) = runtime
                    .data
                    .tabs
                    .iter_mut()
                    .find(|tab| tab.id == tab_id_for_load)
                {
                    tab.url = next_url.clone();
                    tab.loading = loading;
                    tab.loaded = true;
                    tab.last_active_at = Utc::now();
                    tab.title.clone()
                } else {
                    String::new()
                };
                if !loading && next_url.starts_with("http") {
                    runtime.data.history.insert(
                        0,
                        HistoryEntry {
                            id: Uuid::new_v4().simple().to_string(),
                            title,
                            url: next_url,
                            visited_at: Utc::now(),
                        },
                    );
                    runtime.data.history.truncate(5_000);
                }
            });
            emit_snapshot(&app_for_load);
        })
        .on_download(move |_webview, event| handle_download(&app_for_download, event))
        .on_new_window(move |url, _features| {
            let url = url.to_string();
            if let Err(error) = create_tab(&app_for_new_window, Some(url), true) {
                eprintln!("无法打开新标签页: {error}");
            }
            tauri::webview::NewWindowResponse::Deny
        });

    let (position, size) = browser_bounds(app)?;
    let window = app
        .get_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    let webview = window
        .add_child(builder, position, size)
        .map_err(|error| error.to_string())?;
    install_tab_shortcuts(app, &webview)?;
    if !is_active {
        webview.hide().map_err(|error| error.to_string())?;
    }

    let state = app.state::<AppState>();
    state.mutate(|runtime| {
        if let Some(tab) = runtime.data.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            tab.loaded = true;
            tab.loading = true;
            if tab.title.is_empty() {
                tab.title = title;
            }
        }
    })?;
    emit_snapshot(app);
    Ok(())
}

pub fn create_tab(
    app: &AppHandle,
    input: Option<String>,
    activate: bool,
) -> Result<AppSnapshot, String> {
    let (url, title, previous_active_id, previous_shell_mode) = {
        let state = app.state::<AppState>();
        let guard = state.inner.lock().map_err(|_| "应用状态无法读取")?;
        let value = input.unwrap_or_else(|| "quickpane://newtab".into());
        let url = normalize_input(&value, &guard.data.settings.search_template)?;
        let title = if url == "quickpane://newtab" {
            "新标签页".into()
        } else {
            display_title_from_url(&url)
        };
        (
            url,
            title,
            guard.data.active_tab_id.clone(),
            guard.shell_mode,
        )
    };

    let tab = TabRecord::new(url.clone(), title, false);
    let id = tab.id.clone();
    let state = app.state::<AppState>();
    state.mutate(|runtime| {
        runtime.data.tabs.push(tab);
        if activate {
            runtime.data.active_tab_id = Some(id.clone());
            runtime.shell_mode = url == "quickpane://newtab";
        }
        sort_tabs(&mut runtime.data.tabs);
    })?;

    if activate {
        hide_all_tabs(app);
        if url != "quickpane://newtab" {
            if let Err(error) = ensure_tab_webview(app, &id) {
                if let Some(webview) = app.get_webview(&tab_label(&id)) {
                    let _ = webview.close();
                }
                let _ = state.mutate_runtime(|runtime| {
                    runtime.data.tabs.retain(|tab| tab.id != id);
                    if runtime.data.active_tab_id.as_deref() == Some(&id) {
                        runtime.data.active_tab_id = previous_active_id.clone();
                        runtime.shell_mode = previous_shell_mode;
                    }
                    sort_tabs(&mut runtime.data.tabs);
                });
                let _ = state.save();
                hide_all_tabs(app);
                if !previous_shell_mode {
                    if let Some(previous_id) = previous_active_id.as_deref() {
                        let _ = ensure_tab_webview(app, previous_id);
                    }
                }
                show_active_tab(app);
                emit_snapshot(app);
                return Err(error);
            }
        }
    }
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub fn set_tab_pinned(app: &AppHandle, tab_id: &str, pinned: bool) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    state.mutate_result(|runtime| {
        let tab = runtime
            .data
            .tabs
            .iter_mut()
            .find(|tab| tab.id == tab_id)
            .ok_or_else(|| "标签页不存在".to_string())?;
        tab.pinned = pinned;
        sort_tabs(&mut runtime.data.tabs);
        Ok(())
    })?;
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub fn set_tab_muted(app: &AppHandle, tab_id: &str, muted: bool) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    state.mutate_result(|runtime| {
        let tab = runtime
            .data
            .tabs
            .iter_mut()
            .find(|tab| tab.id == tab_id)
            .ok_or_else(|| "标签页不存在".to_string())?;
        tab.muted = muted;
        Ok(())
    })?;
    if let Some(webview) = app.get_webview(&tab_label(tab_id)) {
        let script = if muted {
            MUTE_TAB_SCRIPT
        } else {
            UNMUTE_TAB_SCRIPT
        };
        webview.eval(script).map_err(|error| error.to_string())?;
    }
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub async fn restore_closed_tab(
    app: &AppHandle,
    tab_id: Option<&str>,
) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    let (closed, previous_active_id, previous_shell_mode) = {
        let guard = state.inner.lock().map_err(|_| "应用状态无法读取")?;
        (
            guard
                .data
                .recently_closed
                .iter()
                .find(|tab| tab_id.is_none_or(|requested| tab.id == requested))
                .cloned()
                .ok_or_else(|| "没有可恢复的标签页".to_string())?,
            guard.data.active_tab_id.clone(),
            guard.shell_mode,
        )
    };
    let id = closed.id.clone();
    let url = closed.url.clone();
    state.mutate_result(|runtime| {
        let index = runtime
            .data
            .recently_closed
            .iter()
            .position(|tab| tab.id == id)
            .ok_or_else(|| "没有可恢复的标签页".to_string())?;
        let mut restored = runtime.data.recently_closed.remove(index);
        restored.loading = false;
        restored.loaded = false;
        restored.muted = false;
        runtime.data.tabs.push(restored);
        runtime.data.active_tab_id = Some(id.clone());
        runtime.shell_mode = url == "quickpane://newtab";
        sort_tabs(&mut runtime.data.tabs);
        Ok::<(), String>(())
    })?;

    hide_all_tabs(app);
    if url != "quickpane://newtab" {
        if let Err(error) = ensure_tab_webview(app, &id) {
            if let Some(webview) = app.get_webview(&tab_label(&id)) {
                let _ = webview.close();
            }
            let _ = state.mutate_runtime(|runtime| {
                runtime.data.tabs.retain(|tab| tab.id != id);
                if !runtime
                    .data
                    .recently_closed
                    .iter()
                    .any(|tab| tab.id == closed.id)
                {
                    runtime.data.recently_closed.insert(0, closed.clone());
                    runtime.data.recently_closed.truncate(20);
                }
                if runtime.data.active_tab_id.as_deref() == Some(&id) {
                    runtime.data.active_tab_id = previous_active_id.clone();
                    runtime.shell_mode = previous_shell_mode;
                }
                sort_tabs(&mut runtime.data.tabs);
            });
            let _ = state.save();
            hide_all_tabs(app);
            if !previous_shell_mode {
                if let Some(previous_id) = previous_active_id.as_deref() {
                    let _ = ensure_tab_webview(app, previous_id);
                }
            }
            show_active_tab(app);
            emit_snapshot(app);
            return Err(error);
        }
    }
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub async fn activate_tab(app: &AppHandle, tab_id: &str) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    let (previous_tab, previous_active_id, previous_shell_mode, webview_was_present) = {
        let guard = state.inner.lock().map_err(|_| "应用状态无法读取")?;
        (
            guard
                .data
                .tabs
                .iter()
                .find(|tab| tab.id == tab_id)
                .cloned()
                .ok_or_else(|| "标签页不存在".to_string())?,
            guard.data.active_tab_id.clone(),
            guard.shell_mode,
            app.get_webview(&tab_label(tab_id)).is_some(),
        )
    };
    let is_new_tab = previous_tab.url == "quickpane://newtab";
    let rollback = || {
        rollback_tab_transition(
            app,
            tab_id,
            &previous_tab,
            previous_active_id.clone(),
            previous_shell_mode,
            webview_was_present,
            &previous_tab.url,
            false,
        );
    };
    let result = state.mutate_result(|runtime| {
        let tab = runtime
            .data
            .tabs
            .iter_mut()
            .find(|tab| tab.id == tab_id)
            .ok_or_else(|| "标签页不存在".to_string())?;
        tab.last_active_at = Utc::now();
        runtime.data.active_tab_id = Some(tab_id.to_string());
        runtime.shell_mode = is_new_tab;
        Ok(())
    });
    result?;

    hide_all_tabs(app);
    if !is_new_tab {
        if let Err(error) = ensure_tab_webview(app, tab_id) {
            rollback();
            return Err(error);
        }
        let Some(webview) = app.get_webview(&tab_label(tab_id)) else {
            let error = "当前页面尚未加载".to_string();
            rollback();
            return Err(error);
        };
        if let Err(error) = webview.show().map_err(|error| error.to_string()) {
            rollback();
            return Err(error);
        }
        if let Err(error) = webview.set_focus().map_err(|error| error.to_string()) {
            rollback();
            return Err(error);
        }
    }
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub async fn navigate_tab(
    app: &AppHandle,
    tab_id: &str,
    input: &str,
) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    let (target, previous_tab, previous_active_id, previous_shell_mode, webview_was_present) = {
        let guard = state.inner.lock().map_err(|_| "应用状态无法读取")?;
        let previous_tab = guard
            .data
            .tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .cloned()
            .ok_or_else(|| "标签页不存在".to_string())?;
        (
            normalize_input(input, &guard.data.settings.search_template)?,
            previous_tab,
            guard.data.active_tab_id.clone(),
            guard.shell_mode,
            app.get_webview(&tab_label(tab_id)).is_some(),
        )
    };
    let rollback = || {
        rollback_tab_transition(
            app,
            tab_id,
            &previous_tab,
            previous_active_id.clone(),
            previous_shell_mode,
            webview_was_present,
            &target,
            true,
        );
    };
    let result = state.mutate_result(|runtime| {
        let tab = runtime
            .data
            .tabs
            .iter_mut()
            .find(|tab| tab.id == tab_id)
            .ok_or_else(|| "标签页不存在".to_string())?;
        tab.url = target.clone();
        tab.title = display_title_from_url(&target);
        tab.loading = true;
        runtime.data.active_tab_id = Some(tab_id.to_string());
        runtime.shell_mode = false;
        Ok(())
    });
    result?;

    hide_all_tabs(app);
    if webview_was_present {
        let Some(webview) = app.get_webview(&tab_label(tab_id)) else {
            let error = "当前页面尚未加载".to_string();
            rollback();
            return Err(error);
        };
        let url = Url::parse(&target).map_err(|_| "网址格式无效".to_string())?;
        if let Err(error) = webview.navigate(url).map_err(|error| error.to_string()) {
            rollback();
            return Err(error);
        }
    } else if let Err(error) = ensure_tab_webview(app, tab_id) {
        rollback();
        return Err(error);
    }

    let Some(webview) = app.get_webview(&tab_label(tab_id)) else {
        let error = "当前页面尚未加载".to_string();
        rollback();
        return Err(error);
    };
    if let Err(error) = webview.show().map_err(|error| error.to_string()) {
        rollback();
        return Err(error);
    }
    if let Err(error) = webview.set_focus().map_err(|error| error.to_string()) {
        rollback();
        return Err(error);
    }
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub async fn close_tab(app: &AppHandle, tab_id: &str) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    let (closed_tab, previous_active_id, previous_shell_mode, webview_was_present, was_only_tab) = {
        let guard = state
            .inner
            .lock()
            .map_err(|_| "应用状态无法读取".to_string())?;
        (
            guard
                .data
                .tabs
                .iter()
                .find(|tab| tab.id == tab_id)
                .cloned()
                .ok_or_else(|| "标签页不存在".to_string())?,
            guard.data.active_tab_id.clone(),
            guard.shell_mode,
            app.get_webview(&tab_label(tab_id)).is_some(),
            guard.data.tabs.len() == 1,
        )
    };

    let (next_id, is_new_tab) = state.mutate_result(|runtime| {
        let index = runtime
            .data
            .tabs
            .iter()
            .position(|tab| tab.id == tab_id)
            .ok_or_else(|| "标签页不存在".to_string())?;
        let closed = runtime.data.tabs.remove(index);
        runtime.data.recently_closed.insert(0, closed);
        runtime.data.recently_closed.truncate(20);
        if runtime.data.tabs.is_empty() {
            runtime.data.tabs.push(TabRecord::new(
                "quickpane://newtab".into(),
                "新标签页".into(),
                false,
            ));
        }
        if runtime.data.active_tab_id.as_deref() == Some(tab_id) {
            // 保持关闭标签所在位置优先；只有关闭末尾标签时才回退到前一个。
            runtime.data.active_tab_id = runtime
                .data
                .tabs
                .get(index)
                .or_else(|| runtime.data.tabs.last())
                .map(|tab| tab.id.clone());
        }
        let next = runtime.data.active_tab_id.clone();
        let new_tab = next
            .as_ref()
            .and_then(|id| runtime.data.tabs.iter().find(|tab| &tab.id == id))
            .is_none_or(|tab| tab.url == "quickpane://newtab");
        runtime.shell_mode = new_tab;
        Ok((next, new_tab))
    })?;

    hide_all_tabs(app);
    if webview_was_present {
        if let Some(webview) = app.get_webview(&tab_label(tab_id)) {
            if let Err(error) = webview.close().map_err(|error| error.to_string()) {
                rollback_close_transition(
                    app,
                    &closed_tab,
                    previous_active_id.clone(),
                    previous_shell_mode,
                    next_id.as_deref(),
                    was_only_tab,
                    webview_was_present,
                );
                return Err(error);
            }
        }
    }

    if let Some(id) = next_id.as_deref() {
        if !is_new_tab {
            if let Err(error) = ensure_tab_webview(app, id) {
                rollback_close_transition(
                    app,
                    &closed_tab,
                    previous_active_id.clone(),
                    previous_shell_mode,
                    Some(id),
                    was_only_tab,
                    webview_was_present,
                );
                return Err(error);
            }
            let Some(webview) = app.get_webview(&tab_label(id)) else {
                let error = "当前页面尚未加载".to_string();
                rollback_close_transition(
                    app,
                    &closed_tab,
                    previous_active_id.clone(),
                    previous_shell_mode,
                    Some(id),
                    was_only_tab,
                    webview_was_present,
                );
                return Err(error);
            };
            if let Err(error) = webview.show().map_err(|error| error.to_string()) {
                rollback_close_transition(
                    app,
                    &closed_tab,
                    previous_active_id.clone(),
                    previous_shell_mode,
                    Some(id),
                    was_only_tab,
                    webview_was_present,
                );
                return Err(error);
            }
            if let Err(error) = webview.set_focus().map_err(|error| error.to_string()) {
                rollback_close_transition(
                    app,
                    &closed_tab,
                    previous_active_id.clone(),
                    previous_shell_mode,
                    Some(id),
                    was_only_tab,
                    webview_was_present,
                );
                return Err(error);
            }
        }
    }
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub fn hide_all_tabs(app: &AppHandle) {
    for webview in app.webviews().values() {
        if webview.label().starts_with("tab-") {
            let _ = webview.hide();
        }
    }
}

pub fn resize_tabs(app: &AppHandle) {
    let Ok((position, size)) = browser_bounds(app) else {
        return;
    };
    for webview in app.webviews().values() {
        if webview.label().starts_with("tab-") {
            let _ = webview.set_position(position);
            let _ = webview.set_size(size);
        }
    }
}

pub fn show_active_tab(app: &AppHandle) {
    let state = app.state::<AppState>();
    let active = state.inner.lock().ok().and_then(|runtime| {
        if runtime.locked || runtime.shell_mode {
            None
        } else {
            runtime.data.active_tab_id.clone()
        }
    });
    hide_all_tabs(app);
    if let Some(id) = active {
        if let Some(webview) = app.get_webview(&tab_label(&id)) {
            let _ = webview.show();
            let _ = webview.set_focus();
        }
    }
}

pub fn set_shell_mode(app: &AppHandle, enabled: bool) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    state.mutate_runtime(|runtime| runtime.shell_mode = enabled)?;
    if enabled {
        hide_all_tabs(app);
    } else {
        show_active_tab(app);
    }
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub fn reload_active(app: &AppHandle) -> Result<(), String> {
    let id = active_tab_id(app)?;
    app.get_webview(&tab_label(&id))
        .ok_or_else(|| "当前页面尚未加载".to_string())?
        .reload()
        .map_err(|error| error.to_string())
}

pub fn eval_active(app: &AppHandle, script: &str) -> Result<(), String> {
    let id = active_tab_id(app)?;
    app.get_webview(&tab_label(&id))
        .ok_or_else(|| "当前页面尚未加载".to_string())?
        .eval(script)
        .map_err(|error| error.to_string())
}

pub fn set_zoom(app: &AppHandle, scale: f64) -> Result<(), String> {
    let id = active_tab_id(app)?;
    app.get_webview(&tab_label(&id))
        .ok_or_else(|| "当前页面尚未加载".to_string())?
        .set_zoom(scale.clamp(0.25, 5.0))
        .map_err(|error| error.to_string())
}

pub fn set_all_muted(app: &AppHandle, muted: bool) {
    let script = if muted {
        "document.querySelectorAll('audio,video').forEach((m)=>{m.dataset.qpMuted=m.muted?'1':'0';m.muted=true})"
    } else {
        "document.querySelectorAll('audio,video').forEach((m)=>{m.muted=m.dataset.qpMuted==='1';delete m.dataset.qpMuted})"
    };
    for webview in app.webviews().values() {
        if webview.label().starts_with("tab-") {
            let _ = webview.eval(script);
        }
    }
}

pub fn active_tab_id(app: &AppHandle) -> Result<String, String> {
    app.state::<AppState>()
        .inner
        .lock()
        .map_err(|_| "应用状态无法读取".to_string())?
        .data
        .active_tab_id
        .clone()
        .ok_or_else(|| "没有活动标签页".to_string())
}

pub fn tab_label(tab_id: &str) -> String {
    format!("tab-{tab_id}")
}

#[cfg(windows)]
fn install_tab_shortcuts(app: &AppHandle, webview: &tauri::Webview) -> Result<(), String> {
    let app = app.clone();
    webview
        .with_webview(move |platform| {
            let controller = platform.controller();
            // 以下 COM 调用只跨越 WebView2 的 FFI 边界，不改变 Rust 所有权数据。
            let handler =
                AcceleratorKeyPressedEventHandler::create(Box::new(move |_sender, args| {
                    let Some(args) = args else {
                        return Ok(());
                    };
                    let mut kind = COREWEBVIEW2_KEY_EVENT_KIND::default();
                    let mut key = 0;
                    unsafe {
                        args.KeyEventKind(&mut kind)?;
                        if kind != COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN
                            && kind != COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN
                        {
                            return Ok(());
                        }
                        args.VirtualKey(&mut key)?;
                    }
                    let ctrl = unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0;
                    let shift = unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0;
                    let shortcut = match (ctrl, shift, key as u16) {
                        (false, false, key) if key == VK_ESCAPE.0 => "escape",
                        (true, false, key) if key == VK_TAB.0 => "next-tab",
                        (true, true, key) if key == VK_TAB.0 => "previous-tab",
                        (true, false, key) if key == VK_T.0 => "new-tab",
                        (true, true, key) if key == VK_T.0 => "restore-tab",
                        (true, false, key) if key == VK_L.0 => "focus-address",
                        (true, false, key) if key == VK_W.0 => "close-tab",
                        (true, false, key) if key == VK_H.0 => "history",
                        (true, false, key) if key == VK_J.0 => "downloads",
                        (true, false, key) if key == VK_D.0 => "bookmark",
                        (true, false, key) if key == VK_F.0 => "find",
                        (true, false, key) if key == VK_OEM_PLUS.0 || key == VK_ADD.0 => "zoom-in",
                        (true, false, key) if key == VK_OEM_MINUS.0 || key == VK_SUBTRACT.0 => {
                            "zoom-out"
                        }
                        (true, false, key) if key == VK_0.0 => "zoom-reset",
                        _ => return Ok(()),
                    };
                    unsafe { args.SetHandled(true)? };
                    let _ = app.emit_to(
                        tauri::EventTarget::webview_window("main"),
                        "browser-shortcut",
                        shortcut,
                    );
                    Ok(())
                }));
            let mut token = 0;
            if let Err(error) =
                unsafe { controller.add_AcceleratorKeyPressed(&handler, &mut token) }
            {
                eprintln!("无法安装标签快捷键: {error}");
            }
        })
        .map_err(|error| error.to_string())
}

#[cfg(not(windows))]
fn install_tab_shortcuts(_app: &AppHandle, _webview: &tauri::Webview) -> Result<(), String> {
    Ok(())
}

fn sort_tabs(tabs: &mut [TabRecord]) {
    tabs.sort_by_key(|tab| !tab.pinned);
}

fn rollback_tab_transition(
    app: &AppHandle,
    tab_id: &str,
    previous_tab: &TabRecord,
    previous_active_id: Option<String>,
    previous_shell_mode: bool,
    webview_was_present: bool,
    expected_url: &str,
    restore_navigation: bool,
) {
    let state = app.state::<AppState>();
    let rolled_back = state
        .mutate_runtime(|runtime| {
            let Some(tab) = runtime.data.tabs.iter_mut().find(|tab| tab.id == tab_id) else {
                return false;
            };
            // 只撤销仍属于本次转换的状态，避免覆盖随后发生的导航或标签切换。
            if tab.url != expected_url || runtime.data.active_tab_id.as_deref() != Some(tab_id) {
                return false;
            }
            if restore_navigation {
                let pinned = tab.pinned;
                *tab = previous_tab.clone();
                tab.pinned = pinned;
            } else {
                tab.last_active_at = previous_tab.last_active_at;
            }
            runtime.data.active_tab_id = previous_active_id.clone();
            runtime.shell_mode = previous_shell_mode;
            sort_tabs(&mut runtime.data.tabs);
            true
        })
        .unwrap_or(false);
    if !rolled_back {
        emit_snapshot(app);
        return;
    }
    let _ = state.save();

    if let Some(webview) = app.get_webview(&tab_label(tab_id)) {
        if webview_was_present {
            if restore_navigation {
                if let Ok(url) = Url::parse(&previous_tab.url) {
                    let _ = webview.navigate(url);
                }
            }
        } else {
            let _ = webview.close();
        }
    }
    hide_all_tabs(app);
    if !previous_shell_mode {
        if let Some(previous_id) = previous_active_id.as_deref() {
            let _ = ensure_tab_webview(app, previous_id);
        }
    }
    show_active_tab(app);
    emit_snapshot(app);
}

fn rollback_close_transition(
    app: &AppHandle,
    closed_tab: &TabRecord,
    previous_active_id: Option<String>,
    previous_shell_mode: bool,
    expected_next_id: Option<&str>,
    was_only_tab: bool,
    webview_was_present: bool,
) {
    let state = app.state::<AppState>();
    let restored = state
        .mutate_runtime(|runtime| {
            if runtime.data.active_tab_id.as_deref() != expected_next_id
                || runtime.data.tabs.iter().any(|tab| tab.id == closed_tab.id)
            {
                return false;
            }
            let Some(index) = runtime
                .data
                .recently_closed
                .iter()
                .position(|tab| tab.id == closed_tab.id)
            else {
                return false;
            };
            runtime.data.recently_closed.remove(index);
            if was_only_tab {
                if let Some(next_id) = expected_next_id {
                    runtime.data.tabs.retain(|tab| tab.id != next_id);
                }
            }
            runtime.data.tabs.push(closed_tab.clone());
            runtime.data.active_tab_id = previous_active_id.clone();
            runtime.shell_mode = previous_shell_mode;
            sort_tabs(&mut runtime.data.tabs);
            true
        })
        .unwrap_or(false);
    if !restored {
        emit_snapshot(app);
        return;
    }

    let _ = state.save();
    hide_all_tabs(app);
    if webview_was_present && closed_tab.url != "quickpane://newtab" {
        let _ = ensure_tab_webview(app, &closed_tab.id);
    }
    if !previous_shell_mode {
        if let Some(previous_id) = previous_active_id.as_deref() {
            let _ = ensure_tab_webview(app, previous_id);
        }
    }
    show_active_tab(app);
    emit_snapshot(app);
}

fn handle_download(app: &AppHandle, event: DownloadEvent<'_>) -> bool {
    match event {
        DownloadEvent::Requested { url, destination } => {
            let suggested = destination
                .file_name()
                .and_then(|name| name.to_str())
                .filter(|name| !name.is_empty())
                .map(str::to_string)
                .or_else(|| {
                    url.path_segments()
                        .and_then(Iterator::last)
                        .filter(|name| !name.is_empty())
                        .map(str::to_string)
                })
                .unwrap_or_else(|| "download".into());
            let dialog = app
                .dialog()
                .file()
                .set_title("保存下载文件")
                .set_file_name(&suggested);
            let Some(path) = dialog
                .blocking_save_file()
                .and_then(|path| path.into_path().ok())
            else {
                return false;
            };
            *destination = path.clone();
            let state = app.state::<AppState>();
            let _ = state.mutate(|runtime| {
                runtime.data.downloads.insert(
                    0,
                    DownloadRecord {
                        id: Uuid::new_v4().simple().to_string(),
                        file_name: path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("download")
                            .to_string(),
                        url: url.to_string(),
                        path: Some(path.to_string_lossy().to_string()),
                        state: "downloading".into(),
                        started_at: Utc::now(),
                        finished_at: None,
                    },
                );
            });
            emit_snapshot(app);
            true
        }
        DownloadEvent::Finished { url, path, success } => {
            let state = app.state::<AppState>();
            let path_string = path
                .as_ref()
                .map(|value| value.to_string_lossy().to_string());
            let _ = state.mutate(|runtime| {
                if let Some(record) = runtime.data.downloads.iter_mut().find(|record| {
                    record.url == url.as_str()
                        && record.state == "downloading"
                        && path_string
                            .as_ref()
                            .is_none_or(|path| record.path.as_ref() == Some(path))
                }) {
                    record.state = if success { "completed" } else { "failed" }.into();
                    record.finished_at = Some(Utc::now());
                }
            });
            emit_snapshot(app);
            true
        }
        _ => true,
    }
}

pub fn freeze_idle_tabs(app: &AppHandle) {
    let cutoff = SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(300))
        .and_then(|value| value.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|value| value.as_secs() as i64)
        .unwrap_or_default();
    let active = active_tab_id(app).ok();
    let state = app.state::<AppState>();
    let guard = match state.inner.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    for tab in &guard.data.tabs {
        if Some(&tab.id) == active.as_ref() || tab.pinned {
            continue;
        }
        if tab.last_active_at.timestamp() < cutoff {
            if let Some(webview) = app.get_webview(&tab_label(&tab.id)) {
                let _ = webview
                    .eval("document.querySelectorAll('audio,video').forEach((m)=>m.pause())");
            }
        }
    }
}
