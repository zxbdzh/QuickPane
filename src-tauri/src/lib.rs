mod browser;
mod state;
mod windowing;

use std::time::Duration;

use browser::{
    activate_tab, close_tab, create_tab, emit_snapshot, ensure_tab_webview, eval_active,
    freeze_idle_tabs, navigate_tab, reload_active, resize_tabs, set_shell_mode, set_zoom,
};
use chrono::Utc;
use serde::Deserialize;
use state::{AppSnapshot, AppState, Bookmark, QuickLink};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;
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
}

#[tauri::command]
fn update_settings(
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

    if update.autostart {
        app.autolaunch()
            .enable()
            .map_err(|error| error.to_string())?;
    } else {
        app.autolaunch()
            .disable()
            .map_err(|error| error.to_string())?;
    }
    state.mutate(|runtime| {
        let settings = &mut runtime.data.settings;
        settings.autostart = update.autostart;
        settings.home_url = update.home_url;
        settings.search_template = update.search_template;
        settings.history_days = update.history_days;
        settings.lock_on_system_lock = update.lock_on_system_lock;
        settings.quick_links = update.quick_links;
    })?;
    emit_snapshot(&app);
    Ok(state.snapshot())
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
        lock_now
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
        .plugin(tauri_plugin_opener::init());

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
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running QuickPane");
}
