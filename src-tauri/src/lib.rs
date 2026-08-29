mod browser;
mod extensions;
mod state;
mod windowing;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use browser::{
    activate_tab, browser_data_dir, close_tab, create_tab, emit_snapshot, ensure_tab_webview,
    eval_active, extensions_dir, freeze_idle_tabs, navigate_tab, proxy_browser_args,
    recreate_tab_webviews, reload_active, resize_tabs, set_shell_mode, set_zoom,
};
use chrono::Utc;
use serde::Deserialize;
use state::{AppSnapshot, AppState, Bookmark, QuickLink};
use tauri::{AppHandle, Emitter, LogicalPosition, Manager, WindowEvent};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::UpdaterExt;
use url::Url;
use uuid::Uuid;
use windowing::{
    hide_window, install_session_lock_listener, install_tray, lock_app, quit_app,
    register_shortcut, show_window,
};

#[tauri::command]
fn get_snapshot(state: tauri::State<'_, AppState>) -> AppSnapshot {
    state.snapshot()
}

#[tauri::command]
async fn new_tab(
    app: AppHandle,
    url: Option<String>,
    activate: Option<bool>,
) -> Result<AppSnapshot, String> {
    create_tab(&app, url, activate.unwrap_or(true)).await
}

#[tauri::command]
async fn select_tab(app: AppHandle, tab_id: String) -> Result<AppSnapshot, String> {
    activate_tab(&app, &tab_id).await
}

#[tauri::command]
async fn remove_tab(app: AppHandle, tab_id: String) -> Result<AppSnapshot, String> {
    close_tab(&app, &tab_id).await
}

#[tauri::command]
async fn navigate(app: AppHandle, tab_id: String, input: String) -> Result<AppSnapshot, String> {
    navigate_tab(&app, &tab_id, &input).await
}

#[tauri::command]
fn reload(app: AppHandle) -> Result<(), String> {
    reload_active(&app)
}

#[tauri::command]
fn go_back(app: AppHandle) -> Result<(), String> {
    eval_active(&app, "history.back()")
}

#[tauri::command]
fn go_forward(app: AppHandle) -> Result<(), String> {
    eval_active(&app, "history.forward()")
}

#[tauri::command]
fn find_in_page(app: AppHandle) -> Result<(), String> {
    eval_active(
        &app,
        "window.find(prompt('在页面中查找') || '', false, false, true)",
    )
}

#[tauri::command]
fn zoom_page(app: AppHandle, scale: f64) -> Result<(), String> {
    set_zoom(&app, scale)
}

#[tauri::command]
fn show_shell(app: AppHandle, visible: bool) -> Result<AppSnapshot, String> {
    set_shell_mode(&app, visible)
}

#[tauri::command]
fn hide_to_tray(app: AppHandle) {
    hide_window(&app);
}

#[tauri::command]
fn show_from_tray(app: AppHandle) {
    show_window(&app);
}

#[tauri::command]
fn exit_app(app: AppHandle) {
    quit_app(&app);
}

#[tauri::command]
fn set_global_shortcut(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    shortcut: String,
) -> Result<AppSnapshot, String> {
    let previous = state
        .inner
        .lock()
        .map_err(|_| "应用状态无法读取".to_string())?
        .data
        .settings
        .shortcut
        .clone();
    if let Err(error) = register_shortcut(&app, &shortcut) {
        if let Some(old) = previous.as_deref() {
            let _ = register_shortcut(&app, old);
        }
        return Err(error);
    }
    state.mutate(|runtime| runtime.data.settings.shortcut = Some(shortcut))?;
    emit_snapshot(&app);
    Ok(state.snapshot())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsUpdate {
    autostart: bool,
    home_url: String,
    search_template: String,
    history_days: u32,
    lock_on_system_lock: bool,
    quick_links: Vec<QuickLink>,
    proxy_mode: String,
    proxy_url: String,
}

/// --proxy-server 接受 `scheme://host:port` 或裸 `host:port`。
fn validate_proxy_url(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if let Ok(url) = Url::parse(trimmed) {
        let scheme = url.scheme();
        if matches!(scheme, "http" | "https" | "socks4" | "socks5") && url.host_str().is_some() {
            return Ok(());
        }
        return Err("代理地址仅支持 http/https/socks4/socks5".into());
    }
    let without_scheme = trimmed.split_once("://").map_or(trimmed, |(_, rest)| rest);
    let host_part = without_scheme
        .rsplit('@')
        .next()
        .unwrap_or_default()
        .split(['/', '?'])
        .next()
        .unwrap_or_default();
    let (host, port) = host_part.rsplit_once(':').ok_or("代理地址格式无效")?;
    if host.is_empty() || port.parse::<u16>().is_err() {
        return Err("代理地址格式无效，示例：http://127.0.0.1:7890".into());
    }
    Ok(())
}

#[tauri::command]
async fn update_settings(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    update: SettingsUpdate,
) -> Result<AppSnapshot, String> {
    if !update.home_url.starts_with("http://") && !update.home_url.starts_with("https://") {
        return Err("主页必须是 HTTP 或 HTTPS 地址".into());
    }
    if !update.search_template.starts_with("https://")
        || !update.search_template.contains("{query}")
    {
        return Err("搜索地址必须使用 HTTPS，并包含 {query}".into());
    }
    if !(1..=3650).contains(&update.history_days) {
        return Err("历史保留天数必须在 1 到 3650 之间".into());
    }
    if update.quick_links.len() > 12 {
        return Err("快捷站点最多 12 个".into());
    }
    if !["system", "direct", "custom"].contains(&update.proxy_mode.as_str()) {
        return Err("代理模式无效".into());
    }
    if update.proxy_mode == "custom" {
        if update.proxy_url.trim().is_empty() {
            return Err("自定义代理模式必须填写代理地址".into());
        }
        validate_proxy_url(&update.proxy_url)?;
    }

    // 自启动仅在开关变化时操作注册表；disable 在条目不存在时不算错误。
    let autostart_current = state.mutate(|runtime| runtime.data.settings.autostart)?;
    if update.autostart != autostart_current {
        let result = if update.autostart {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        };
        if let Err(error) = result {
            let not_found = error.to_string().contains("os error 2");
            if update.autostart || !not_found {
                return Err(error.to_string());
            }
        }
    }
    // 代理是 WebView2 环境级参数，变更后需要重建全部标签 WebView 才生效。
    let proxy_changed = state.mutate(|runtime| {
        let settings = &mut runtime.data.settings;
        let changed = settings.proxy_mode != update.proxy_mode
            || settings.proxy_url.trim() != update.proxy_url.trim();
        settings.proxy_mode = update.proxy_mode.clone();
        settings.proxy_url = update.proxy_url.trim().to_string();
        changed
    })?;
    state.mutate(|runtime| {
        let settings = &mut runtime.data.settings;
        settings.autostart = update.autostart;
        settings.home_url = update.home_url;
        settings.search_template = update.search_template;
        settings.history_days = update.history_days;
        settings.lock_on_system_lock = update.lock_on_system_lock;
        settings.quick_links = update.quick_links;
    })?;
    if proxy_changed {
        recreate_tab_webviews(&app).await?;
    }
    emit_snapshot(&app);
    Ok(state.snapshot())
}

#[tauri::command]
fn list_extensions(app: AppHandle) -> Vec<extensions::ExtInfo> {
    extensions::list(&app)
}

/// 菜单用独立置顶子窗口承载：主 WebView 里的 HTML 浮层会被标签 WebView 盖住。
/// x/y 是按钮锚点相对主窗口客户区的逻辑坐标。
/// 必须是 async 命令：同步命令在主线程执行，在其间创建 WebView 窗口页面会加载失败
/// （create_tab 等异步命令创建的 WebView 均正常，见 docs/extensions.md 同类经验）。
#[tauri::command]
async fn show_menu_window(app: AppHandle, x: f64, y: f64) -> Result<(), String> {
    let main = app
        .get_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    let scale = main.scale_factor().map_err(|error| error.to_string())?;
    let origin = main
        .inner_position()
        .map_err(|error| error.to_string())?
        .to_logical::<f64>(scale);
    let position = LogicalPosition::new(origin.x + x, origin.y + y);

    if let Some(menu) = app.get_webview_window("menu") {
        let _ = menu.set_position(position);
        let _ = menu.show();
        let _ = menu.set_focus();
        return Ok(());
    }
    let menu = tauri::WebviewWindowBuilder::new(
        &app,
        "menu",
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("QuickPane 菜单")
    .inner_size(250.0, 300.0)
    .position(position.x, position.y)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .shadow(false)
    .focused(true)
    .build()
    .map_err(|error| error.to_string())?;
    // 创建后显式抢焦点：菜单内容加载期间焦点可能被主窗口的 WebView 抢回。
    let _ = menu.set_focus();
    Ok(())
}

#[tauri::command]
async fn install_extension(app: AppHandle) -> Result<Vec<extensions::ExtInfo>, String> {
    let folder = app
        .dialog()
        .file()
        .set_title("选择包含 manifest.json 的扩展文件夹")
        .blocking_pick_folder()
        .and_then(|path| path.into_path().ok());
    let Some(folder) = folder else {
        return Ok(extensions::list(&app));
    };
    let items = extensions::install_from_folder(&app, &folder)?;
    recreate_tab_webviews(&app).await?;
    Ok(items)
}

#[tauri::command]
async fn remove_extension(
    app: AppHandle,
    extension_id: String,
) -> Result<Vec<extensions::ExtInfo>, String> {
    let items = extensions::remove(&app, &extension_id)?;
    recreate_tab_webviews(&app).await?;
    Ok(items)
}

#[tauri::command]
async fn set_extension_enabled(
    app: AppHandle,
    extension_id: String,
    enabled: bool,
) -> Result<Vec<extensions::ExtInfo>, String> {
    let items = extensions::set_enabled(&app, &extension_id, enabled)?;
    recreate_tab_webviews(&app).await?;
    Ok(items)
}

/// 扩展面板小窗最近一次显示的时刻，用于忽略创建瞬间的焦点抖动。
static EXTENSION_POPUP_SHOWN_AT: std::sync::Mutex<Option<std::time::Instant>> =
    std::sync::Mutex::new(None);

/// 在置顶小窗中打开扩展面板（chrome-extension:// 页面）。
/// WebView2 没有工具栏弹出面板的宿主，用与标签页相同 WebView2 环境的独立窗口承载。
/// x/y 为触发图标相对主窗口客户区的逻辑坐标（导航栏扩展图标锚点），缺省时锚定右上角。
#[tauri::command]
async fn show_extension_popup(
    app: AppHandle,
    url: String,
    x: Option<f64>,
    y: Option<f64>,
) -> Result<(), String> {
    let parsed = Url::parse(&url).map_err(|_| "扩展面板地址无效".to_string())?;
    if parsed.scheme() != "chrome-extension" {
        return Err("仅支持扩展面板地址".into());
    }
    let main = app
        .get_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    let scale = main.scale_factor().map_err(|error| error.to_string())?;
    // 图标坐标相对客户区（inner），不能用含标题栏的 outer_position，否则面板偏移。
    let origin = main
        .inner_position()
        .map_err(|error| error.to_string())?
        .to_logical::<f64>(scale);
    let inner = main
        .inner_size()
        .map_err(|error| error.to_string())?
        .to_logical::<f64>(scale);
    // 有锚点时对齐图标下沿（面板右缘贴图标右缘），否则锚定右上角模拟工具栏弹出。
    let position = match (x, y) {
        (Some(x), Some(y)) => LogicalPosition::new(
            (origin.x + x + 24.0 - 400.0).max(origin.x + 8.0),
            origin.y + y + 6.0,
        ),
        _ => LogicalPosition::new(
            (origin.x + inner.width - 412.0).max(origin.x + 8.0),
            origin.y + 64.0,
        ),
    };

    if let Some(popup) = app.get_webview_window("extension-popup") {
        *EXTENSION_POPUP_SHOWN_AT.lock().expect("popup state poisoned") = Some(std::time::Instant::now());
        popup.navigate(parsed).map_err(|error| error.to_string())?;
        let _ = popup.set_position(position);
        let _ = popup.show();
        let _ = popup.set_focus();
        return Ok(());
    }

    // 必须与标签页 WebView 共享数据目录和环境参数（扩展、代理），否则 WebView2
    // 会因环境不一致而创建失败，且扩展在该窗口中不可用。
    let (data_dir, proxy_args) = {
        let state = app.state::<AppState>();
        let guard = state.inner.lock().map_err(|_| "应用状态无法读取")?;
        (
            browser_data_dir(&app)?,
            proxy_browser_args(&guard.data.settings.proxy_mode, &guard.data.settings.proxy_url),
        )
    };

    let mut builder =
        tauri::WebviewWindowBuilder::new(&app, "extension-popup", tauri::WebviewUrl::External(parsed))
            .title("扩展面板")
            .inner_size(400.0, 620.0)
            .position(position.x, position.y)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .focused(true)
            .data_directory(data_dir)
            .browser_extensions_enabled(true)
            .extensions_path(extensions_dir(&app));
    if let Some(args) = &proxy_args {
        builder = builder.additional_browser_args(args);
    }
    let popup = builder.build().map_err(|error| error.to_string())?;
    *EXTENSION_POPUP_SHOWN_AT.lock().expect("popup state poisoned") = Some(std::time::Instant::now());
    let _ = popup.set_focus();
    Ok(())
}

/// 固定/取消固定扩展到导航栏（仅持久化 id 列表，图标随快照刷新）。
#[tauri::command]
fn toggle_extension_pin(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    extension_id: String,
    pinned: bool,
) -> Result<AppSnapshot, String> {
    extensions::check_id(&extension_id)?;
    state.mutate(|runtime| {
        let pins = &mut runtime.data.settings.pinned_extensions;
        if pinned {
            if !pins.contains(&extension_id) {
                pins.push(extension_id);
            }
        } else {
            pins.retain(|id| id != &extension_id);
        }
    })?;
    emit_snapshot(&app);
    Ok(state.snapshot())
}

/// 检查更新后的待安装更新，由 install_update 消费。
#[derive(Default)]
pub struct UpdateState(std::sync::Mutex<Option<tauri_plugin_updater::Update>>);

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    version: String,
    notes: Option<String>,
    pub_date: Option<String>,
}

#[tauri::command]
async fn check_update(
    app: AppHandle,
    state: tauri::State<'_, UpdateState>,
) -> Result<Option<UpdateInfo>, String> {
    let update = app
        .updater()
        .map_err(|error| error.to_string())?
        .check()
        .await
        .map_err(|error| error.to_string())?;
    Ok(match update {
        Some(update) => {
            let info = UpdateInfo {
                version: update.version.clone(),
                notes: update.body.clone(),
                pub_date: update.date.map(|date| date.to_string()),
            };
                *state.0.lock().map_err(|_| "更新状态异常".to_string())? = Some(update);
            Some(info)
        }
        None => {
            *state.0.lock().map_err(|_| "更新状态异常".to_string())? = None;
            None
        },
    })
}

/// 下载并安装待安装更新，进度经 update-progress 事件推送，完成后应用自动重启。
#[tauri::command]
async fn install_update(
    app: AppHandle,
    state: tauri::State<'_, UpdateState>,
) -> Result<(), String> {
    let update = state
        .0
        .lock()
        .map_err(|_| "更新状态异常".to_string())?
        .clone()
        .ok_or_else(|| "没有待安装的更新，请先检查更新".to_string())?;
    let downloaded = AtomicU64::new(0);
    update
        .download_and_install(
            |chunk, total| {
                let downloaded = downloaded.fetch_add(chunk as u64, Ordering::Relaxed) + chunk as u64;
                let _ = app.emit(
                    "update-progress",
                    serde_json::json!({ "downloaded": downloaded, "total": total }),
                );
            },
            || {},
        )
        .await
        .map_err(|error| error.to_string())?;
    app.restart();
}

#[tauri::command]
fn add_bookmark(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    title: String,
    url: String,
) -> Result<AppSnapshot, String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("只能收藏网页地址".into());
    }
    state.mutate(|runtime| {
        if let Some(existing) = runtime
            .data
            .bookmarks
            .iter_mut()
            .find(|item| item.url == url)
        {
            existing.title = title;
        } else {
            runtime.data.bookmarks.insert(
                0,
                Bookmark {
                    id: Uuid::new_v4().simple().to_string(),
                    title,
                    url,
                    created_at: Utc::now(),
                },
            );
        }
    })?;
    emit_snapshot(&app);
    Ok(state.snapshot())
}

#[tauri::command]
fn remove_bookmark(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    bookmark_id: String,
) -> Result<AppSnapshot, String> {
    state.mutate(|runtime| runtime.data.bookmarks.retain(|item| item.id != bookmark_id))?;
    emit_snapshot(&app);
    Ok(state.snapshot())
}

#[tauri::command]
fn clear_history(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<AppSnapshot, String> {
    state.mutate(|runtime| runtime.data.history.clear())?;
    emit_snapshot(&app);
    Ok(state.snapshot())
}

#[tauri::command]
fn clear_downloads(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<AppSnapshot, String> {
    state.mutate(|runtime| runtime.data.downloads.clear())?;
    emit_snapshot(&app);
    Ok(state.snapshot())
}

#[tauri::command]
fn open_download(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_site_data(app: AppHandle) -> Result<(), String> {
    if let Some(webview) = app
        .webviews()
        .values()
        .find(|webview| webview.label().starts_with("tab-"))
    {
        webview
            .clear_all_browsing_data()
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn set_app_password(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    current_password: Option<String>,
    new_password: String,
) -> Result<AppSnapshot, String> {
    let has_password = state
        .inner
        .lock()
        .map_err(|_| "应用状态无法读取".to_string())?
        .data
        .settings
        .password_hash
        .is_some();
    if has_password
        && current_password
            .as_deref()
            .is_none_or(|password| !state.verify_password(password))
    {
        return Err("当前应用密码不正确".into());
    }
    let hash = AppState::hash_password(&new_password)?;
    state.mutate(|runtime| {
        runtime.data.settings.password_hash = Some(hash);
        runtime.locked = false;
        runtime.first_run = false;
    })?;
    emit_snapshot(&app);
    Ok(state.snapshot())
}

#[tauri::command]
fn disable_app_password(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    current_password: String,
) -> Result<AppSnapshot, String> {
    if !state.verify_password(&current_password) {
        return Err("当前应用密码不正确".into());
    }
    state.mutate(|runtime| {
        runtime.data.settings.password_hash = None;
        runtime.locked = false;
    })?;
    emit_snapshot(&app);
    Ok(state.snapshot())
}

#[tauri::command]
async fn unlock_app(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    password: String,
) -> Result<AppSnapshot, String> {
    if !state.verify_password(&password) {
        return Err("应用密码不正确".into());
    }
    let active = state.mutate(|runtime| {
        runtime.locked = false;
        runtime.first_run = false;
        runtime.shell_mode = runtime
            .data
            .active_tab_id
            .as_ref()
            .and_then(|id| runtime.data.tabs.iter().find(|tab| &tab.id == id))
            .is_none_or(|tab| tab.url == "quickpane://newtab");
        runtime.data.active_tab_id.clone()
    })?;
    if let Some(id) = active {
        ensure_tab_webview(&app, &id).await?;
    }
    browser::show_active_tab(&app);
    emit_snapshot(&app);
    Ok(state.snapshot())
}

#[tauri::command]
fn skip_password_setup(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<AppSnapshot, String> {
    state.mutate(|runtime| {
        runtime.first_run = false;
        runtime.locked = false;
    })?;
    emit_snapshot(&app);
    Ok(state.snapshot())
}

#[tauri::command]
fn lock_now(app: AppHandle) {
    lock_app(&app);
}

fn register_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        get_snapshot,
        new_tab,
        select_tab,
        remove_tab,
        navigate,
        reload,
        go_back,
        go_forward,
        find_in_page,
        zoom_page,
        show_shell,
        hide_to_tray,
        show_from_tray,
        exit_app,
        set_global_shortcut,
        update_settings,
        add_bookmark,
        remove_bookmark,
        clear_history,
        clear_downloads,
        open_download,
        clear_site_data,
        set_app_password,
        disable_app_password,
        unlock_app,
        skip_password_setup,
        lock_now,
        list_extensions,
        install_extension,
        remove_extension,
        set_extension_enabled,
        show_menu_window,
        show_extension_popup,
        toggle_extension_pin,
        check_update,
        install_update
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_window(app);
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--autostart")
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build());

    register_commands(builder)
        .setup(|app| {
            let handle = app.handle().clone();
            let state = AppState::load(&handle).map_err(std::io::Error::other)?;
            let configured_shortcut = state
                .inner
                .lock()
                .ok()
                .and_then(|runtime| runtime.data.settings.shortcut.clone());
            app.manage(state);
            app.manage(UpdateState::default());

            install_tray(&handle).map_err(std::io::Error::other)?;
            install_session_lock_listener(&handle).map_err(std::io::Error::other)?;
            if let Some(shortcut) = configured_shortcut {
                if let Err(error) = register_shortcut(&handle, &shortcut) {
                    let _ = handle.emit("shortcut-error", error);
                }
            }

            let from_autostart = std::env::args().any(|arg| arg == "--autostart");
            if from_autostart {
                hide_window(&handle);
            }

            let timer_app = handle.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(60));
                freeze_idle_tabs(&timer_app);
            });
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                let quitting = window
                    .app_handle()
                    .state::<AppState>()
                    .inner
                    .lock()
                    .map(|runtime| runtime.quitting)
                    .unwrap_or(false);
                if !quitting {
                    api.prevent_close();
                    hide_window(window.app_handle());
                }
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                resize_tabs(window.app_handle());
            }
            // 扩展面板小窗：点击外部（失焦）自动收起；忽略创建瞬间的焦点抖动。
            WindowEvent::Focused(false) if window.label() == "extension-popup" => {
                let settled = EXTENSION_POPUP_SHOWN_AT
                    .lock()
                    .ok()
                    .and_then(|at| at.map(|instant| instant.elapsed().as_millis() > 400))
                    .unwrap_or(true);
                if settled {
                    let _ = window.hide();
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running QuickPane");
}
