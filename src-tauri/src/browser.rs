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

pub const CHROME_HEIGHT: f64 = 86.0;

/// WebView2 的默认附加参数（与 Tauri 默认值保持一致），注入代理时必须一并带上。
const WEBVIEW2_DEFAULT_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

/// 代理设置 → WebView2 附加浏览器参数；system 模式返回 None（使用默认行为）。
pub fn proxy_browser_args(proxy_mode: &str, proxy_url: &str) -> Option<String> {
    match proxy_mode {
        "direct" => Some(format!("{WEBVIEW2_DEFAULT_ARGS} --no-proxy-server")),
        "custom" => {
            let url = proxy_url.trim();
            (!url.is_empty())
                .then(|| format!("{WEBVIEW2_DEFAULT_ARGS} --proxy-server={url}"))
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
        ensure_tab_webview(app, &tab_id).await?;
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

pub async fn ensure_tab_webview(app: &AppHandle, tab_id: &str) -> Result<(), String> {
    let label = tab_label(tab_id);
    if app.get_webview(&label).is_some() {
        return Ok(());
    }

    let (url, title, is_active, locked, shell_mode, proxy_args) = {
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
            let _ = app_for_new_window.emit("new-window-requested", url.to_string());
            tauri::webview::NewWindowResponse::Deny
        });

    let (position, size) = browser_bounds(app)?;
    let window = app
        .get_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    let webview = window
        .add_child(builder, position, size)
        .map_err(|error| error.to_string())?;
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

pub async fn create_tab(
    app: &AppHandle,
    input: Option<String>,
    activate: bool,
) -> Result<AppSnapshot, String> {
    let (url, title) = {
        let state = app.state::<AppState>();
        let guard = state.inner.lock().map_err(|_| "应用状态无法读取")?;
        let value = input.unwrap_or_else(|| "quickpane://newtab".into());
        let url = normalize_input(&value, &guard.data.settings.search_template)?;
        let title = if url == "quickpane://newtab" {
            "新标签页".into()
        } else {
            display_title_from_url(&url)
        };
        (url, title)
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
    })?;

    if activate {
        hide_all_tabs(app);
        if url != "quickpane://newtab" {
            ensure_tab_webview(app, &id).await?;
        }
    }
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub async fn activate_tab(app: &AppHandle, tab_id: &str) -> Result<AppSnapshot, String> {
    let is_new_tab = {
        let state = app.state::<AppState>();
        state.mutate(|runtime| {
            let tab = runtime
                .data
                .tabs
                .iter_mut()
                .find(|tab| tab.id == tab_id)
                .ok_or_else(|| "标签页不存在".to_string())?;
            tab.last_active_at = Utc::now();
            let is_new_tab = tab.url == "quickpane://newtab";
            runtime.data.active_tab_id = Some(tab_id.to_string());
            runtime.shell_mode = is_new_tab;
            Ok::<bool, String>(is_new_tab)
        })??
    };

    hide_all_tabs(app);
    if !is_new_tab {
        ensure_tab_webview(app, tab_id).await?;
        if let Some(webview) = app.get_webview(&tab_label(tab_id)) {
            webview.show().map_err(|error| error.to_string())?;
            webview.set_focus().map_err(|error| error.to_string())?;
        }
    }
    emit_snapshot(app);
    Ok(app.state::<AppState>().snapshot())
}

pub async fn navigate_tab(
    app: &AppHandle,
    tab_id: &str,
    input: &str,
) -> Result<AppSnapshot, String> {
    let target = {
        let state = app.state::<AppState>();
        let guard = state.inner.lock().map_err(|_| "应用状态无法读取")?;
        normalize_input(input, &guard.data.settings.search_template)?
    };
    if target == "quickpane://newtab" {
        return activate_tab(app, tab_id).await;
    }

    let state = app.state::<AppState>();
    state.mutate(|runtime| {
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
        Ok::<(), String>(())
    })??;

    hide_all_tabs(app);
    if let Some(webview) = app.get_webview(&tab_label(tab_id)) {
        let url = Url::parse(&target).map_err(|_| "网址格式无效".to_string())?;
        webview.navigate(url).map_err(|error| error.to_string())?;
        webview.show().map_err(|error| error.to_string())?;
        webview.set_focus().map_err(|error| error.to_string())?;
    } else {
        ensure_tab_webview(app, tab_id).await?;
    }
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub async fn close_tab(app: &AppHandle, tab_id: &str) -> Result<AppSnapshot, String> {
    if let Some(webview) = app.get_webview(&tab_label(tab_id)) {
        webview.close().map_err(|error| error.to_string())?;
    }

    let (next_id, is_new_tab) = {
        let state = app.state::<AppState>();
        state.mutate(|runtime| {
            if let Some(index) = runtime.data.tabs.iter().position(|tab| tab.id == tab_id) {
                let closed = runtime.data.tabs.remove(index);
                runtime.data.recently_closed.insert(0, closed);
                runtime.data.recently_closed.truncate(20);
            }
            if runtime.data.tabs.is_empty() {
                runtime.data.tabs.push(TabRecord::new(
                    "quickpane://newtab".into(),
                    "新标签页".into(),
                    false,
                ));
            }
            if runtime.data.active_tab_id.as_deref() == Some(tab_id) {
                runtime.data.active_tab_id = runtime.data.tabs.last().map(|tab| tab.id.clone());
            }
            let next = runtime.data.active_tab_id.clone();
            let new_tab = next
                .as_ref()
                .and_then(|id| runtime.data.tabs.iter().find(|tab| &tab.id == id))
                .is_none_or(|tab| tab.url == "quickpane://newtab");
            runtime.shell_mode = new_tab;
            (next, new_tab)
        })?
    };

    hide_all_tabs(app);
    if let Some(id) = next_id {
        if !is_new_tab {
            ensure_tab_webview(app, &id).await?;
            if let Some(webview) = app.get_webview(&tab_label(&id)) {
                let _ = webview.show();
                let _ = webview.set_focus();
            }
        }
    }
    emit_snapshot(app);
    Ok(app.state::<AppState>().snapshot())
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
        }
    }
}

pub fn set_shell_mode(app: &AppHandle, enabled: bool) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    state.mutate(|runtime| runtime.shell_mode = enabled)?;
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
