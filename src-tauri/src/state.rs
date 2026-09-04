use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Instant,
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
pub const DEFAULT_PALETTE_SHORTCUT: &str = "Ctrl+Shift+A";
pub const DEFAULT_TAB_HIBERNATION_MINUTES: u32 = 15;
pub const DEFAULT_WORKSPACE_NAME: &str = "默认工作区";
pub const WORKSPACE_NAME_MAX_CHARS: usize = 24;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TabRecord {
    pub id: String,
    pub title: String,
    pub url: String,
    pub pinned: bool,
    pub loading: bool,
    pub loaded: bool,
    pub muted: bool,
    /// 休眠中：后台 WebView 已释放，激活时按 url 重建。
    pub hibernated: bool,
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
            hibernated: false,
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

/// 工作区：暂存非当前工作区的标签集合与激活标签。
/// 当前工作区的标签位于 PersistedData::tabs，切走时才写回这里的记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRecord {
    pub id: String,
    pub name: String,
    pub tabs: Vec<TabRecord>,
    pub active_tab_id: Option<String>,
}

impl WorkspaceRecord {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().simple().to_string(),
            name: name.into(),
            tabs: Vec::new(),
            active_tab_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub shortcut: Option<String>,
    /// 快速切换面板快捷键（Ctrl+K 之外的可配置呼出键）。
    #[serde(alias = "tabSearchShortcut")]
    pub palette_shortcut: String,
    pub autostart: bool,
    pub home_url: String,
    pub search_template: String,
    pub history_days: u32,
    pub password_hash: Option<String>,
    pub lock_on_system_lock: bool,
    pub auto_lock_after_hide_seconds: u32,
    pub quick_links: Vec<QuickLink>,
    /// "system" 跟随系统代理，"direct" 强制直连，"custom" 使用 proxy_url。
    pub proxy_mode: String,
    pub proxy_url: String,
    /// 固定到导航栏的扩展 id（Extensions/ 下的文件夹名）。
    pub pinned_extensions: Vec<String>,
    /// 标签休眠阈值（分钟）：0 关闭，仅暂停媒体；后台标签超时后释放 WebView。
    pub tab_hibernation_minutes: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shortcut: None,
            palette_shortcut: DEFAULT_PALETTE_SHORTCUT.into(),
            autostart: false,
            home_url: DEFAULT_HOME.into(),
            search_template: DEFAULT_SEARCH.into(),
            history_days: 90,
            password_hash: None,
            lock_on_system_lock: true,
            auto_lock_after_hide_seconds: 0,
            quick_links: vec![QuickLink {
                id: "kaodes".into(),
                title: "考得尚".into(),
                url: DEFAULT_HOME.into(),
            }],
            proxy_mode: "system".into(),
            proxy_url: String::new(),
            pinned_extensions: Vec::new(),
            tab_hibernation_minutes: DEFAULT_TAB_HIBERNATION_MINUTES,
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
    /// 全部工作区记录；当前工作区的 tabs 在切换离开时才写回。
    pub workspaces: Vec<WorkspaceRecord>,
    pub active_workspace_id: Option<String>,
}

impl Default for PersistedData {
    fn default() -> Self {
        let tab = TabRecord::new("quickpane://newtab".into(), "新标签页".into(), false);
        let workspace = WorkspaceRecord::new(DEFAULT_WORKSPACE_NAME);
        Self {
            active_tab_id: Some(tab.id.clone()),
            tabs: vec![tab],
            recently_closed: Vec::new(),
            history: Vec::new(),
            bookmarks: Vec::new(),
            downloads: Vec::new(),
            settings: Settings::default(),
            workspaces: vec![workspace.clone()],
            active_workspace_id: Some(workspace.id),
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

impl AppSnapshot {
    fn from_runtime(
        runtime: &RuntimeData,
        pinned_extensions: Vec<ExtInfo>,
        recovery_message: Option<String>,
    ) -> Self {
        let is_redacted = runtime.locked || runtime.first_run;
        let has_password = runtime.data.settings.password_hash.is_some();
        let mut data = if is_redacted {
            // 锁屏快照只保留空壳，避免通过 IPC 泄露历史、网址、下载路径或密码哈希。
            PersistedData::default()
        } else {
            runtime.data.clone()
        };
        // 前端只需要 hasPassword；不要把可用于离线破解的 Argon2 哈希发送到 IPC 消费者。
        data.settings.password_hash = None;
        Self {
            data,
            locked: runtime.locked,
            first_run: runtime.first_run,
            has_password,
            window_visible: runtime.window_visible,
            pinned_extensions: if is_redacted {
                Vec::new()
            } else {
                pinned_extensions
            },
            recovery_message,
        }
    }
}

#[derive(Clone)]
pub struct RuntimeData {
    pub data: PersistedData,
    pub locked: bool,
    pub first_run: bool,
    pub window_visible: bool,
    pub quitting: bool,
    pub shell_mode: bool,
    /// 浏览态收缩：main WebView 只覆盖顶部 chrome（86px），网页区域无遮挡可交互；
    /// 浮层/页面/锁屏需要时扩回满幅。
    pub shell_collapsed: bool,
    pub previous_window: isize,
    pub hidden_since: Option<Instant>,
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
                shell_collapsed: false,
                previous_window: 0,
                hidden_since: None,
            }),
            path,
            recovery_message,
        };
        state.save()?;
        Ok(state)
    }

    pub fn normalize(data: &mut PersistedData) {
        data.history.retain(|entry| {
            entry.visited_at >= Utc::now() - Duration::days(data.settings.history_days as i64)
        });
        if !matches!(
            data.settings.auto_lock_after_hide_seconds,
            0 | 60 | 300 | 900
        ) {
            data.settings.auto_lock_after_hide_seconds = 0;
        }
        if ![0, 5, 15, 30, 60].contains(&data.settings.tab_hibernation_minutes) {
            data.settings.tab_hibernation_minutes = DEFAULT_TAB_HIBERNATION_MINUTES;
        }
        data.history.truncate(5_000);
        data.downloads.truncate(500);
        data.recently_closed.truncate(20);

        for tab in &mut data.tabs {
            tab.loading = false;
            tab.loaded = false;
            tab.muted = false;
            tab.hibernated = false;
        }
        for workspace in &mut data.workspaces {
            for tab in &mut workspace.tabs {
                tab.loading = false;
                tab.loaded = false;
                tab.muted = false;
                tab.hibernated = false;
            }
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
        data.tabs.sort_by_key(|tab| !tab.pinned);

        // 工作区兜底：旧数据无 workspaces 字段时补一个默认工作区并指向它。
        if data.workspaces.is_empty() {
            data.workspaces
                .push(WorkspaceRecord::new(DEFAULT_WORKSPACE_NAME));
        }
        let workspace_valid = data
            .active_workspace_id
            .as_ref()
            .is_some_and(|id| data.workspaces.iter().any(|workspace| &workspace.id == id));
        if !workspace_valid {
            data.active_workspace_id = data
                .workspaces
                .first()
                .map(|workspace| workspace.id.clone());
        }
    }

    pub fn snapshot(&self) -> AppSnapshot {
        let guard = self.inner.lock().expect("app state poisoned");
        let is_redacted = guard.locked || guard.first_run;
        let data_dir = self
            .path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let pinned_extensions = if is_redacted {
            Vec::new()
        } else {
            extensions::pinned_infos(&data_dir, &guard.data.settings.pinned_extensions)
        };
        AppSnapshot::from_runtime(&guard, pinned_extensions, self.recovery_message.clone())
    }

    pub fn save(&self) -> Result<(), String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "应用状态无法读取".to_string())?;
        atomic_write_json(&self.path, &guard.data)
    }

    pub fn mutate<T>(&self, action: impl FnOnce(&mut RuntimeData) -> T) -> Result<T, String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "应用状态无法读取".to_string())?;
        let previous = guard.clone();
        let output = action(&mut guard);
        // 保持状态锁直到写盘结束；失败时同时恢复持久化数据和运行态，避免内存与磁盘分叉。
        if let Err(error) = atomic_write_json(&self.path, &guard.data) {
            *guard = previous;
            return Err(error);
        }
        Ok(output)
    }

    pub fn mutate_result<T>(
        &self,
        action: impl FnOnce(&mut RuntimeData) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "应用状态无法读取".to_string())?;
        let previous = guard.clone();
        let output = match action(&mut guard) {
            Ok(output) => output,
            Err(error) => {
                *guard = previous;
                return Err(error);
            }
        };
        // 保持状态锁直到写盘结束；失败时同时恢复持久化数据和运行态，避免内存与磁盘分叉。
        if let Err(error) = atomic_write_json(&self.path, &guard.data) {
            *guard = previous;
            return Err(error);
        }
        Ok(output)
    }

    pub fn mutate_runtime<T>(
        &self,
        action: impl FnOnce(&mut RuntimeData) -> T,
    ) -> Result<T, String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "应用状态无法读取".to_string())?;
        Ok(action(&mut guard))
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
        Ok(())
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
        assert_eq!(data.workspaces.len(), 1);
        assert_eq!(data.workspaces[0].name, DEFAULT_WORKSPACE_NAME);
        assert_eq!(
            data.active_workspace_id.as_deref(),
            Some(data.workspaces[0].id.as_str())
        );
    }

    #[test]
    fn legacy_data_without_workspaces_gets_default_workspace() {
        // 旧版 quickpane.json 没有 workspaces 字段：serde 默认为空，load 后 normalize 兜底。
        let legacy = serde_json::json!({
            "tabs": [{
                "id": "tab-1",
                "title": "页面",
                "url": "https://example.com",
                "pinned": false,
                "loading": false,
                "loaded": false,
                "muted": false,
                "createdAt": "2026-01-01T00:00:00Z",
                "lastActiveAt": "2026-01-01T00:00:00Z"
            }],
            "activeTabId": "tab-1",
            "recentlyClosed": [],
            "history": [],
            "bookmarks": [],
            "downloads": [],
            "settings": {
                "tabSearchShortcut": "Ctrl+Shift+A",
                "recentlyClosedShortcut": "Ctrl+Shift+Y",
                "autostart": false,
                "homeUrl": "https://kaodes.com",
                "searchTemplate": "https://cn.bing.com/search?q={query}",
                "historyDays": 90,
                "lockOnSystemLock": true,
                "autoLockAfterHideSeconds": 0,
                "quickLinks": [],
                "proxyMode": "system",
                "proxyUrl": "",
                "pinnedExtensions": []
            }
        });
        let mut data: PersistedData = serde_json::from_value(legacy).expect("parse legacy data");
        // 旧字段 tabSearchShortcut 经 serde alias 迁移到 palette_shortcut。
        assert_eq!(data.settings.palette_shortcut, "Ctrl+Shift+A");
        // 容器级 serde(default)：旧数据缺 workspaces 时反序列化即落默认工作区；
        // active_workspace_id 的默认值来自另一个 Default 实例，id 不匹配，由 normalize 修复。
        assert_eq!(data.workspaces.len(), 1);
        data.tabs[0].hibernated = true;
        data.tabs[0].loaded = true;

        AppState::normalize(&mut data);

        assert_eq!(data.workspaces.len(), 1);
        assert_eq!(data.workspaces[0].name, DEFAULT_WORKSPACE_NAME);
        assert_eq!(
            data.active_workspace_id.as_deref(),
            Some(data.workspaces[0].id.as_str())
        );
        assert!(!data.tabs[0].hibernated);
        assert!(!data.tabs[0].loaded);
    }

    #[test]
    fn hibernation_threshold_normalizes_to_default() {
        let mut data = PersistedData::default();
        data.settings.tab_hibernation_minutes = 7;
        AppState::normalize(&mut data);
        assert_eq!(
            data.settings.tab_hibernation_minutes,
            DEFAULT_TAB_HIBERNATION_MINUTES
        );
        for allowed in [0, 5, 15, 30, 60] {
            data.settings.tab_hibernation_minutes = allowed;
            AppState::normalize(&mut data);
            assert_eq!(data.settings.tab_hibernation_minutes, allowed);
        }
    }

    #[test]
    fn first_run_snapshot_redacts_persisted_data() {
        let mut data = PersistedData::default();
        data.settings.home_url = "https://private.example".into();
        data.settings.password_hash = Some("stored-hash".into());
        let runtime = RuntimeData {
            data,
            locked: false,
            first_run: true,
            window_visible: true,
            quitting: false,
            shell_mode: true,
            shell_collapsed: false,
            previous_window: 0,
            hidden_since: None,
        };

        let snapshot = AppSnapshot::from_runtime(&runtime, Vec::new(), None);
        assert_eq!(snapshot.data.settings.home_url, DEFAULT_HOME);
        assert!(snapshot.first_run);
        assert!(snapshot.has_password);
        assert!(snapshot.data.settings.password_hash.is_none());
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
                shell_collapsed: false,
                previous_window: 0,
                hidden_since: None,
            }),
            path: PathBuf::from("quickpane-test.json"),
            recovery_message: None,
        };

        let snapshot = state.snapshot();
        assert!(snapshot.has_password);
        assert!(snapshot.data.settings.password_hash.is_none());
    }

    #[test]
    fn mutate_restores_memory_when_persistence_fails() {
        let missing_parent = std::env::temp_dir()
            .join(format!("quickpane-missing-{}", Uuid::new_v4()))
            .join("nested");
        let state = AppState {
            inner: Mutex::new(RuntimeData {
                data: PersistedData::default(),
                locked: false,
                first_run: false,
                window_visible: true,
                quitting: false,
                shell_mode: true,
                shell_collapsed: false,
                previous_window: 0,
                hidden_since: None,
            }),
            path: missing_parent.join("quickpane.json"),
            recovery_message: None,
        };
        let original_home = state
            .inner
            .lock()
            .expect("state lock")
            .data
            .settings
            .home_url
            .clone();

        let result = state.mutate(|runtime| {
            runtime.data.settings.home_url = "https://changed.example".into();
            runtime.locked = true;
        });

        assert!(result.is_err());
        let runtime = state.inner.lock().expect("state lock");
        assert_eq!(runtime.data.settings.home_url, original_home);
        assert!(!runtime.locked);
    }

    #[test]
    fn concurrent_mutations_keep_memory_and_disk_in_sync() {
        let directory = std::env::temp_dir().join(format!("quickpane-state-{}", Uuid::new_v4()));
        fs::create_dir_all(&directory).expect("create temp directory");
        let state = std::sync::Arc::new(AppState {
            inner: Mutex::new(RuntimeData {
                data: PersistedData::default(),
                locked: false,
                first_run: false,
                window_visible: true,
                quitting: false,
                shell_mode: true,
                shell_collapsed: false,
                previous_window: 0,
                hidden_since: None,
            }),
            path: directory.join("quickpane.json"),
            recovery_message: None,
        });
        state.save().expect("save initial state");

        let workers: Vec<_> = (0..8)
            .map(|index| {
                let state = state.clone();
                std::thread::spawn(move || {
                    state
                        .mutate(|runtime| {
                            runtime.data.settings.quick_links.push(QuickLink {
                                id: format!("test-{index}"),
                                title: format!("Test {index}"),
                                url: format!("https://{index}.example"),
                            });
                        })
                        .expect("mutate state");
                })
            })
            .collect();
        for worker in workers {
            worker.join().expect("join worker");
        }

        let memory = state.inner.lock().expect("state lock").data.clone();
        let disk: PersistedData = serde_json::from_slice(
            &fs::read(directory.join("quickpane.json")).expect("read saved state"),
        )
        .expect("parse saved state");
        assert_eq!(
            serde_json::to_value(memory).expect("serialize memory"),
            serde_json::to_value(disk).expect("serialize disk")
        );
        fs::remove_dir_all(directory).expect("remove temp directory");
    }

    #[test]
    fn mutate_result_restores_memory_when_action_fails() {
        let missing_parent = std::env::temp_dir()
            .join(format!("quickpane-missing-{}", Uuid::new_v4()))
            .join("nested");
        let state = AppState {
            inner: Mutex::new(RuntimeData {
                data: PersistedData::default(),
                locked: false,
                first_run: false,
                window_visible: true,
                quitting: false,
                shell_mode: true,
                shell_collapsed: false,
                previous_window: 0,
                hidden_since: None,
            }),
            path: missing_parent.join("quickpane.json"),
            recovery_message: None,
        };
        let original_home = state
            .inner
            .lock()
            .expect("state lock")
            .data
            .settings
            .home_url
            .clone();

        let result: Result<(), String> = state.mutate_result(|runtime| {
            runtime.data.settings.home_url = "https://changed.example".into();
            Err("rejected".into())
        });

        assert_eq!(result.expect_err("action should fail"), "rejected");
        assert_eq!(
            state
                .inner
                .lock()
                .expect("state lock")
                .data
                .settings
                .home_url,
            original_home
        );
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
