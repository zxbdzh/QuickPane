use std::path::{Path, PathBuf};

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

use base64::Engine;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::browser::extensions_dir;

pub const BUILTIN_ID: &str = "violentmonkey";

/// 仅首次安装随包版本；尊重用户禁用或卸载，避免每次启动重新安装。
pub fn install_builtin(app: &AppHandle) -> Result<bool, String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let marker = root.join("violentmonkey-installed");
    if marker.exists() {
        return Ok(false);
    }
    let target = ext_dir(app, BUILTIN_ID)?;
    if !target.exists() && !ext_disabled_dir(app, BUILTIN_ID)?.exists() {
        let source = app
            .path()
            .resolve(
                "resources/violentmonkey",
                tauri::path::BaseDirectory::Resource,
            )
            .map_err(|error| error.to_string())?;
        if read_manifest(&source).is_none() {
            return Err("内置油猴资源缺失".into());
        }
        copy_dir(&source, &target)?;
    }
    std::fs::write(marker, "2.49.0").map_err(|error| error.to_string())?;
    Ok(true)
}

fn host_window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.webviews()
        .keys()
        .find(|label| label.starts_with("extension-host-"))
        .and_then(|label| app.get_webview_window(label))
}

/// 没有网页时也能检查扩展；宿主不显示、不承载远程网页。
pub async fn ensure_loaded(app: &AppHandle) -> Result<(), String> {
    static HOST_LOCK: tauri::async_runtime::Mutex<()> = tauri::async_runtime::Mutex::const_new(());
    let _guard = HOST_LOCK.lock().await;
    let host = match host_window(app) {
        Some(host) => host,
        None => {
            let state = app.state::<crate::state::AppState>();
            let args = {
                let guard = state.inner.lock().map_err(|_| "应用状态无法读取")?;
                crate::browser::proxy_browser_args(
                    &guard.data.settings.proxy_mode,
                    &guard.data.settings.proxy_url,
                )
            };
            let mut builder = tauri::WebviewWindowBuilder::new(
                app,
                format!("extension-host-{}", Uuid::new_v4().simple()),
                tauri::WebviewUrl::External("about:blank".parse().unwrap()),
            )
            .visible(false)
            .focused(false)
            .skip_taskbar(true)
            // 标题不能用 Tauri 默认值；旧版窗口状态插件曾把它恢复成可见窗口。
            .title("QuickPane Extension Host")
            .data_directory(crate::browser::browser_data_dir(app)?)
            .browser_extensions_enabled(true);
            if let Some(args) = args {
                builder = builder.additional_browser_args(&args);
            }
            builder.build().map_err(|error| error.to_string())?
        }
    };
    load_profile(app, host.as_ref()).await
}

/// 只打开已安装扩展声明的页面；不信任前端拼接的 chrome-extension 地址。
pub fn page_url(app: &AppHandle, id: &str, options: bool) -> Result<url::Url, String> {
    let info = list(app)
        .into_iter()
        .find(|info| info.id == id && info.enabled)
        .ok_or("扩展未启用")?;
    if let Some(error) = info.load_error {
        return Err(format!("扩展加载失败：{error}"));
    }
    let url = if options {
        info.options_url
    } else {
        info.popup_url
    }
    .ok_or("扩展未提供该页面")?;
    url.parse().map_err(|_| "扩展页面地址无效".into())
}

/// WebView2 返回的运行时 ID 才能用于导航；目录哈希只用作本地安装标识。
#[derive(Clone)]
struct LoadedExtension {
    id: String,
    error: Option<String>,
}
static LOADED: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, LoadedExtension>>,
> = std::sync::LazyLock::new(Default::default);

#[cfg(windows)]
pub async fn load_profile(app: &AppHandle, webview: &tauri::Webview) -> Result<(), String> {
    use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2Profile7, ICoreWebView2_13};
    use webview2_com::{take_pwstr, ProfileAddBrowserExtensionCompletedHandler};
    use windows::core::{Interface, HSTRING, PWSTR};

    static LOAD_LOCK: tauri::async_runtime::Mutex<()> = tauri::async_runtime::Mutex::const_new(());
    let _guard = LOAD_LOCK.lock().await;
    for info in list(app).into_iter().filter(|info| info.enabled) {
        if LOADED
            .lock()
            .map_err(|_| "扩展状态无法读取")?
            .get(&info.id)
            .is_some_and(|value| value.error.is_none())
        {
            continue;
        }
        let path = ext_dir(app, &info.id)?;
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        webview
            .with_webview(move |platform| {
                let result = (|| -> windows::core::Result<()> {
                    // 回调必须留在 UI 线程，异步只传递普通字符串，不跨线程传 COM 对象。
                    unsafe {
                        let core = platform
                            .controller()
                            .CoreWebView2()?
                            .cast::<ICoreWebView2_13>()?;
                        let profile = core.Profile()?.cast::<ICoreWebView2Profile7>()?;
                        let callback_sender = sender.clone();
                        let handler = ProfileAddBrowserExtensionCompletedHandler::create(Box::new(
                            move |error, extension| {
                                let result = error
                                    .and_then(|_| {
                                        let extension = extension.ok_or_else(|| {
                                            windows::core::Error::from_hresult(
                                                windows::core::HRESULT(0x80004005u32 as i32),
                                            )
                                        })?;
                                        let mut id = PWSTR::null();
                                        extension.Id(&mut id)?;
                                        Ok(take_pwstr(id))
                                    })
                                    .map_err(|error| error.to_string());
                                let _ = callback_sender.try_send(result);
                                Ok(())
                            },
                        ));
                        profile.AddBrowserExtension(&HSTRING::from(path.as_path()), &handler)?;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    let _ = sender.try_send(Err(error.to_string()));
                }
            })
            .map_err(|error| error.to_string())?;
        let result = tauri::async_runtime::spawn_blocking(move || {
            receiver.recv_timeout(std::time::Duration::from_secs(15))
        })
        .await
        .map_err(|error| error.to_string())?
        .map_err(|_| "扩展加载超时，请重新检查")?;
        let loaded = match result {
            Ok(id) => LoadedExtension { id, error: None },
            Err(error) => LoadedExtension {
                id: String::new(),
                error: Some(error),
            },
        };
        LOADED
            .lock()
            .map_err(|_| "扩展状态无法读取")?
            .insert(info.id, loaded);
    }
    crate::browser::emit_snapshot(app);
    Ok(())
}
/// 目录移动不会停止已注册扩展，还会改变无 key 扩展的 ID；启停必须调用原生 Profile。
pub async fn update_runtime(
    app: &AppHandle,
    id: &str,
    enabled: Option<bool>,
) -> Result<(), String> {
    use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2Profile7, ICoreWebView2_13};
    use webview2_com::{
        take_pwstr, BrowserExtensionEnableCompletedHandler, BrowserExtensionRemoveCompletedHandler,
        ProfileGetBrowserExtensionsCompletedHandler,
    };
    use windows::core::{Interface, PWSTR};
    ensure_loaded(app).await?;
    let runtime_id = LOADED
        .lock()
        .map_err(|_| "扩展状态无法读取")?
        .get(id)
        .map(|value| value.id.clone())
        .unwrap_or_default();
    if runtime_id.is_empty() {
        return Ok(());
    }
    let host = host_window(app).ok_or("扩展宿主不存在")?;
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    host.with_webview(move |platform| {
        let result = (|| -> windows::core::Result<()> {
            unsafe {
                let profile = platform
                    .controller()
                    .CoreWebView2()?
                    .cast::<ICoreWebView2_13>()?
                    .Profile()?
                    .cast::<ICoreWebView2Profile7>()?;
                let callback_sender = sender.clone();
                let handler = ProfileGetBrowserExtensionsCompletedHandler::create(Box::new(
                    move |error, list| {
                        let result = (|| -> windows::core::Result<()> {
                            error?;
                            let list = list.ok_or_else(windows::core::Error::empty)?;
                            let mut count = 0;
                            list.Count(&mut count)?;
                            for index in 0..count {
                                let extension = list.GetValueAtIndex(index)?;
                                let mut actual = PWSTR::null();
                                extension.Id(&mut actual)?;
                                if take_pwstr(actual) != runtime_id {
                                    continue;
                                }
                                let tx = callback_sender.clone();
                                if let Some(enabled) = enabled {
                                    let handler = BrowserExtensionEnableCompletedHandler::create(
                                        Box::new(move |result| {
                                            let _ = tx.try_send(result.map_err(|e| e.to_string()));
                                            Ok(())
                                        }),
                                    );
                                    extension.Enable(enabled, &handler)?;
                                } else {
                                    let handler = BrowserExtensionRemoveCompletedHandler::create(
                                        Box::new(move |result| {
                                            let _ = tx.try_send(result.map_err(|e| e.to_string()));
                                            Ok(())
                                        }),
                                    );
                                    extension.Remove(&handler)?;
                                }
                                return Ok(());
                            }
                            let _ = callback_sender.try_send(Ok(()));
                            Ok(())
                        })();
                        if let Err(error) = result {
                            let _ = callback_sender.try_send(Err(error.to_string()));
                        }
                        Ok(())
                    },
                ));
                profile.GetBrowserExtensions(&handler)?;
                Ok(())
            }
        })();
        if let Err(error) = result {
            let _ = sender.try_send(Err(error.to_string()));
        }
    })
    .map_err(|error| error.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        receiver.recv_timeout(std::time::Duration::from_secs(15))
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|_| "扩展操作超时，请重试")?
}

/// 已安装扩展的展示信息，从磁盘上的 manifest.json 实时解析。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtInfo {
    /// 扩展所在文件夹名，同时作为管理用的稳定 id。
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    /// 图标的 data URL（避免前端依赖 asset protocol），解析失败为 None。
    pub icon: Option<String>,
    /// 已由 WebView2 确认的面板地址。
    pub popup_url: Option<String>,
    pub load_error: Option<String>,
    pub options_url: Option<String>,
    pub builtin: bool,
    pub loaded: bool,
}

/// 按 Chromium 规则推导扩展 ID：manifest 带 `key`（商店来源解包后常见）时用
/// 公钥 DER 的 SHA-256 前 16 字节；否则回退到路径哈希（UTF-16LE 字节）。
/// 半字节统一映射为 a..p。
fn compute_extension_id(path: &Path, manifest: &serde_json::Value) -> String {
    if let Some(key) = manifest.get("key").and_then(|v| v.as_str()) {
        if let Ok(der) = base64::engine::general_purpose::STANDARD.decode(key.trim()) {
            return to_extension_id(&Sha256::digest(der)[..16]);
        }
    }
    #[cfg(windows)]
    let bytes: Vec<u8> = path
        .as_os_str()
        .encode_wide()
        .flat_map(|unit| unit.to_le_bytes())
        .collect();
    #[cfg(not(windows))]
    let bytes: Vec<u8> = path.as_os_str().as_encoded_bytes().to_vec();
    to_extension_id(&Sha256::digest(&bytes)[..16])
}

fn to_extension_id(bytes: &[u8]) -> String {
    let mut id = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        for nibble in [byte >> 4, byte & 0x0F] {
            id.push((b'a' + nibble) as char);
        }
    }
    id
}

/// 从 manifest 提取弹出面板 URL（MV3 action / MV2 browser_action）。
fn manifest_popup(extension_id: &str, manifest: &serde_json::Value) -> Option<String> {
    let action = manifest
        .get("action")
        .or_else(|| manifest.get("browser_action"))?;
    let popup = action.get("default_popup")?.as_str()?;
    extension_page(extension_id, popup)
}
fn extension_page(id: &str, path: &str) -> Option<String> {
    let base = url::Url::parse(&format!("chrome-extension://{id}/")).ok()?;
    let url = base.join(path).ok()?;
    (url.scheme() == "chrome-extension" && url.host_str() == Some(id)).then(|| url.to_string())
}

/// 禁用标记与旧版隔离目录；保持兼容已安装扩展。
fn disabled_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("ExtensionsDisabled");
    std::fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

fn ext_dir(app: &AppHandle, id: &str) -> Result<PathBuf, String> {
    sanitize_id(id)?;
    Ok(extensions_dir(app).join(id))
}

fn ext_disabled_dir(app: &AppHandle, id: &str) -> Result<PathBuf, String> {
    sanitize_id(id)?;
    Ok(disabled_dir(app)?.join(id))
}

/// id 只能是单段路径名，防止借 id 穿越目录。
fn sanitize_id(id: &str) -> Result<(), String> {
    let valid = !id.is_empty()
        && !id.starts_with('.')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ' '));
    if valid {
        Ok(())
    } else {
        Err("扩展 id 无效".into())
    }
}

/// 校验扩展目录 id（单段路径名），供固定列表等外部使用。
pub fn check_id(id: &str) -> Result<(), String> {
    sanitize_id(id)
}

/// 读取单个扩展目录的展示信息；目录不存在或 manifest 非法时返回 None。
fn info_from_dir(path: &Path, enabled: bool) -> Option<ExtInfo> {
    if !path.is_dir() {
        return None;
    }
    if is_reparse_or_link(path).ok()? {
        return None;
    }
    let manifest = read_manifest(path)?;
    let id = path.file_name()?.to_string_lossy().to_string();
    let enabled = enabled
        && !path
            .parent()?
            .parent()?
            .join("ExtensionsDisabled")
            .join(format!("{id}.disabled"))
            .exists();
    let loaded = LOADED.lock().ok()?.get(&id).cloned();
    let extension_id = loaded
        .as_ref()
        .map(|loaded| loaded.id.clone())
        .unwrap_or_else(|| compute_extension_id(path, &manifest));
    Some(ExtInfo {
        name: resolve_message(path, manifest.get("name")?.as_str()?).unwrap_or_else(|| id.clone()),
        version: manifest
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string(),
        description: manifest
            .get("description")
            .and_then(|v| v.as_str())
            .and_then(|raw| resolve_message(path, raw))
            .unwrap_or_default(),
        icon: manifest_icon(path, &manifest),
        popup_url: manifest_popup(&extension_id, &manifest),
        options_url: manifest
            .get("options_ui")
            .and_then(|value| value.get("page"))
            .or_else(|| manifest.get("options_page"))
            .and_then(|value| value.as_str())
            .and_then(|path| extension_page(&extension_id, path)),
        builtin: id == BUILTIN_ID,
        loaded: enabled && loaded.as_ref().is_some_and(|value| value.error.is_none()),
        load_error: loaded.and_then(|loaded| loaded.error),
        id,
        enabled,
    })
}

pub fn list(app: &AppHandle) -> Vec<ExtInfo> {
    let dir = extensions_dir(app);
    let Ok(disabled) = disabled_dir(app) else {
        return Vec::new();
    };
    let mut items = read_installed(&dir, true);
    items.extend(read_installed(&disabled, false));
    items.sort_by_key(|a| a.name.to_lowercase());
    items
}

/// 快照用：按固定列表读取启用中的扩展信息（不存在的 id 跳过）。
pub fn pinned_infos(data_dir: &Path, ids: &[String]) -> Vec<ExtInfo> {
    ids.iter()
        .filter(|id| sanitize_id(id).is_ok())
        .filter_map(|id| info_from_dir(&data_dir.join("Extensions").join(id), true))
        .filter(|info| info.enabled)
        .collect()
}

fn read_installed(dir: &Path, enabled: bool) -> Vec<ExtInfo> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut items = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let internal = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('.'));
        if internal || !path.is_dir() {
            continue;
        }
        if let Some(info) = info_from_dir(&path, enabled) {
            items.push(info);
        }
    }
    items
}

fn read_manifest(dir: &Path) -> Option<serde_json::Value> {
    let contents = std::fs::read_to_string(dir.join("manifest.json")).ok()?;
    serde_json::from_str(&contents).ok()
}

/// 只接受 Chromium 支持的单段 locale 名称，避免把 manifest 值当作路径。
fn is_safe_locale(locale: &str) -> bool {
    !locale.is_empty()
        && locale.len() <= 64
        && locale
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

fn read_locale_messages(dir: &Path, locale: &str) -> Option<String> {
    if !is_safe_locale(locale) {
        return None;
    }
    let root = std::fs::canonicalize(dir).ok()?;
    let candidate = dir.join("_locales").join(locale).join("messages.json");
    let canonical = std::fs::canonicalize(&candidate).ok()?;
    if !canonical.starts_with(&root) || is_reparse_or_link(&candidate).ok()? {
        return None;
    }
    std::fs::read_to_string(canonical).ok()
}

/// 解析 manifest 字段的 `__MSG_key__` 国际化写法，非该格式时原样返回。
fn resolve_message(dir: &Path, raw: &str) -> Option<String> {
    let Some(key) = raw
        .strip_prefix("__MSG_")
        .and_then(|rest| rest.strip_suffix("__"))
    else {
        return Some(raw.to_string());
    };
    let manifest = read_manifest(dir)?;
    let locale = manifest
        .get("default_locale")
        .and_then(|v| v.as_str())
        .unwrap_or("en");
    let messages = read_locale_messages(dir, locale).or_else(|| read_locale_messages(dir, "en"))?;
    let parsed: serde_json::Value = serde_json::from_str(&messages).ok()?;
    parsed
        .get(key)
        .and_then(|entry| entry.get("message"))
        .and_then(|message| message.as_str())
        .map(str::to_string)
}

const MAX_ICON_BYTES: u64 = 2 * 1024 * 1024;
const MAX_COPY_DEPTH: usize = 32;
const MAX_COPY_FILES: usize = 10_000;
const MAX_COPY_BYTES: u64 = 512 * 1024 * 1024;

fn is_reparse_or_link(path: &Path) -> Result<bool, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
        return Ok(true);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        Ok(metadata.file_attributes() & 0x400 != 0)
    }
    #[cfg(not(windows))]
    Ok(false)
}

fn manifest_icon(dir: &Path, manifest: &serde_json::Value) -> Option<String> {
    let icons = manifest.get("icons")?.as_object()?;
    let relative = ["256", "128", "48", "16"]
        .iter()
        .find_map(|size| icons.get(*size))
        .or_else(|| icons.values().next())?
        .as_str()?;
    let relative = relative.trim();
    let relative_path = Path::new(relative);
    if relative.is_empty()
        || relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }
    let root = std::fs::canonicalize(dir).ok()?;
    let icon_path = dir.join(relative);
    if is_reparse_or_link(&icon_path).ok()? {
        return None;
    }
    let canonical = std::fs::canonicalize(&icon_path).ok()?;
    if !canonical.starts_with(&root) || std::fs::metadata(&canonical).ok()?.len() > MAX_ICON_BYTES {
        return None;
    }
    let bytes = std::fs::read(&canonical).ok()?;
    let mime = match icon_path
        .extension()?
        .to_str()?
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => return None,
    };
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Some(format!("data:{mime};base64,{encoded}"))
}

fn copy_dir(source: &Path, target: &Path) -> Result<(), String> {
    fn copy(
        source: &Path,
        target: &Path,
        depth: usize,
        files: &mut usize,
        bytes: &mut u64,
    ) -> Result<(), String> {
        if depth > MAX_COPY_DEPTH || is_reparse_or_link(source)? {
            return Err("扩展包含不受支持的链接或目录联接".into());
        }
        std::fs::create_dir_all(target).map_err(|error| error.to_string())?;
        for entry in std::fs::read_dir(source).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let next_source = entry.path();
            let next_target = target.join(entry.file_name());
            if is_reparse_or_link(&next_source)? {
                return Err("扩展包含不受支持的链接或目录联接".into());
            }
            let metadata =
                std::fs::symlink_metadata(&next_source).map_err(|error| error.to_string())?;
            if metadata.is_dir() {
                copy(&next_source, &next_target, depth + 1, files, bytes)?;
            } else if metadata.is_file() {
                *files += 1;
                *bytes = bytes.saturating_add(metadata.len());
                if *files > MAX_COPY_FILES || *bytes > MAX_COPY_BYTES {
                    return Err("扩展文件数量或大小超出限制".into());
                }
                std::fs::copy(&next_source, &next_target).map_err(|error| error.to_string())?;
            } else {
                return Err("扩展包含不受支持的文件类型".into());
            }
        }
        Ok(())
    }
    let result = copy(source, target, 0, &mut 0, &mut 0);
    if result.is_err() {
        let _ = std::fs::remove_dir_all(target);
    }
    result
}

/// 校验并把选中的未打包扩展复制进扩展目录；不负责 WebView 重建。
pub fn install_from_folder(app: &AppHandle, folder: &Path) -> Result<Vec<ExtInfo>, String> {
    if !folder.join("manifest.json").is_file() {
        return Err("所选文件夹不是有效的扩展：缺少 manifest.json".into());
    }
    let manifest = read_manifest(folder).ok_or("manifest.json 无法解析")?;
    let id = compute_extension_id(folder, &manifest);

    let target = ext_dir(app, &id)?;
    let staging = target.with_file_name(format!(".{id}.installing-{}", Uuid::new_v4().simple()));
    let backup = target.with_file_name(format!(".{id}.backup-{}", Uuid::new_v4().simple()));
    let _ = std::fs::remove_dir_all(&staging);
    copy_dir(folder, &staging)?;

    let had_existing = target.exists();
    if had_existing {
        std::fs::rename(&target, &backup).map_err(|error| {
            let _ = std::fs::remove_dir_all(&staging);
            error.to_string()
        })?;
    }
    if let Err(error) = std::fs::rename(&staging, &target) {
        let _ = std::fs::remove_dir_all(&staging);
        if had_existing {
            let _ = std::fs::rename(&backup, &target);
        }
        return Err(error.to_string());
    }
    if had_existing {
        let _ = std::fs::remove_dir_all(&backup);
    }
    LOADED.lock().map_err(|_| "扩展状态无法读取")?.remove(&id);
    Ok(list(app))
}

/// 卸载：同时清理启用与禁用两处的目录。
pub fn remove(app: &AppHandle, id: &str) -> Result<Vec<ExtInfo>, String> {
    sanitize_id(id)?;
    LOADED.lock().map_err(|_| "扩展状态无法读取")?.remove(id);
    let marker = disabled_dir(app)?.join(format!("{id}.disabled"));
    if marker.exists() {
        std::fs::remove_file(marker).map_err(|error| error.to_string())?;
    }
    for path in [ext_dir(app, id)?, ext_disabled_dir(app, id)?] {
        if path.exists() {
            std::fs::remove_dir_all(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(list(app))
}

/// 用旁路标记记录禁用，不移动扩展目录，保持扩展 ID 和脚本数据。
pub fn set_enabled(app: &AppHandle, id: &str, enabled: bool) -> Result<Vec<ExtInfo>, String> {
    sanitize_id(id)?;
    let marker = disabled_dir(app)?.join(format!("{id}.disabled"));
    if enabled {
        let legacy = ext_disabled_dir(app, id)?;
        let target = ext_dir(app, id)?;
        if legacy.exists() && !target.exists() {
            std::fs::rename(legacy, target).map_err(|error| error.to_string())?;
        }
        if marker.exists() {
            std::fs::remove_file(marker).map_err(|error| error.to_string())?;
        }
    } else {
        std::fs::write(marker, "disabled").map_err(|error| error.to_string())?;
    }
    Ok(list(app))
}

#[cfg(test)]
mod tests {
    use super::is_safe_locale;

    #[test]
    fn locale_name_cannot_escape_extension_directory() {
        assert!(is_safe_locale("en"));
        assert!(is_safe_locale("zh_CN"));
        assert!(is_safe_locale("en-US"));
        assert!(!is_safe_locale("../secret"));
        assert!(!is_safe_locale(r"..\secret"));
        assert!(!is_safe_locale("C:\\secret"));
        assert!(!is_safe_locale(""));
    }
    #[test]
    fn extension_page_cannot_escape_its_origin() {
        assert_eq!(
            super::extension_page("abcdefghijklmnop", "popup/index.html").as_deref(),
            Some("chrome-extension://abcdefghijklmnop/popup/index.html")
        );
        assert!(super::extension_page("abcdefghijklmnop", "https://example.com/").is_none());
        assert!(super::extension_page("abcdefghijklmnop", "//other/options.html").is_none());
    }

    #[test]
    fn disabled_extension_keeps_its_path_and_id() {
        let root =
            std::env::temp_dir().join(format!("quickpane-extension-{}", uuid::Uuid::new_v4()));
        let ext = root.join("Extensions/test");
        std::fs::create_dir_all(&ext).unwrap();
        std::fs::create_dir_all(root.join("ExtensionsDisabled")).unwrap();
        std::fs::write(
            ext.join("manifest.json"),
            r#"{"name":"Test","version":"1","action":{"default_popup":"popup.html"}}"#,
        )
        .unwrap();
        let before = super::info_from_dir(&ext, true).unwrap();
        std::fs::write(root.join("ExtensionsDisabled/test.disabled"), "disabled").unwrap();
        let after = super::info_from_dir(&ext, true).unwrap();
        assert!(before.enabled);
        assert!(!after.enabled);
        assert_eq!(before.popup_url, after.popup_url);
        assert!(super::pinned_infos(&root, &["test".into()]).is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
}
