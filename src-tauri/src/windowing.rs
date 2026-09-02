use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::{
    browser::{emit_snapshot, hide_all_tabs, set_all_muted, show_active_tab},
    state::AppState,
};

pub fn install_tray(app: &AppHandle) -> Result<(), String> {
    let toggle = MenuItem::with_id(app, "toggle", "显示 / 隐藏", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let quit = MenuItem::with_id(app, "quit", "退出 QuickPane", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let menu =
        Menu::with_items(app, &[&toggle, &settings, &quit]).map_err(|error| error.to_string())?;

    let icon = app.default_window_icon().cloned();
    TrayIconBuilder::with_id("quickpane-tray")
        .tooltip("QuickPane")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => toggle_window(app),
            "settings" => {
                show_window(app);
                let state = app.state::<AppState>();
                let _ = state.mutate(|runtime| runtime.shell_mode = true);
                hide_all_tabs(app);
                let _ = app.emit("open-section", "settings");
                emit_snapshot(app);
            }
            "quit" => quit_app(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                toggle_window(tray.app_handle());
            }
        })
        .icon(icon.ok_or_else(|| "缺少应用图标".to_string())?)
        .build(app)
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn register_shortcut(app: &AppHandle, shortcut: &str) -> Result<(), String> {
    validate_shortcut(shortcut)?;
    app.global_shortcut()
        .unregister_all()
        .map_err(|error| error.to_string())?;
    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                toggle_window(app);
            }
        })
        .map_err(|error| format!("无法注册快捷键，可能已被其他软件占用：{error}"))
}

pub fn validate_shortcut(shortcut: &str) -> Result<(), String> {
    let normalized = shortcut.to_ascii_lowercase();
    let has_modifier = ["ctrl", "control", "alt", "shift", "super", "meta"]
        .iter()
        .any(|modifier| normalized.contains(modifier));
    let parts = normalized
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if !has_modifier || parts.len() < 2 {
        return Err("快捷键至少需要一个修饰键和一个普通按键".into());
    }
    shortcut
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map(|_| ())
        .map_err(|_| "快捷键格式无效".into())
}

pub fn toggle_window(app: &AppHandle) {
    let visible = app
        .state::<AppState>()
        .inner
        .lock()
        .map(|runtime| runtime.window_visible)
        .unwrap_or(false);
    if visible {
        hide_window(app);
    } else {
        show_window(app);
    }
}

pub fn hide_window(app: &AppHandle) {
    let Some(window) = app.get_window("main") else {
        return;
    };
    let current = capture_foreground_window(window.hwnd().ok().map(|value| value.0 as isize));
    let previous = if current == 0 {
        app.state::<AppState>()
            .inner
            .lock()
            .map(|runtime| runtime.previous_window)
            .unwrap_or_default()
    } else {
        current
    };
    set_all_muted(app, true);
    let _ = window.set_fullscreen(false);
    let _ = window.hide();
    // 菜单弹层窗口跟随主窗口一起收起，避免悬浮残留。
    if let Some(menu) = app.get_webview_window("menu") {
        let _ = menu.hide();
    }
    if let Some(popup) = app.get_webview_window("extension-popup") {
        let _ = popup.hide();
    }
    if let Some(state) = app.try_state::<AppState>() {
        let _ = state.mutate(|runtime| {
            runtime.window_visible = false;
            if previous != 0 {
                runtime.previous_window = previous;
            }
        });
    }
    restore_foreground_window(previous);
    emit_snapshot(app);
}

pub fn show_window(app: &AppHandle) {
    let Some(window) = app.get_window("main") else {
        return;
    };
    let was_visible = app
        .try_state::<AppState>()
        .and_then(|state| {
            state
                .inner
                .lock()
                .ok()
                .map(|runtime| runtime.window_visible)
        })
        .unwrap_or(false);
    let shell_mode = app
        .try_state::<AppState>()
        .and_then(|state| state.inner.lock().ok().map(|runtime| runtime.shell_mode))
        .unwrap_or(true);
    let locked = app
        .try_state::<AppState>()
        .and_then(|state| {
            state
                .inner
                .lock()
                .ok()
                .map(|runtime| runtime.locked || runtime.first_run)
        })
        .unwrap_or(true);
    let previous = capture_foreground_window(window.hwnd().ok().map(|value| value.0 as isize));
    if let Some(state) = app.try_state::<AppState>() {
        let _ = state.mutate(|runtime| {
            runtime.window_visible = true;
            if previous != 0 {
                runtime.previous_window = previous;
            }
        });
    }
    let _ = window.show();
    let _ = window.unminimize();
    force_foreground(&window);
    if locked {
        // 锁屏界面可见时，后台标签仍必须保持静音。
        hide_all_tabs(app);
        set_all_muted(app, true);
    } else {
        set_all_muted(app, false);
        show_active_tab(app);
    }
    if !was_visible && shell_mode && !locked {
        // 仅壳层页面需要地址栏焦点；浏览页面时由 show_active_tab 聚焦子 WebView。
        if let Some(webview) = app.get_webview("main") {
            let _ = webview.set_focus();
        }
        let _ = app.emit("focus-address", ());
    }
    emit_snapshot(app);
}

pub fn lock_app(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        let should_lock = state
            .inner
            .lock()
            .map(|runtime| runtime.data.settings.password_hash.is_some())
            .unwrap_or(false);
        if should_lock {
            let _ = state.mutate(|runtime| {
                runtime.locked = true;
                runtime.shell_mode = true;
            });
            hide_all_tabs(app);
            set_all_muted(app, true);
            if let Some(menu) = app.get_webview_window("menu") {
                let _ = menu.hide();
            }
            if let Some(popup) = app.get_webview_window("extension-popup") {
                let _ = popup.hide();
            }
            let _ = app.emit("open-section", "lock");
            emit_snapshot(app);
        }
    }
}

pub fn lock_on_system_lock(app: &AppHandle) {
    let enabled = app
        .try_state::<AppState>()
        .and_then(|state| {
            state
                .inner
                .lock()
                .ok()
                .map(|runtime| runtime.data.settings.lock_on_system_lock)
        })
        .unwrap_or(false);
    if enabled {
        lock_app(app);
    }
}

pub fn quit_app(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        let _ = state.mutate(|runtime| runtime.quitting = true);
    }
    app.exit(0);
}

#[cfg(windows)]
pub fn install_session_lock_listener(app: &AppHandle) -> Result<(), String> {
    use windows::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::RemoteDesktop::{WTSRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION},
        UI::WindowsAndMessaging::{
            CallWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC, WM_WTSSESSION_CHANGE, WNDPROC,
        },
    };

    static APP_HANDLE: std::sync::OnceLock<AppHandle> = std::sync::OnceLock::new();
    static PREVIOUS_PROC: std::sync::OnceLock<isize> = std::sync::OnceLock::new();

    unsafe extern "system" fn session_wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_WTSSESSION_CHANGE && wparam.0 as u32 == 0x7 {
            if let Some(app) = APP_HANDLE.get() {
                lock_on_system_lock(app);
            }
        }
        let previous = PREVIOUS_PROC.get().copied().unwrap_or_default();
        if previous == 0 {
            LRESULT(0)
        } else {
            let proc: WNDPROC = Some(std::mem::transmute(previous));
            CallWindowProcW(proc, hwnd, msg, wparam, lparam)
        }
    }

    let window = app
        .get_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    let native = windows::Win32::Foundation::HWND(hwnd.0 as *mut _);
    APP_HANDLE.set(app.clone()).ok();
    let previous = unsafe {
        SetWindowLongPtrW(
            native,
            GWLP_WNDPROC,
            session_wnd_proc as *const () as usize as isize,
        )
    };
    PREVIOUS_PROC.set(previous).ok();
    unsafe { WTSRegisterSessionNotification(native, NOTIFY_FOR_THIS_SESSION) }
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(not(windows))]
pub fn install_session_lock_listener(_app: &AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn capture_foreground_window(own: Option<isize>) -> isize {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    let hwnd = unsafe { GetForegroundWindow() }.0 as isize;
    if own == Some(hwnd) {
        0
    } else {
        hwnd
    }
}

#[cfg(not(windows))]
fn capture_foreground_window(_own: Option<isize>) -> isize {
    0
}

#[cfg(windows)]
fn restore_foreground_window(hwnd: isize) {
    if hwnd == 0 {
        return;
    }
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            IsIconic, IsWindow, SetForegroundWindow, ShowWindow, SW_RESTORE,
        },
    };
    let native = HWND(hwnd as *mut _);
    unsafe {
        if IsWindow(Some(native)).as_bool() {
            // 仅在最小化时才 RESTORE；SW_RESTORE 会把最大化窗口还原成普通大小。
            if IsIconic(native).as_bool() {
                let _ = ShowWindow(native, SW_RESTORE);
            }
            let _ = SetForegroundWindow(native);
        }
    }
}

#[cfg(not(windows))]
fn restore_foreground_window(_hwnd: isize) {}

#[cfg(windows)]
fn force_foreground(window: &tauri::Window) {
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{IsIconic, SetForegroundWindow, ShowWindow, SW_RESTORE},
    };
    if let Ok(hwnd) = window.hwnd() {
        let native = HWND(hwnd.0 as *mut _);
        unsafe {
            if IsIconic(native).as_bool() {
                let _ = ShowWindow(native, SW_RESTORE);
            }
            let _ = SetForegroundWindow(native);
        }
    }
    let _ = window.set_focus();
}

#[cfg(not(windows))]
fn force_foreground(window: &tauri::Window) {
    let _ = window.set_focus();
}
