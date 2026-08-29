# QuickPane 浏览器扩展支持设计

## 现状（已实现）

基于 WebView2 的原生扩展能力（`Profile.AddBrowserExtension`），通过 Tauri 2.11 / wry 0.55 透传的 `browser_extensions_enabled` + `extensions_path` 配置实现，无需 fork 依赖。

### 目录约定

```
%APPDATA%/QuickPane/
├── Extensions/            # 启用的扩展，每个子文件夹 = 一个未打包扩展
│   └── <扩展名>/manifest.json
└── ExtensionsDisabled/    # 禁用的扩展（wry 只加载 Extensions/，移动即禁用）
```

### 生效机制

- 每个标签 WebView 创建时开启扩展环境并指向 `Extensions/`，扩展安装进所有标签共享的 WebView2 Profile；主窗口壳（不同 data dir）不受影响。
- 重复调用 `AddBrowserExtension` 对同一扩展是幂等更新，wry 忽略加载错误。
- 安装/卸载/启停后调用 `recreate_tab_webviews` 重建标签 WebView 立即生效（Cookie 与登录态在数据目录中保留）。
- **注意**：wry 的加载器对扩展目录做 `read_dir?`，目录不存在会导致 WebView 创建失败，因此 `browser::extensions_dir` 保证目录一定存在。

### 管理命令（Rust ↔ 前端）

| 命令 | 行为 |
| --- | --- |
| `list_extensions` | 遍历两个目录，解析 manifest.json（名称支持 `__MSG_key__` i18n，图标转 data URL） |
| `install_extension` | Rust 端弹系统文件夹选择器 → 校验 manifest.json → 递归复制 → 重建 WebView |
| `set_extension_enabled` | 在 Extensions / ExtensionsDisabled 间移动目录 → 重建 WebView |
| `remove_extension` | 删除两处目录 → 重建 WebView |

设置页「扩展」分组提供列表（图标/名称/版本/描述）、启停开关、卸载和「添加扩展」按钮。

## 已知限制

1. **仅支持未打包扩展**（含 manifest.json 的文件夹）。CRX 需先解包（CRX3 = 头部 + ZIP）。
2. **API 子集**：WebView2 支持 chrome.* 的常用部分（tabs/storage/scripting/webNavigation/downloads/contextMenus/alarms/cookies 等）；依赖浏览器 UI 的能力不可用——工具栏按钮浮出面板（action popup）、侧边栏、chrome://extensions 页均不存在，contextMenus 仅在页面内生效。
3. **加载失败静默**：wry 的完成回调忽略错误，manifest 非法的文件夹只是不生效，列表界面无法反映加载失败（列表解析与 WebView2 实际加载是两套逻辑）。
4. **WebView2 Runtime 版本**：需 1.0.2210.55+（`ICoreWebView2Profile7`）；Evergreen 运行时自动更新，Win10/11 默认满足。

### 面板（action popup）补救方案（已实现）

WebView2 没有工具栏可承载弹出面板，但 `chrome-extension://<id>/popup.html` 允许作为顶层页面加载（同扩展的选项页同理）。因此：

- Rust 侧按 Chromium 规则推导扩展 ID——manifest 带 `key`（商店来源解包后常见）时用公钥 DER 的 SHA-256 前 16 字节，否则回退路径哈希（UTF-16LE）；半字节统一映射 a..p。读取 manifest 的 `action.default_popup`（兼容 MV2 `browser_action`）生成 `popupUrl`。
- 扩展管理页对声明了面板的扩展显示「打开面板」按钮，点击即在新标签页打开面板页面；面板的 HTML/JS 与后台 Worker 的 chrome.runtime 通信照常工作，仅从浮窗变为页面。
- 风险点：ID 推导必须与 WebView2 内部注册的 ID 一致——带 `key` 的扩展用公钥推导，纯本地文件夹用路径哈希（路径大小写敏感）；若某扩展面板打不开，优先核对该扩展 manifest 是否含 `key` 及推导结果。

## 后续方向

- **CRX 支持**：安装命令接受 .crx，按 CRX3 格式解析头部并解压到 Extensions/。
- **加载状态反馈**：用 `with_webview` 拿到 `ICoreWebView2Profile7` 自行调用 `AddBrowserExtension`，在完成回调里记录成功/失败并同步到前端。
- **商店分发**：从 Edge 加载项商店下载 CRX 解包安装；需要处理版本更新（对比 manifest version 覆盖安装）。
- **权限展示**：解析 manifest 的 `permissions` / `host_permissions` 字段，安装前在确认对话框中展示。
