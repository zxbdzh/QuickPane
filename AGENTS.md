# AGENTS.md — QuickPane

轻量级 Windows 浏览器（Tauri 2 + WebView2 + React 19），核心卖点是全局快捷键即时显示/隐藏窗口。仅面向 Windows 10/11，不跨平台。

## 常用命令

```bash
npm install
npm run tauri dev     # 开发（先起 vite @localhost:1420，再起 Rust）
npm run tauri build   # 打包安装程序
npm run build         # tsc + vite build（tsc 即类型检查）
npm test              # Node 原生测试（当前覆盖地址栏建议匹配、拼音、排序、去重与边界）
cargo check           # 在 src-tauri/ 下检查 Rust
```

验证改动需运行相应自动化检查，并通过 `npm run tauri dev` 实际操作窗口。

## 目录与架构边界

- `src/` — React 前端。`App.tsx` 是几乎全部 UI（单文件）；`api.ts` 是唯一允许 `invoke` Tauri 后端的地方；`types.ts` 定义与 Rust serde 结构对应的 `AppSnapshot` 等类型。
- `src-tauri/src/lib.rs` — 所有 `#[tauri::command]` 入口 + setup/窗口事件。命令薄封装，实际逻辑在子模块。
- `src-tauri/src/browser.rs` — 多标签页：真实 webview 的创建/激活/导航/冻结/缩放。
- `src-tauri/src/windowing.rs` — 显示/隐藏、托盘、全局快捷键、会话锁（Argon2id）。
- `src-tauri/src/state.rs` — `AppState`（Mutex 管理）与持久化设置、历史、书签。
- `src-tauri/tauri.conf.json` + `capabilities/default.json` — 窗口、打包与 Tauri 权限配置。

## 约定与注意事项

- 前后端通过 `AppSnapshot`（serde 驼峰命名）交换状态：改 Rust 状态结构时必须同步 `src/types.ts` 和 `src/api.ts`，命令名用 snake_case、参数用驼峰（Tauri 自动转换）。
- 每个标签是一个独立 WebView，通过快照/事件同步到前端；涉及 tab 生命周期的改动在 `browser.rs`，不要在命令层直接操作 webview。
- Windows 专用代码用 `#[cfg(windows)]` + `windows` crate（见 Cargo.toml 的 target 段）。
- 隐藏时静音媒体、恢复前台窗口、close-to-tray 等行为在 `windowing.rs`，改动时注意不要破坏"显示时恢复上一个前台窗口"逻辑。
- 开发运行前需本机装有 Rust toolchain 与 WebView2 运行时（Windows 11 自带）。

## Agent 技能

### Issue 跟踪器

Issue 与规格统一记录在 `zxbdzh/QuickPane` 的 GitHub Issues 中，使用 `gh` CLI 操作。详见 `docs/agents/issue-tracker.md`。

### 领域文档

本仓库采用单上下文结构；相关文件存在时，先阅读根目录的 `CONTEXT.md` 和 `docs/adr/` 下与当前工作相关的 ADR。详见 `docs/agents/domain.md`。
