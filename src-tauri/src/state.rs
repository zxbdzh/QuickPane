use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::extensions::{self, ExtInfo};

pub const DEFAULT_HOME: &str = "https://kaodes.com";
pub const DEFAULT_SEARCH: &str = "https://cn.bing.com/search?q={query}";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabRecord {
    pub id: String,
    pub title: String,
    pub url: String,
    pub pinned: bool,
    pub loading: bool,
    pub loaded: bool,
    pub muted: bool,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

impl TabRecord {
    pub fn new(url: String, title: String, loaded: bool) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().simple().to_string(),
            title,
            url,
            pinned: false,
            loading: loaded,
            loaded,
            muted: false,
            created_at: now,
            last_active_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub visited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRecord {
    pub id: String,
    pub file_name: String,
    pub url: String,
    pub path: Option<String>,
    pub state: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickLink {
    pub id: String,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub shortcut: Option<String>,
    pub autostart: bool,
    pub home_url: String,
    pub search_template: String,
    pub history_days: u32,
    pub password_hash: Option<String>,
    pub lock_on_system_lock: bool,
    pub quick_links: Vec<QuickLink>,
    /// "system" 跟随系统代理，"direct" 强制直连，"custom" 使用 proxy_url。
    pub proxy_mode: String,
    pub proxy_url: String,
    /// 固定到导航栏的扩展 id（Extensions/ 下的文件夹名）。
    pub pinned_extensions: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shortcut: None,
            autostart: false,
            home_url: DEFAULT_HOME.into(),
            search_template: DEFAULT_SEARCH.into(),
            history_days: 90,
            password_hash: None,
            lock_on_system_lock: true,
            quick_links: vec![QuickLink {
                id: "kaodes".into(),
                title: "考得尚".into(),
                url: DEFAULT_HOME.into(),
            }],
            proxy_mode: "system".into(),
            proxy_url: String::new(),
            pinned_extensions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PersistedData {
    pub tabs: Vec<TabRecord>,
    pub active_tab_id: Option<String>,
    pub recently_closed: Vec<TabRecord>,
    pub history: Vec<HistoryEntry>,
    pub bookmarks: Vec<Bookmark>,
    pub downloads: Vec<DownloadRecord>,
    pub settings: Settings,
}

impl Default for PersistedData {
    fn default() -> Self {
        let tab = TabRecord::new("quickpane://newtab".into(), "新标签页".into(), false);
        Self {
            active_tab_id: Some(tab.id.clone()),
            tabs: vec![tab],
            recently_closed: Vec::new(),
            history: Vec::new(),
            bookmarks: Vec::new(),
            downloads: Vec::new(),
            settings: Settings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub data: PersistedData,
    pub locked: bool,
    pub first_run: bool,
    pub has_password: bool,
    pub window_visible: bool,
    /// 固定到导航栏的扩展（从磁盘实时解析，随快照事件刷新）。
    pub pinned_extensions: Vec<ExtInfo>,
    pub recovery_message: Option<String>,
}
pub struct RuntimeData {
    pub data: PersistedData,
    pub locked: bool,
    pub first_run: bool,
    pub window_visible: bool,
    pub quitting: bool,
    pub shell_mode: bool,
    pub previous_window: isize,
}

pub struct AppState {
    pub inner: Mutex<RuntimeData>,
    path: PathBuf,
    recovery_message: Option<String>,
}

impl AppState {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?;
        fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
        let path = data_dir.join("quickpane.json");
        let existed = path.exists();
        let mut recovery_message = None;
        let mut data = if existed {
            let contents = fs::read_to_string(&path).map_err(|error| error.to_string())?;
            match serde_json::from_str::<PersistedData>(&contents) {
                Ok(data) => data,
                Err(_error) => {
                    let backup = path.with_file_name(format!(
                        "quickpane.json.{}.corrupt",
                        Utc::now().format("%Y%m%d%H%M%S")
                    ));
                    fs::rename(&path, &backup).map_err(|rename_error| rename_error.to_string())?;
                    recovery_message = Some(format!(
                        "状态文件损坏，已备份为 {}，已使用默认状态启动。",
                        backup.display()
                    ));
                    PersistedData::default()
                }
            }
        } else {
            PersistedData::default()
        };

        Self::normalize(&mut data);
        let locked = data.settings.password_hash.is_some();
        let state = Self {
            inner: Mutex::new(RuntimeData {
                data,
                locked,
                first_run: !existed || recovery_message.is_some(),
                window_visible: true,
                quitting: false,
                shell_mode: true,
                previous_window: 0,
            }),
            path,
            recovery_message,
        };
        state.save()?;
        Ok(state)
    }

    fn normalize(data: &mut PersistedData) {
        data.history.retain(|entry| {
            entry.visited_at >= Utc::now() - Duration::days(data.settings.history_days as i64)
        });
        data.history.truncate(5_000);
        data.downloads.truncate(500);
        data.recently_closed.truncate(20);

        for tab in &mut data.tabs {
            tab.loading = false;
            tab.loaded = false;
            tab.muted = false;
        }

        if data.tabs.is_empty() {
            let tab = TabRecord::new("quickpane://newtab".into(), "新标签页".into(), false);
            data.active_tab_id = Some(tab.id.clone());
            data.tabs.push(tab);
        }
        let active_exists = data
            .active_tab_id
            .as_ref()
            .is_some_and(|id| data.tabs.iter().any(|tab| &tab.id == id));
        if !active_exists {
            data.active_tab_id = data.tabs.first().map(|tab| tab.id.clone());
        }
    }

    pub fn snapshot(&self) -> AppSnapshot {
        let guard = self.inner.lock().expect("app state poisoned");
        let has_password = guard.data.settings.password_hash.is_some();
        let data_dir = self
            .path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let locked = guard.locked || guard.first_run;
        let mut data = if locked {
            // 锁屏快照只保留空壳，避免通过 IPC 泄露历史、网址、下载路径或密码哈希。
            PersistedData::default()
        } else {
            guard.data.clone()
        };
        // 前端只需要 hasPassword；不要把可用于离线破解的 Argon2 哈希发送到 IPC 消费者。
        data.settings.password_hash = None;
        AppSnapshot {
            data,
            locked: guard.locked,
            first_run: guard.first_run,
            has_password,
            window_visible: guard.window_visible,
            pinned_extensions: if locked {
                Vec::new()
            } else {
                extensions::pinned_infos(&data_dir, &guard.data.settings.pinned_extensions)
            },
            recovery_message: self.recovery_message.clone(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let data = self
            .inner
            .lock()
            .map_err(|_| "应用状态无法读取".to_string())?
            .data
            .clone();
        atomic_write_json(&self.path, &data)
    }

    pub fn mutate<T>(&self, action: impl FnOnce(&mut RuntimeData) -> T) -> Result<T, String> {
        let output = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|_| "应用状态无法读取".to_string())?;
            action(&mut guard)
        };
        self.save()?;
        Ok(output)
    }

    pub fn verify_password(&self, password: &str) -> bool {
        let guard = match self.inner.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        let Some(hash) = guard.data.settings.password_hash.as_deref() else {
            return true;
        };
        let Ok(parsed) = PasswordHash::new(hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    }

    pub fn hash_password(password: &str) -> Result<String, String> {
        if password.chars().count() < 4 {
            return Err("应用密码至少需要 4 个字符".into());
        }
        let salt = SaltString::generate(&mut rand_core::OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|error| error.to_string())
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn atomic_write_json(path: &Path, data: &PersistedData) -> Result<(), String> {
    let temp = path.with_extension("json.tmp");
    let serialized = serde_json::to_vec_pretty(data).map_err(|error| error.to_string())?;
    fs::write(&temp, serialized).map_err(|error| error.to_string())?;

    if !path.exists() {
        return fs::rename(&temp, path).map_err(|error| error.to_string());
    }

    #[cfg(windows)]
    {
        use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
        use windows::{core::PCWSTR, Win32::Storage::FileSystem::ReplaceFileW};
        let existing: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();
        let replacement: Vec<u16> = OsStr::new(&temp).encode_wide().chain(Some(0)).collect();
        unsafe {
            ReplaceFileW(
                PCWSTR(existing.as_ptr()),
                PCWSTR(replacement.as_ptr()),
                PCWSTR::null(),
                windows::Win32::Storage::FileSystem::REPLACE_FILE_FLAGS(0),
                None,
                None,
            )
            .map_err(|error| error.to_string())?;
        }
        return Ok(());
    }

    #[cfg(not(windows))]
    fs::rename(&temp, path).map_err(|error| error.to_string())
}

pub fn display_title_from_url(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|value| value.host_str().map(str::to_string))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "新标签页".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_data_contains_a_new_tab() {
        let data = PersistedData::default();
        assert_eq!(data.tabs.len(), 1);
        assert_eq!(data.tabs[0].url, "quickpane://newtab");
        assert_eq!(
            data.active_tab_id.as_deref(),
            Some(data.tabs[0].id.as_str())
        );
    }

    #[test]
    fn snapshot_never_exposes_password_hash() {
        let hash = AppState::hash_password("correct horse").expect("hash password");
        let mut data = PersistedData::default();
        data.settings.password_hash = Some(hash);
        let state = AppState {
            inner: Mutex::new(RuntimeData {
                data,
                locked: false,
                first_run: false,
                window_visible: true,
                quitting: false,
                shell_mode: true,
                previous_window: 0,
            }),
            path: PathBuf::from("quickpane-test.json"),
            recovery_message: None,
        };

        let snapshot = state.snapshot();
        assert!(snapshot.has_password);
        assert!(snapshot.data.settings.password_hash.is_none());
    }

    #[test]
    fn password_hash_can_be_verified_without_storing_plaintext() {
        let hash = AppState::hash_password("correct horse").expect("hash password");
        assert!(!hash.contains("correct horse"));
        let parsed = PasswordHash::new(&hash).expect("parse hash");
        assert!(Argon2::default()
            .verify_password(b"correct horse", &parsed)
            .is_ok());
        assert!(Argon2::default()
            .verify_password(b"wrong password", &parsed)
            .is_err());
    }

    #[test]
    fn password_requires_four_characters() {
        assert!(AppState::hash_password("123").is_err());
        assert!(AppState::hash_password("1234").is_ok());
    }
}
