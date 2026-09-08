use std::cmp::min;

use tauri::{AppHandle, Manager};

use crate::browser::emit_snapshot;
use crate::state::{
    AppSnapshot, AppState, RuntimeData, SessionSnapshotRecord, SessionSnapshotTab, TabRecord,
    WorkspaceRecord,
};
use crate::workspace::{close_all_tab_webviews, restore_active_tab_webview};

pub const NEW_TAB_URL: &str = "quickpane://newtab";
pub const MAX_SESSION_SNAPSHOTS: usize = 100;
pub const SESSION_SNAPSHOT_NAME_MAX_CHARS: usize = 40;

/// 校验快照名称：去除首尾空白，限制长度 1~40 字符。
pub fn validate_session_snapshot_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("快照名称不能为空".into());
    }
    let char_count = trimmed.chars().count();
    if char_count > SESSION_SNAPSHOT_NAME_MAX_CHARS {
        return Err(format!(
            "快照名称不能超过 {SESSION_SNAPSHOT_NAME_MAX_CHARS} 个字符"
        ));
    }
    Ok(trimmed.to_string())
}

/// 纯数据流转：将当前 runtime 中的标签捕获为命名会话快照。
pub fn save_session_snapshot_data(
    runtime: &mut RuntimeData,
    name: &str,
) -> Result<SessionSnapshotRecord, String> {
    let valid_name = validate_session_snapshot_name(name)?;
    if runtime.data.session_snapshots.len() >= MAX_SESSION_SNAPSHOTS {
        return Err(format!("最多保存 {MAX_SESSION_SNAPSHOTS} 个会话快照"));
    }

    let tabs: Vec<SessionSnapshotTab> = runtime
        .data
        .tabs
        .iter()
        .map(|t| SessionSnapshotTab {
            url: t.url.clone(),
            title: t.title.clone(),
            pinned: t.pinned,
        })
        .collect();

    if tabs.is_empty() {
        return Err("当前没有可保存的标签页".into());
    }

    let active_index = runtime
        .data
        .active_tab_id
        .as_deref()
        .and_then(|active_id| runtime.data.tabs.iter().position(|t| t.id == active_id))
        .unwrap_or(0);

    let snapshot = SessionSnapshotRecord::new(valid_name, tabs, active_index);
    runtime.data.session_snapshots.insert(0, snapshot.clone());
    Ok(snapshot)
}

/// 纯数据流转：删除指定 ID 的会话快照。
pub fn delete_session_snapshot_data(
    runtime: &mut RuntimeData,
    snapshot_id: &str,
) -> Result<(), String> {
    let before_len = runtime.data.session_snapshots.len();
    runtime
        .data
        .session_snapshots
        .retain(|s| s.id != snapshot_id);
    if runtime.data.session_snapshots.len() == before_len {
        return Err("未找到指定会话快照".into());
    }
    Ok(())
}

/// 纯数据流转：重命名指定 ID 的会话快照。
pub fn rename_session_snapshot_data(
    runtime: &mut RuntimeData,
    snapshot_id: &str,
    new_name: &str,
) -> Result<(), String> {
    let valid_name = validate_session_snapshot_name(new_name)?;
    let snapshot = runtime
        .data
        .session_snapshots
        .iter_mut()
        .find(|s| s.id == snapshot_id)
        .ok_or_else(|| "未找到指定会话快照".to_string())?;
    snapshot.name = valid_name;
    Ok(())
}

/// 纯数据流转：恢复会话快照。
/// - as_new_workspace: true -> 暂存当前标签，并在新建的工作区中打开快照。
/// - as_new_workspace: false -> 用快照替换当前工作区标签（原标签全部移入 recently_closed，便于撤销）。
pub fn restore_session_snapshot_data(
    runtime: &mut RuntimeData,
    snapshot_id: &str,
    as_new_workspace: bool,
) -> Result<(), String> {
    let snapshot = runtime
        .data
        .session_snapshots
        .iter()
        .find(|s| s.id == snapshot_id)
        .cloned()
        .ok_or_else(|| "未找到指定会话快照".to_string())?;

    let restored_tabs: Vec<TabRecord> = if snapshot.tabs.is_empty() {
        vec![TabRecord::new(NEW_TAB_URL.into(), "新标签页".into(), false)]
    } else {
        snapshot
            .tabs
            .into_iter()
            .map(|t| TabRecord::new(t.url, t.title, t.pinned))
            .collect()
    };

    let target_active_index = min(snapshot.active_index, restored_tabs.len().saturating_sub(1));
    let next_active_id = restored_tabs.get(target_active_index).map(|t| t.id.clone());

    if as_new_workspace {
        // 暂存当前激活工作区标签
        if let Some(active_ws_id) = runtime.data.active_workspace_id.clone() {
            if let Some(ws) = runtime
                .data
                .workspaces
                .iter_mut()
                .find(|w| w.id == active_ws_id)
            {
                ws.tabs = std::mem::take(&mut runtime.data.tabs);
                ws.active_tab_id = runtime.data.active_tab_id.take();
            }
        }

        // 新建并切入工作区
        let mut new_ws = WorkspaceRecord::new(&snapshot.name);
        new_ws.active_tab_id = next_active_id.clone();
        let new_ws_id = new_ws.id.clone();
        runtime.data.workspaces.push(new_ws);
        runtime.data.active_workspace_id = Some(new_ws_id);
        runtime.data.tabs = restored_tabs;
        runtime.data.active_tab_id = next_active_id;
    } else {
        // 就地替换：原标签整体退入 recently_closed
        let old_tabs = std::mem::take(&mut runtime.data.tabs);
        for old in old_tabs.into_iter().rev() {
            runtime.data.recently_closed.insert(0, old);
        }
        if runtime.data.recently_closed.len() > 30 {
            runtime.data.recently_closed.truncate(30);
        }
        runtime.data.tabs = restored_tabs;
        runtime.data.active_tab_id = next_active_id;
    }

    Ok(())
}

pub fn save_session_snapshot(app: &AppHandle, name: &str) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    state.mutate_result(|runtime| save_session_snapshot_data(runtime, name))?;
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub fn delete_session_snapshot(app: &AppHandle, snapshot_id: &str) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    state.mutate_result(|runtime| delete_session_snapshot_data(runtime, snapshot_id))?;
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub fn rename_session_snapshot(
    app: &AppHandle,
    snapshot_id: &str,
    new_name: &str,
) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    state.mutate_result(|runtime| rename_session_snapshot_data(runtime, snapshot_id, new_name))?;
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub fn restore_session_snapshot(
    app: &AppHandle,
    snapshot_id: &str,
    as_new_workspace: bool,
) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    close_all_tab_webviews(app);
    state.mutate_result(|runtime| {
        restore_session_snapshot_data(runtime, snapshot_id, as_new_workspace)
    })?;
    restore_active_tab_webview(app);
    emit_snapshot(app);
    Ok(state.snapshot())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PersistedData;

    fn tab(url: &str, title: &str) -> TabRecord {
        TabRecord::new(url.into(), title.into(), false)
    }

    fn runtime_fixture() -> RuntimeData {
        let t1 = tab("https://example.com/1", "One");
        let t2 = tab("https://example.com/2", "Two");
        let active_id = t2.id.clone();
        let ws = WorkspaceRecord::new("默认");
        let ws_id = ws.id.clone();
        RuntimeData {
            data: PersistedData {
                tabs: vec![t1, t2],
                active_tab_id: Some(active_id),
                workspaces: vec![ws],
                active_workspace_id: Some(ws_id),
                ..PersistedData::default()
            },
            locked: false,
            first_run: false,
            window_visible: true,
            quitting: false,
            shell_mode: false,
            shell_collapsed: false,
            previous_window: 0,
            hidden_since: None,
        }
    }

    #[test]
    fn validate_name_rules() {
        assert!(validate_session_snapshot_name("").is_err());
        assert!(validate_session_snapshot_name("   ").is_err());
        assert_eq!(
            validate_session_snapshot_name("  工作现场  ").unwrap(),
            "工作现场"
        );
        let too_long = "a".repeat(SESSION_SNAPSHOT_NAME_MAX_CHARS + 1);
        assert!(validate_session_snapshot_name(&too_long).is_err());
    }

    #[test]
    fn save_snapshot_records_tabs_and_active_index() {
        let mut rt = runtime_fixture();
        let saved = save_session_snapshot_data(&mut rt, "事故调查").expect("save");
        assert_eq!(saved.name, "事故调查");
        assert_eq!(saved.tabs.len(), 2);
        assert_eq!(saved.tabs[0].url, "https://example.com/1");
        assert_eq!(saved.tabs[1].url, "https://example.com/2");
        assert_eq!(saved.active_index, 1);
        assert_eq!(rt.data.session_snapshots.len(), 1);
    }

    #[test]
    fn rename_and_delete_snapshot() {
        let mut rt = runtime_fixture();
        let saved = save_session_snapshot_data(&mut rt, "旧名称").unwrap();
        assert!(rename_session_snapshot_data(&mut rt, &saved.id, "新名称").is_ok());
        assert_eq!(rt.data.session_snapshots[0].name, "新名称");

        assert!(delete_session_snapshot_data(&mut rt, &saved.id).is_ok());
        assert!(rt.data.session_snapshots.is_empty());
        assert!(delete_session_snapshot_data(&mut rt, &saved.id).is_err());
    }

    #[test]
    fn restore_as_new_workspace_preserves_old_tabs() {
        let mut rt = runtime_fixture();
        let original_t1_id = rt.data.tabs[0].id.clone();
        let snapshot = save_session_snapshot_data(&mut rt, "调研备份").unwrap();

        restore_session_snapshot_data(&mut rt, &snapshot.id, true).expect("restore new ws");

        assert_eq!(rt.data.workspaces.len(), 2);
        assert_eq!(rt.data.workspaces[1].name, "调研备份", "新工作区以快照命名");
        assert_eq!(
            rt.data.active_workspace_id.as_deref(),
            Some(rt.data.workspaces[1].id.as_str())
        );
        assert_eq!(rt.data.tabs.len(), 2);
        // 原工作区 tabs 已安全暂存
        assert_eq!(rt.data.workspaces[0].tabs.len(), 2);
        assert_eq!(rt.data.workspaces[0].tabs[0].id, original_t1_id);
    }

    #[test]
    fn restore_in_place_stashes_old_tabs_into_recently_closed() {
        let mut rt = runtime_fixture();
        let original_t1_id = rt.data.tabs[0].id.clone();
        let snapshot = save_session_snapshot_data(&mut rt, "覆盖恢复").unwrap();

        restore_session_snapshot_data(&mut rt, &snapshot.id, false).expect("restore in place");

        assert_eq!(rt.data.workspaces.len(), 1, "未新建工作区");
        assert_eq!(rt.data.tabs.len(), 2);
        assert_eq!(rt.data.recently_closed.len(), 2);
        assert!(rt
            .data
            .recently_closed
            .iter()
            .any(|t| t.id == original_t1_id));
    }
}
