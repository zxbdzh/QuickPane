use std::path::{Path, PathBuf};

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

use base64::Engine;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::browser::extensions_dir;

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
    /// action/browser_action 的 default_popup，Webview2 无弹出面板宿主，
    /// 由前端用该 URL 在标签页中打开面板页面。
    pub popup_url: Option<String>,
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
    Some(format!(
        "chrome-extension://{extension_id}/{}",
        popup.trim_start_matches('/')
    ))
}

/// wry 会加载扩展目录下的每一个子文件夹，禁用即移动到旁边的隔离目录。
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
    let extension_id = compute_extension_id(path, &manifest);
    let id = path.file_name()?.to_string_lossy().to_string();
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
    items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    items
}

/// 快照用：按固定列表读取启用中的扩展信息（不存在的 id 跳过）。
pub fn pinned_infos(data_dir: &Path, ids: &[String]) -> Vec<ExtInfo> {
    ids.iter()
        .filter(|id| sanitize_id(id).is_ok())
        .filter_map(|id| info_from_dir(&data_dir.join("Extensions").join(id), true))
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
        return Ok(metadata.file_attributes() & 0x400 != 0);
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
    Ok(list(app))
}

/// 卸载：同时清理启用与禁用两处的目录。
pub fn remove(app: &AppHandle, id: &str) -> Result<Vec<ExtInfo>, String> {
    for path in [ext_dir(app, id)?, ext_disabled_dir(app, id)?] {
        if path.exists() {
            std::fs::remove_dir_all(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(list(app))
}

/// 启用/禁用通过在两个目录间移动文件夹实现，wry 只加载扩展目录。
pub fn set_enabled(app: &AppHandle, id: &str, enabled: bool) -> Result<Vec<ExtInfo>, String> {
    let (from, to) = if enabled {
        (ext_disabled_dir(app, id)?, ext_dir(app, id)?)
    } else {
        (ext_dir(app, id)?, ext_disabled_dir(app, id)?)
    };
    if !from.exists() {
        return Ok(list(app));
    }
    if to.exists() {
        std::fs::remove_dir_all(&to).map_err(|error| error.to_string())?;
    }
    std::fs::rename(&from, &to).map_err(|error| error.to_string())?;
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
}
