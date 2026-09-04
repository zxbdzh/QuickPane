//! 轻量工作区：暂存模型——`data.tabs` 始终是当前工作区的标签，
//! `workspaces[]` 记录每个工作区的标签集合与激活标签；切换时「暂存旧 / 装载新」。
//! 纯数据函数只动 RuntimeData（可单测），AppHandle 包装层补 WebView 副作用。

use tauri::{AppHandle, Manager};

use crate::browser::{
    emit_snapshot, ensure_tab_webview, hide_all_tabs, set_shell_expanded, show_active_tab,
    tab_label, MUTE_TAB_SCRIPT, UNMUTE_TAB_SCRIPT,
};
use crate::state::{
    AppSnapshot, AppState, Bookmark, RuntimeData, TabRecord, WorkspaceRecord,
    WORKSPACE_NAME_MAX_CHARS,
};

const NEW_TAB_URL: &str = "quickpane://newtab";

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabBatchUpdate {
    /// close | bookmark | mute | unmute | move
    pub action: String,
    pub tab_ids: Vec<String>,
    pub workspace_id: Option<String>,
}

pub fn validate_workspace_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("工作区名称不能为空".into());
    }
    if trimmed.chars().count() > WORKSPACE_NAME_MAX_CHARS {
        return Err(format!(
            "工作区名称不能超过 {WORKSPACE_NAME_MAX_CHARS} 个字符"
        ));
    }
    Ok(trimmed.to_string())
}

fn sort_tabs(tabs: &mut [TabRecord]) {
    tabs.sort_by_key(|tab| !tab.pinned);
}

/// 暂存：当前 tabs + 激活位写回当前工作区记录（清运行态标志，保留 pinned/muted）。
pub(crate) fn stash_current_tabs(runtime: &mut RuntimeData) {
    let Some(active_id) = runtime.data.active_workspace_id.clone() else {
        return;
    };
    let Some(workspace) = runtime
        .data
        .workspaces
        .iter_mut()
        .find(|workspace| workspace.id == active_id)
    else {
        return;
    };
    let mut tabs = std::mem::take(&mut runtime.data.tabs);
    for tab in &mut tabs {
        tab.loading = false;
        tab.loaded = false;
        tab.hibernated = false;
    }
    workspace.tabs = tabs;
    workspace.active_tab_id = runtime.data.active_tab_id.clone();
}

/// 装载：目标工作区记录 → data.tabs / activeTabId；空工作区补新标签页。
pub(crate) fn load_workspace_tabs(
    runtime: &mut RuntimeData,
    workspace_id: &str,
) -> Result<(), String> {
    let Some(workspace) = runtime
        .data
        .workspaces
        .iter()
        .find(|workspace| workspace.id == workspace_id)
    else {
        return Err("工作区不存在".into());
    };
    let mut tabs = workspace.tabs.clone();
    let recorded_active = workspace.active_tab_id.clone();
    if tabs.is_empty() {
        tabs.push(TabRecord::new(NEW_TAB_URL.into(), "新标签页".into(), false));
    }
    for tab in &mut tabs {
        tab.loading = false;
        tab.loaded = false;
        tab.hibernated = false;
    }
    let active_valid = recorded_active
        .as_ref()
        .is_some_and(|id| tabs.iter().any(|tab| &tab.id == id));
    runtime.data.active_tab_id = if active_valid {
        recorded_active
    } else {
        tabs.first().map(|tab| tab.id.clone())
    };
    runtime.data.tabs = tabs;
    runtime.data.active_workspace_id = Some(workspace_id.to_string());
    runtime.shell_mode = runtime
        .data
        .active_tab_id
        .as_ref()
        .and_then(|id| runtime.data.tabs.iter().find(|tab| &tab.id == id))
        .is_none_or(|tab| tab.url == NEW_TAB_URL);
    Ok(())
}

/// 创建并切换到新工作区。
pub(crate) fn create_workspace_data(
    runtime: &mut RuntimeData,
    name: &str,
) -> Result<String, String> {
    let name = validate_workspace_name(name)?;
    let workspace = WorkspaceRecord::new(name);
    let id = workspace.id.clone();
    stash_current_tabs(runtime);
    runtime.data.workspaces.push(workspace);
    load_workspace_tabs(runtime, &id)?;
    Ok(id)
}

pub(crate) fn rename_workspace_data(
    runtime: &mut RuntimeData,
    workspace_id: &str,
    name: &str,
) -> Result<(), String> {
    let name = validate_workspace_name(name)?;
    let workspace = runtime
        .data
        .workspaces
        .iter_mut()
        .find(|workspace| workspace.id == workspace_id)
        .ok_or_else(|| "工作区不存在".to_string())?;
    workspace.name = name;
    Ok(())
}

pub(crate) fn remove_workspace_data(
    runtime: &mut RuntimeData,
    workspace_id: &str,
) -> Result<(), String> {
    if runtime.data.workspaces.len() <= 1 {
        return Err("至少保留一个工作区".into());
    }
    if runtime.data.active_workspace_id.as_deref() == Some(workspace_id) {
        return Err("无法删除当前工作区，请先切换到其它工作区".into());
    }
    runtime
        .data
        .workspaces
        .retain(|workspace| workspace.id != workspace_id);
    Ok(())
}

/// 移动标签到目标工作区；返回被移动的标签（供关闭 WebView）。激活转移规则同 close_tab。
pub(crate) fn move_tabs_data(
    runtime: &mut RuntimeData,
    tab_ids: &[String],
    workspace_id: &str,
) -> Result<Vec<TabRecord>, String> {
    let active = runtime.data.active_workspace_id.clone();
    if active.as_deref() == Some(workspace_id) {
        return Err("标签已在当前工作区".into());
    }
    let target_exists = runtime
        .data
        .workspaces
        .iter()
        .any(|workspace| workspace.id == workspace_id);
    if !target_exists {
        return Err("目标工作区不存在".into());
    }
    if tab_ids.is_empty() {
        return Ok(Vec::new());
    }

    let first_removed_index = runtime
        .data
        .tabs
        .iter()
        .position(|tab| tab_ids.contains(&tab.id));
    let active_moved = runtime
        .data
        .active_tab_id
        .as_ref()
        .is_some_and(|id| tab_ids.contains(id));

    let mut moved: Vec<TabRecord> = Vec::new();
    runtime.data.tabs.retain(|tab| {
        if tab_ids.contains(&tab.id) {
            moved.push(tab.clone());
            false
        } else {
            true
        }
    });
    if moved.is_empty() {
        return Ok(moved);
    }
    for tab in &mut moved {
        tab.loading = false;
        tab.loaded = false;
        tab.hibernated = false;
    }

    if runtime.data.tabs.is_empty() {
        runtime
            .data
            .tabs
            .push(TabRecord::new(NEW_TAB_URL.into(), "新标签页".into(), false));
    }
    if active_moved {
        let index = first_removed_index.unwrap_or(0);
        runtime.data.active_tab_id = runtime
            .data
            .tabs
            .get(index)
            .or_else(|| runtime.data.tabs.last())
            .map(|tab| tab.id.clone());
    }
    runtime.shell_mode = runtime
        .data
        .active_tab_id
        .as_ref()
        .and_then(|id| runtime.data.tabs.iter().find(|tab| &tab.id == id))
        .is_none_or(|tab| tab.url == NEW_TAB_URL);
    sort_tabs(&mut runtime.data.tabs);

    if let Some(workspace) = runtime
        .data
        .workspaces
        .iter_mut()
        .find(|workspace| workspace.id == workspace_id)
    {
        workspace.tabs.extend(moved.iter().cloned());
    }
    Ok(moved)
}

/// 批量关闭：进 recently_closed（截断 20）、激活转移、空则补新标签页。返回被关闭的标签。
pub(crate) fn batch_close_data(
    runtime: &mut RuntimeData,
    tab_ids: &[String],
) -> Result<Vec<TabRecord>, String> {
    if tab_ids.is_empty() {
        return Ok(Vec::new());
    }
    let first_removed_index = runtime
        .data
        .tabs
        .iter()
        .position(|tab| tab_ids.contains(&tab.id));
    let active_closed = runtime
        .data
        .active_tab_id
        .as_ref()
        .is_some_and(|id| tab_ids.contains(id));

    let mut closed: Vec<TabRecord> = Vec::new();
    runtime.data.tabs.retain(|tab| {
        if tab_ids.contains(&tab.id) {
            closed.push(tab.clone());
            false
        } else {
            true
        }
    });
    if closed.is_empty() {
        return Ok(closed);
    }
    for tab in &mut closed {
        tab.loading = false;
        tab.loaded = false;
        tab.hibernated = false;
    }
    runtime
        .data
        .recently_closed
        .splice(0..0, closed.iter().cloned());
    runtime.data.recently_closed.truncate(20);

    if runtime.data.tabs.is_empty() {
        runtime
            .data
            .tabs
            .push(TabRecord::new(NEW_TAB_URL.into(), "新标签页".into(), false));
    }
    if active_closed {
        let index = first_removed_index.unwrap_or(0);
        runtime.data.active_tab_id = runtime
            .data
            .tabs
            .get(index)
            .or_else(|| runtime.data.tabs.last())
            .map(|tab| tab.id.clone());
    }
    runtime.shell_mode = runtime
        .data
        .active_tab_id
        .as_ref()
        .and_then(|id| runtime.data.tabs.iter().find(|tab| &tab.id == id))
        .is_none_or(|tab| tab.url == NEW_TAB_URL);
    sort_tabs(&mut runtime.data.tabs);
    Ok(closed)
}

/// 批量收藏：http(s) 标签按 URL 去重写入书签（已存在则更新标题）。
pub(crate) fn batch_bookmark_data(runtime: &mut RuntimeData, tab_ids: &[String]) {
    for tab in runtime
        .data
        .tabs
        .iter()
        .filter(|tab| tab_ids.contains(&tab.id))
    {
        if !tab.url.starts_with("http://") && !tab.url.starts_with("https://") {
            continue;
        }
        let title = if tab.title.trim().is_empty() {
            tab.url.clone()
        } else {
            tab.title.clone()
        };
        if let Some(existing) = runtime
            .data
            .bookmarks
            .iter_mut()
            .find(|item| item.url == tab.url)
        {
            existing.title = title;
        } else {
            runtime.data.bookmarks.insert(
                0,
                Bookmark {
                    id: uuid::Uuid::new_v4().simple().to_string(),
                    title,
                    url: tab.url.clone(),
                    created_at: chrono::Utc::now(),
                },
            );
        }
    }
}

pub(crate) fn batch_mute_data(runtime: &mut RuntimeData, tab_ids: &[String], muted: bool) {
    for tab in runtime
        .data
        .tabs
        .iter_mut()
        .filter(|tab| tab_ids.contains(&tab.id))
    {
        tab.muted = muted;
    }
}

fn close_all_tab_webviews(app: &AppHandle) {
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
}

/// 装载完成后的收尾：按激活标签恢复 WebView（新标签页则先扩幅）。
fn restore_active_tab_webview(app: &AppHandle) {
    let state = app.state::<AppState>();
    let active = state
        .inner
        .lock()
        .ok()
        .and_then(|runtime| runtime.data.active_tab_id.clone());
    let Some(id) = active else {
        return;
    };
    let is_new_tab = state
        .inner
        .lock()
        .ok()
        .and_then(|runtime| {
            runtime
                .data
                .tabs
                .iter()
                .find(|tab| tab.id == id)
                .map(|tab| tab.url == NEW_TAB_URL)
        })
        .unwrap_or(false);
    if is_new_tab {
        // 新标签页：先扩幅再下发快照，避免收缩视口内的首帧布局跳动。
        let _ = set_shell_expanded(app, true);
        hide_all_tabs(app);
    } else {
        hide_all_tabs(app);
        if ensure_tab_webview(app, &id).is_ok() {
            show_active_tab(app);
        }
    }
}

pub fn create_workspace(app: &AppHandle, name: &str) -> Result<AppSnapshot, String> {
    validate_workspace_name(name)?;
    let state = app.state::<AppState>();
    close_all_tab_webviews(app);
    state.mutate_result(|runtime| create_workspace_data(runtime, name))?;
    restore_active_tab_webview(app);
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub fn rename_workspace(
    app: &AppHandle,
    workspace_id: &str,
    name: &str,
) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    state.mutate_result(|runtime| rename_workspace_data(runtime, workspace_id, name))?;
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub fn remove_workspace(app: &AppHandle, workspace_id: &str) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    state.mutate_result(|runtime| remove_workspace_data(runtime, workspace_id))?;
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub fn switch_workspace(app: &AppHandle, workspace_id: &str) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    {
        let guard = state
            .inner
            .lock()
            .map_err(|_| "应用状态无法读取".to_string())?;
        if guard.data.active_workspace_id.as_deref() == Some(workspace_id) {
            return Ok(state.snapshot());
        }
        if !guard
            .data
            .workspaces
            .iter()
            .any(|workspace| workspace.id == workspace_id)
        {
            return Err("工作区不存在".into());
        }
    }
    close_all_tab_webviews(app);
    state.mutate_result(|runtime| {
        stash_current_tabs(runtime);
        load_workspace_tabs(runtime, workspace_id)
    })?;
    restore_active_tab_webview(app);
    emit_snapshot(app);
    Ok(state.snapshot())
}

pub fn move_tab_to_workspace(
    app: &AppHandle,
    tab_id: &str,
    workspace_id: &str,
) -> Result<AppSnapshot, String> {
    apply_tab_batch(
        app,
        &TabBatchUpdate {
            action: "move".into(),
            tab_ids: vec![tab_id.to_string()],
            workspace_id: Some(workspace_id.to_string()),
        },
    )
}

pub fn apply_tab_batch(app: &AppHandle, update: &TabBatchUpdate) -> Result<AppSnapshot, String> {
    let state = app.state::<AppState>();
    match update.action.as_str() {
        "close" => {
            let closed = state.mutate_result(|runtime| {
                let closed = batch_close_data(runtime, &update.tab_ids)?;
                Ok(closed)
            })?;
            for tab in &closed {
                if let Some(webview) = app.get_webview(&tab_label(&tab.id)) {
                    let _ = webview.close();
                }
            }
            restore_active_tab_webview(app);
        }
        "bookmark" => {
            state.mutate(|runtime| batch_bookmark_data(runtime, &update.tab_ids))?;
        }
        "mute" | "unmute" => {
            let muted = update.action == "mute";
            state.mutate(|runtime| batch_mute_data(runtime, &update.tab_ids, muted))?;
            let script = if muted {
                MUTE_TAB_SCRIPT
            } else {
                UNMUTE_TAB_SCRIPT
            };
            for id in &update.tab_ids {
                if let Some(webview) = app.get_webview(&tab_label(id)) {
                    let _ = webview.eval(script);
                }
            }
        }
        "move" => {
            let workspace_id = update
                .workspace_id
                .as_deref()
                .ok_or_else(|| "缺少目标工作区".to_string())?;
            let (moved, active_changed) = state.mutate_result(|runtime| {
                let before_active = runtime.data.active_tab_id.clone();
                let moved = move_tabs_data(runtime, &update.tab_ids, workspace_id)?;
                let active_changed = before_active != runtime.data.active_tab_id;
                Ok((moved, active_changed))
            })?;
            for tab in &moved {
                if let Some(webview) = app.get_webview(&tab_label(&tab.id)) {
                    let _ = webview.close();
                }
            }
            if active_changed {
                restore_active_tab_webview(app);
            }
        }
        _ => return Err("未知的批量操作".into()),
    }
    emit_snapshot(app);
    Ok(state.snapshot())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PersistedData;

    fn runtime_with(
        tabs: Vec<TabRecord>,
        active: Option<String>,
        workspaces: Vec<WorkspaceRecord>,
        active_ws: Option<String>,
    ) -> RuntimeData {
        RuntimeData {
            data: PersistedData {
                tabs,
                active_tab_id: active,
                workspaces,
                active_workspace_id: active_ws,
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

    fn tab(url: &str, title: &str) -> TabRecord {
        let mut record = TabRecord::new(url.into(), title.into(), false);
        record.last_active_at = chrono::Utc::now();
        record
    }

    #[test]
    fn stash_and_load_round_trip_preserves_tabs_and_active() {
        let a = tab("https://a.example", "A");
        let mut b = tab("https://b.example", "B");
        b.pinned = true;
        let ws = WorkspaceRecord::new("工作");
        let ws_id = ws.id.clone();
        let other = WorkspaceRecord::new("学习");
        let other_id = other.id.clone();
        let mut runtime = runtime_with(
            vec![a.clone(), b.clone()],
            Some(b.id.clone()),
            vec![ws, other],
            Some(ws_id.clone()),
        );

        // 切走：暂存当前工作区
        stash_current_tabs(&mut runtime);
        assert!(runtime.data.tabs.is_empty());
        load_workspace_tabs(&mut runtime, &other_id).expect("load workspace");
        assert!(runtime.data.tabs[0].url == NEW_TAB_URL);

        // 切回：标签与激活位恢复，pinned 保留
        stash_current_tabs(&mut runtime);
        load_workspace_tabs(&mut runtime, &ws_id).expect("load workspace");
        assert_eq!(runtime.data.tabs.len(), 2);
        assert_eq!(runtime.data.active_tab_id.as_deref(), Some(b.id.as_str()));
        assert_eq!(
            runtime.data.workspaces[0]
                .tabs
                .iter()
                .find(|t| t.id == b.id)
                .map(|t| t.pinned),
            Some(true)
        );
    }

    #[test]
    fn load_empty_workspace_creates_new_tab() {
        let ws = WorkspaceRecord::new("工作");
        let id = ws.id.clone();
        let mut runtime = runtime_with(Vec::new(), None, vec![ws], Some(id.clone()));
        stash_current_tabs(&mut runtime);
        load_workspace_tabs(&mut runtime, &id).expect("load workspace");
        assert_eq!(runtime.data.tabs.len(), 1);
        assert_eq!(runtime.data.tabs[0].url, NEW_TAB_URL);
        assert!(runtime.shell_mode);
        assert_eq!(
            runtime.data.active_workspace_id.as_deref(),
            Some(id.as_str())
        );
    }

    #[test]
    fn create_workspace_switches_to_new_one() {
        let a = tab("https://a.example", "A");
        let mut runtime = runtime_with(vec![a.clone()], Some(a.id.clone()), Vec::new(), None);
        // normalize 兜底：无工作区时补默认
        if runtime.data.workspaces.is_empty() {
            runtime
                .data
                .workspaces
                .push(WorkspaceRecord::new(crate::state::DEFAULT_WORKSPACE_NAME));
        }
        runtime.data.active_workspace_id = runtime.data.workspaces.first().map(|w| w.id.clone());

        let id = create_workspace_data(&mut runtime, " 临时查询 ").expect("create workspace");
        assert_eq!(runtime.data.workspaces.len(), 2);
        assert_eq!(runtime.data.workspaces[1].name, "临时查询");
        assert_eq!(
            runtime.data.active_workspace_id.as_deref(),
            Some(id.as_str())
        );
        assert_eq!(runtime.data.tabs.len(), 1);
        assert_eq!(runtime.data.tabs[0].url, NEW_TAB_URL);
        // 旧工作区记录已暂存原标签
        assert_eq!(runtime.data.workspaces[0].tabs.len(), 1);
        assert_eq!(runtime.data.workspaces[0].tabs[0].id, a.id);
    }

    #[test]
    fn remove_workspace_guards_last_and_active() {
        let ws = WorkspaceRecord::new("唯一");
        let id = ws.id.clone();
        let mut runtime = runtime_with(Vec::new(), None, vec![ws], Some(id.clone()));
        assert!(remove_workspace_data(&mut runtime, &id).is_err());

        let other = WorkspaceRecord::new("第二");
        runtime.data.workspaces.push(other);
        // 当前激活的不能删
        assert!(remove_workspace_data(&mut runtime, &id).is_err());
        // 切到第二个后可删
        stash_current_tabs(&mut runtime);
        let second_id = runtime.data.workspaces[1].id.clone();
        load_workspace_tabs(&mut runtime, &second_id).unwrap();
        assert!(remove_workspace_data(&mut runtime, &id).is_ok());
        assert_eq!(runtime.data.workspaces.len(), 1);
    }

    #[test]
    fn move_tabs_transfers_active_like_close() {
        let a = tab("https://a.example", "A");
        let b = tab("https://b.example", "B");
        let c = tab("https://c.example", "C");
        let ws = WorkspaceRecord::new("工作");
        let ws_id = ws.id.clone();
        let target = WorkspaceRecord::new("目标");
        let target_id = target.id.clone();
        let mut runtime = runtime_with(
            vec![a.clone(), b.clone(), c.clone()],
            Some(c.id.clone()),
            vec![ws, target],
            Some(ws_id),
        );

        // 移动激活的 c：激活落到移除位置（末尾回退前一个 b）
        let moved = move_tabs_data(&mut runtime, std::slice::from_ref(&c.id), &target_id)
            .expect("move tabs");
        assert_eq!(moved.len(), 1);
        assert_eq!(runtime.data.active_tab_id.as_deref(), Some(b.id.as_str()));
        assert_eq!(
            runtime
                .data
                .workspaces
                .iter()
                .find(|w| w.id == target_id)
                .map(|w| w.tabs.len()),
            Some(1)
        );

        // 移动到当前工作区报错
        let active_ws = runtime.data.active_workspace_id.clone().unwrap();
        assert!(move_tabs_data(&mut runtime, std::slice::from_ref(&a.id), &active_ws).is_err());
    }

    #[test]
    fn batch_close_moves_to_recently_closed_and_fixes_active() {
        let a = tab("https://a.example", "A");
        let b = tab("https://b.example", "B");
        let c = tab("https://c.example", "C");
        let mut runtime = runtime_with(
            vec![a.clone(), b.clone(), c.clone()],
            Some(b.id.clone()),
            Vec::new(),
            None,
        );

        let closed = batch_close_data(&mut runtime, &[b.id.clone(), c.id.clone()]).expect("close");
        assert_eq!(closed.len(), 2);
        assert_eq!(runtime.data.tabs.len(), 1);
        assert_eq!(runtime.data.active_tab_id.as_deref(), Some(a.id.as_str()));
        assert_eq!(runtime.data.recently_closed.len(), 2);
        assert!(runtime.data.recently_closed.iter().any(|t| t.id == b.id));
        // 全部关闭 → 补新标签页
        batch_close_data(&mut runtime, std::slice::from_ref(&a.id)).expect("close all");
        assert_eq!(runtime.data.tabs.len(), 1);
        assert_eq!(runtime.data.tabs[0].url, NEW_TAB_URL);
    }

    #[test]
    fn batch_bookmark_dedupes_by_url() {
        let mut a = tab("https://a.example/x", "标题一");
        let mut runtime = runtime_with(vec![a.clone()], Some(a.id.clone()), Vec::new(), None);
        batch_bookmark_data(&mut runtime, &[a.id.clone()]);
        assert_eq!(runtime.data.bookmarks.len(), 1);
        a.title = "新标题".into();
        runtime.data.tabs[0].title = "新标题".into();
        batch_bookmark_data(&mut runtime, &[a.id.clone()]);
        assert_eq!(runtime.data.bookmarks.len(), 1);
        assert_eq!(runtime.data.bookmarks[0].title, "新标题");
    }

    #[test]
    fn batch_mute_sets_flags() {
        let a = tab("https://a.example", "A");
        let b = tab("https://b.example", "B");
        let mut runtime = runtime_with(
            vec![a.clone(), b.clone()],
            Some(a.id.clone()),
            Vec::new(),
            None,
        );
        batch_mute_data(&mut runtime, &[a.id.clone(), b.id.clone()], true);
        assert!(runtime.data.tabs.iter().all(|t| t.muted));
        batch_mute_data(&mut runtime, std::slice::from_ref(&b.id), false);
        assert!(!runtime.data.tabs[1].muted);
    }
}
