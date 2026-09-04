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
    display_title_from_url, AppSnapshot, AppState, DownloadRecord, HistoryEntry, RuntimeData,
    TabRecord,
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
    GetKeyState, VK_0, VK_ADD, VK_CONTROL, VK_D, VK_F, VK_H, VK_J, VK_L, VK_MENU, VK_OEM_MINUS,
    VK_OEM_PLUS, VK_SHIFT, VK_SUBTRACT, VK_T, VK_TAB, VK_W,
};

pub const CHROME_HEIGHT: f64 = 86.0;

/// 代理设置 → WebView2 附加浏览器参数；system 模式返回 None（使用默认行为）。
/// 代理分支会覆盖 Wry 默认参数，因此只补回 UI/PDF 兼容项，不关闭 SmartScreen。
const WEBVIEW2_DEFAULT_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI";

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
    // 创建即下沉：保证新标签 WebView 永远位于 UI 层（main WebView）之下。
    sink_tab_below_shell(app);
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
                rollback_opened_tab_transition(
                    app,
                    &id,
                    previous_active_id.clone(),
                    previous_shell_mode,
                    None,
                );
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
            rollback_opened_tab_transition(
                app,
                &id,
                previous_active_id.clone(),
                previous_shell_mode,
                Some(&closed),
            );
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
    let transition_origin = TabTransitionOrigin {
        previous_active_id,
        previous_shell_mode,
        webview_was_present,
    };
    let rollback = || {
        rollback_tab_transition(
            app,
            tab_id,
            &previous_tab,
            &transition_origin,
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
    let transition_origin = TabTransitionOrigin {
        previous_active_id,
        previous_shell_mode,
        webview_was_present,
    };
    let rollback = || {
        rollback_tab_transition(
            app,
            tab_id,
            &previous_tab,
            &transition_origin,
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

/// 把标签层 WebView 压到主 WebView（React UI 层）之下。
/// Win32 同级子窗口按创建顺序叠放，后创建的 tab WebView 默认盖在 UI 上；
/// tauri::Webview 未暴露 HWND，这里枚举主窗口直接子窗口，
/// bounds 未覆盖完整客户区的（tab 层，高度少 CHROME_HEIGHT）全部压到 z 序最底，
/// 满幅的 main UI WebView 天然在其上——地址下拉、菜单等浮层原生覆盖网页。
#[cfg(windows)]
fn sink_tab_below_shell(app: &AppHandle) {
    use windows::Win32::Foundation::{HWND, LPARAM, RECT};
    use windows::Win32::UI::WindowsAndMessaging::{EnumChildWindows, GetClientRect};

    struct SinkCtx {
        parent: HWND,
        client: RECT,
    }

    unsafe extern "system" fn sink_enum_proc(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
        use windows::core::BOOL;
        use windows::Win32::Foundation::{POINT, RECT};
        use windows::Win32::Graphics::Gdi::MapWindowPoints;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowRect, SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        };

        let ctx = &*(lparam.0 as *const SinkCtx);
        let mut rect = RECT::default();
        if unsafe { GetWindowRect(hwnd, &mut rect) }.is_err() {
            return BOOL(1);
        }
        let mut corners = [
            POINT {
                x: rect.left,
                y: rect.top,
            },
            POINT {
                x: rect.right,
                y: rect.bottom,
            },
        ];
        unsafe { MapWindowPoints(None, Some(ctx.parent), &mut corners) };
        let covers_client = corners[0].x <= 0
            && corners[0].y <= 0
            && corners[1].x >= ctx.client.right
            && corners[1].y >= ctx.client.bottom;
        if !covers_client {
            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    Some(HWND_BOTTOM),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
        BOOL(1)
    }

    let Some(window) = app.get_window("main") else {
        return;
    };
    let Ok(hwnd) = window.hwnd() else {
        return;
    };
    let native = HWND(hwnd.0 as *mut _);
    let mut client = RECT::default();
    if unsafe { GetClientRect(native, &mut client) }.is_err() {
        return;
    }
    let ctx = SinkCtx {
        parent: native,
        client,
    };
    unsafe {
        let _ = EnumChildWindows(
            Some(native),
            Some(sink_enum_proc),
            LPARAM(&ctx as *const SinkCtx as isize),
        );
    }
}

#[cfg(not(windows))]
fn sink_tab_below_shell(_app: &AppHandle) {}

pub fn resize_tabs(app: &AppHandle) {
    let Ok((position, size)) = browser_bounds(app) else {
        return;
    };
    let collapsed = app
        .state::<AppState>()
        .inner
        .lock()
        .map(|runtime| runtime.shell_collapsed)
        .unwrap_or(false);
    for webview in app.webviews().values() {
        if webview.label().starts_with("tab-") {
            let _ = webview.set_position(position);
            let _ = webview.set_size(size);
        }
    }
    if let Some(shell) = app.get_webview("main") {
        // 窗口尺寸变化时，main WebView 按当前模式（收缩=仅 chrome / 满幅）同步宽度。
        let shell_size = if collapsed {
            LogicalSize::new(size.width, CHROME_HEIGHT)
        } else {
            LogicalSize::new(size.width, size.height + CHROME_HEIGHT)
        };
        let _ = shell.set_position(LogicalPosition::new(0.0, 0.0));
        let _ = shell.set_size(shell_size);
    }
}

/// 浏览态把 main WebView 收缩到顶部 chrome（网页区域无遮挡、鼠标直达网页）；
/// 浮层/页面/锁屏需要覆盖内容区时扩回满幅（UI 层在 z 序上方，浮层照常盖住网页）。
pub fn set_shell_expanded(app: &AppHandle, expanded: bool) -> Result<(), String> {
    let collapsed = !expanded;
    let changed = app
        .state::<AppState>()
        .inner
        .lock()
        .map(|runtime| runtime.shell_collapsed != collapsed)
        .unwrap_or(true);
    if !changed {
        return Ok(());
    }
    app.state::<AppState>()
        .mutate_runtime(|runtime| runtime.shell_collapsed = collapsed)?;
    let Some(window) = app.get_window("main") else {
        return Ok(());
    };
    let size = window.inner_size().map_err(|error| error.to_string())?;
    let scale = window.scale_factor().map_err(|error| error.to_string())?;
    let logical = size.to_logical::<f64>(scale);
    let Some(shell) = app.get_webview("main") else {
        return Ok(());
    };
    let target = if collapsed {
        LogicalSize::new(logical.width.max(1.0), CHROME_HEIGHT)
    } else {
        LogicalSize::new(logical.width.max(1.0), logical.height.max(1.0))
    };
    shell
        .set_position(LogicalPosition::new(0.0, 0.0))
        .map_err(|error| error.to_string())?;
    shell.set_size(target).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn show_active_tab(app: &AppHandle) {
    let state = app.state::<AppState>();
    let active = state.inner.lock().ok().and_then(|runtime| {
        if tab_content_should_be_visible(&runtime) {
            runtime.data.active_tab_id.clone()
        } else {
            None
        }
    });
    hide_all_tabs(app);
    if let Some(id) = active {
        if let Some(webview) = app.get_webview(&tab_label(&id)) {
            // 防御性再下沉一次：任何路径把 z 序抬起来都能在这里纠正。
            sink_tab_below_shell(app);
            let _ = webview.show();
            let _ = webview.set_focus();
        }
    }
}

fn tab_content_should_be_visible(runtime: &RuntimeData) -> bool {
    // 网页常驻 UI 层之下（UI 透明区域透出网页），shell 页面不再隐藏网页；
    // 仅窗口隐藏（恢复前台窗口）和锁屏（隐私）时收起。
    runtime.window_visible && !runtime.locked
}

pub fn set_shell_mode(app: &AppHandle, enabled: bool) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    state.mutate_runtime(|runtime| runtime.shell_mode = enabled)?;
    show_active_tab(app);
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
fn configured_tab_action(
    app: &AppHandle,
    ctrl: bool,
    shift: bool,
    alt: bool,
    key: u16,
) -> Option<&'static str> {
    let key_name = if (b'A' as u16..=b'Z' as u16).contains(&key)
        || (b'0' as u16..=b'9' as u16).contains(&key)
    {
        char::from_u32(key as u32)?.to_ascii_lowercase().to_string()
    } else {
        match key {
            value if value == VK_TAB.0 => "tab".into(),
            value if value == VK_OEM_PLUS.0 || value == VK_ADD.0 => "=".into(),
            value if value == VK_OEM_MINUS.0 || value == VK_SUBTRACT.0 => "-".into(),
            _ => return None,
        }
    };
    let current = format!(
        "{}{}{}{}",
        if ctrl { "ctrl+" } else { "" },
        if alt { "alt+" } else { "" },
        if shift { "shift+" } else { "" },
        key_name
    );
    let state = app.state::<AppState>();
    let runtime = state.inner.lock().ok()?;
    let canonical = |value: &str| {
        value
            .split('+')
            .map(|part| part.trim().to_ascii_lowercase())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("+")
    };
    let current = canonical(&current);
    if current == canonical(&runtime.data.settings.tab_search_shortcut) {
        Some("tab-search")
    } else if current == canonical(&runtime.data.settings.recently_closed_shortcut) {
        Some("recently-closed")
    } else {
        None
    }
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
                    let alt = unsafe { GetKeyState(VK_MENU.0 as i32) } < 0;
                    let shortcut = configured_tab_action(&app, ctrl, shift, alt, key as u16).or(
                        match (ctrl, shift, key as u16) {
                            (true, false, key) if key == VK_TAB.0 => Some("next-tab"),
                            (true, true, key) if key == VK_TAB.0 => Some("previous-tab"),
                            (true, false, key) if key == VK_T.0 => Some("new-tab"),
                            (true, true, key) if key == VK_T.0 => Some("restore-tab"),
                            (true, false, key) if key == VK_L.0 => Some("focus-address"),
                            (true, false, key) if key == VK_W.0 => Some("close-tab"),
                            (true, false, key) if key == VK_H.0 => Some("history"),
                            (true, false, key) if key == VK_J.0 => Some("downloads"),
                            (true, false, key) if key == VK_D.0 => Some("bookmark"),
                            (true, false, key) if key == VK_F.0 => Some("find"),
                            (true, false, key) if key == VK_OEM_PLUS.0 || key == VK_ADD.0 => {
                                Some("zoom-in")
                            }
                            (true, false, key) if key == VK_OEM_MINUS.0 || key == VK_SUBTRACT.0 => {
                                Some("zoom-out")
                            }
                            (true, false, key) if key == VK_0.0 => Some("zoom-reset"),
                            _ => None,
                        },
                    );
                    let Some(shortcut) = shortcut else {
                        return Ok(());
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

fn rollback_opened_tab_transition(
    app: &AppHandle,
    tab_id: &str,
    previous_active_id: Option<String>,
    previous_shell_mode: bool,
    recently_closed: Option<&TabRecord>,
) {
    let state = app.state::<AppState>();
    let rolled_back = state
        .mutate_runtime(|runtime| {
            rollback_opened_tab_data(
                runtime,
                tab_id,
                &previous_active_id,
                previous_shell_mode,
                recently_closed,
            )
        })
        .unwrap_or(false);
    if !rolled_back {
        emit_snapshot(app);
        return;
    }
    let _ = state.save();

    if let Some(webview) = app.get_webview(&tab_label(tab_id)) {
        let _ = webview.close();
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

fn rollback_opened_tab_data(
    runtime: &mut RuntimeData,
    tab_id: &str,
    previous_active_id: &Option<String>,
    previous_shell_mode: bool,
    recently_closed: Option<&TabRecord>,
) -> bool {
    // 后续切换已接管焦点时，不撤销它的状态。
    if runtime.data.active_tab_id.as_deref() != Some(tab_id) {
        return false;
    }
    runtime.data.tabs.retain(|tab| tab.id != tab_id);
    if let Some(closed) = recently_closed {
        if !runtime
            .data
            .recently_closed
            .iter()
            .any(|tab| tab.id == closed.id)
        {
            runtime.data.recently_closed.insert(0, closed.clone());
            runtime.data.recently_closed.truncate(20);
        }
    }
    runtime.data.active_tab_id = previous_active_id.clone();
    runtime.shell_mode = previous_shell_mode;
    sort_tabs(&mut runtime.data.tabs);
    true
}

struct TabTransitionOrigin {
    previous_active_id: Option<String>,
    previous_shell_mode: bool,
    webview_was_present: bool,
}

fn rollback_tab_transition(
    app: &AppHandle,
    tab_id: &str,
    previous_tab: &TabRecord,
    origin: &TabTransitionOrigin,
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
            runtime.data.active_tab_id = origin.previous_active_id.clone();
            runtime.shell_mode = origin.previous_shell_mode;
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
        if origin.webview_was_present {
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
    if !origin.previous_shell_mode {
        if let Some(previous_id) = origin.previous_active_id.as_deref() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PersistedData;

    fn runtime_with(
        tabs: Vec<TabRecord>,
        active_tab_id: Option<String>,
        recently_closed: Vec<TabRecord>,
        shell_mode: bool,
    ) -> RuntimeData {
        RuntimeData {
            data: PersistedData {
                tabs,
                active_tab_id,
                recently_closed,
                ..PersistedData::default()
            },
            locked: false,
            first_run: false,
            window_visible: true,
            quitting: false,
            shell_mode,
            shell_collapsed: false,
            previous_window: 0,
            hidden_since: None,
        }
    }

    #[test]
    fn proxy_args_keep_smart_screen_enabled() {
        assert_eq!(
            proxy_browser_args("direct", ""),
            Some("--disable-features=msWebOOUI,msPdfOOUI --no-proxy-server".into())
        );
        assert_eq!(
            proxy_browser_args("custom", " http://127.0.0.1:8080 "),
            Some(
                "--disable-features=msWebOOUI,msPdfOOUI --proxy-server=http://127.0.0.1:8080"
                    .into(),
            )
        );
    }
    #[test]
    fn tab_content_visibility_requires_window_and_unlock() {
        let runtime = runtime_with(Vec::new(), None, Vec::new(), false);
        assert!(tab_content_should_be_visible(&runtime));

        let mut hidden = runtime.clone();
        hidden.window_visible = false;
        assert!(!tab_content_should_be_visible(&hidden));

        let mut locked = runtime;
        locked.locked = true;
        assert!(!tab_content_should_be_visible(&locked));
    }

    #[test]
    fn rollback_opened_tab_removes_a_failed_new_tab() {
        let previous = TabRecord::new("https://before.example".into(), "Before".into(), false);
        let opened = TabRecord::new("https://opened.example".into(), "Opened".into(), false);
        let mut runtime = runtime_with(
            vec![previous.clone(), opened.clone()],
            Some(opened.id.clone()),
            Vec::new(),
            false,
        );

        assert!(rollback_opened_tab_data(
            &mut runtime,
            &opened.id,
            &Some(previous.id.clone()),
            false,
            None,
        ));
        assert_eq!(runtime.data.tabs.len(), 1);
        assert_eq!(runtime.data.tabs[0].id, previous.id);
        assert_eq!(runtime.data.active_tab_id, Some(previous.id));
        assert!(!runtime.shell_mode);
    }

    #[test]
    fn rollback_opened_tab_returns_a_failed_restore_to_recently_closed() {
        let previous = TabRecord::new("quickpane://newtab".into(), "新标签页".into(), false);
        let closed = TabRecord::new("https://restore.example".into(), "Restore".into(), false);
        let mut restored = closed.clone();
        restored.muted = false;
        let mut runtime = runtime_with(
            vec![previous.clone(), restored.clone()],
            Some(restored.id.clone()),
            Vec::new(),
            false,
        );

        assert!(rollback_opened_tab_data(
            &mut runtime,
            &restored.id,
            &Some(previous.id.clone()),
            true,
            Some(&closed),
        ));
        assert_eq!(runtime.data.tabs.len(), 1);
        assert_eq!(runtime.data.active_tab_id, Some(previous.id));
        assert!(runtime.shell_mode);
        assert_eq!(runtime.data.recently_closed.len(), 1);
        assert_eq!(runtime.data.recently_closed[0].id, closed.id);
    }
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
