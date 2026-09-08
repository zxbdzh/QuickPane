<div align="center">

<img src="src-tauri/icons/128x128.png" alt="QuickPane" width="88" height="88" />

# QuickPane

**按一下快捷键，浏览器立即出现；再按一下，回到原来的工作。**

一个为 Windows 打造的轻量级快捷浏览器。保留标签页、历史记录、书签和扩展，却不会一直占据任务栏。

[下载最新版本](https://github.com/zxbdzh/QuickPane/releases/latest) · [报告问题](https://github.com/zxbdzh/QuickPane/issues) · [English](README.en.md)

![Tauri](https://img.shields.io/badge/Tauri-2-24c8db?logo=tauri&logoColor=white)
![React](https://img.shields.io/badge/React-19-61dafb?logo=react&logoColor=111)
![TypeScript](https://img.shields.io/badge/TypeScript-strict-3178c6?logo=typescript&logoColor=white)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078d4?logo=windows&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green)

</div>

<p align="center">
  <img src="docs/assets/quickpane-new-tab.png" alt="QuickPane new tab page" width="720" />
</p>

<p align="center"><sub>QuickPane 界面：一个随时待命的轻量级浏览器。</sub></p>

<p align="center">
  <img src="docs/assets/quickpane-demo.gif" alt="QuickPane 浏览演示" width="720" />
</p>

---

## 为什么是 QuickPane

浏览器应该在需要时出现，而不是一直占据任务栏和注意力。

QuickPane 运行在后台，通过一个全局快捷键显示或隐藏整个浏览器窗口。标签页和浏览会话持续保留，隐藏时自动静音媒体，并在收起后恢复之前的前台窗口。

## 界面一览

<p align="center">
  <img src="docs/assets/quickpane-browser.png" alt="QuickPane 浏览页面" width="48%" />
  <img src="docs/assets/quickpane-settings.png" alt="QuickPane 设置页面" width="48%" />
</p>
<p align="center">
  <sub>网页浏览：在独立标签页中打开网站，支持地址栏导航、后退、前进和刷新。</sub>
  <sub>设置中心：自定义老板键、面板快捷键、主页、搜索引擎、标签休眠、代理和应用锁。</sub>
</p>
<p align="center">
  <img src="docs/assets/quickpane-extensions.png" alt="QuickPane 扩展管理" width="48%" />
  <img src="docs/assets/quickpane-menu.png" alt="QuickPane 应用菜单" width="48%" />
</p>
<p align="center">
  <sub>扩展管理：安装、启用、停用和移除未打包的浏览器扩展。</sub>
  <sub>快捷菜单：快速访问历史记录、书签、下载、标签管理、锁定、扩展和设置。</sub>
</p>

## GitHub Stars

[![GitHub Stars](https://img.shields.io/github/stars/zxbdzh/QuickPane?style=flat)](https://github.com/zxbdzh/QuickPane/stargazers)

欢迎 Star QuickPane，Star 数会由 GitHub 自动更新。

## 快速开始

1. 从 [Releases](https://github.com/zxbdzh/QuickPane/releases/latest) 下载 Windows 安装包。
2. 启动 QuickPane，在设置里配置老板键（显示 / 隐藏窗口）。未配置时可用托盘图标显示或隐藏。
3. 按 `Ctrl+K` 或默认的 `Ctrl+Shift+A` 打开快速切换面板；在地址栏输入网址或搜索关键词。
4. 按老板键或 `Esc` 隐藏窗口，稍后继续上次的浏览会话。

## 快捷键

老板键（显示 / 隐藏）和快速切换面板快捷键都可以在设置中自定义。组合键至少需要包含一个修饰键和一个普通按键，例如 `Alt+Q`、`Ctrl+Alt+B`。`Ctrl+K` 始终可以打开快速切换面板。

| 动作 | 快捷键 |
| --- | --- |
| 显示 / 隐藏窗口 | 设置中的老板键；未配置时用托盘图标 |
| 隐藏窗口 | `Esc` |
| 快速切换面板 | `Ctrl+K` 或设置中的面板快捷键（默认 `Ctrl+Shift+A`） |
| 最近关闭的标签页 | 标签栏右侧时钟按钮，或在快速切换面板中筛选 |
| 切换到下一个标签页 | `Ctrl+Tab` |
| 切换到上一个标签页 | `Ctrl+Shift+Tab` |
| 新建标签页 | `Ctrl+T` |
| 恢复关闭的标签页 | `Ctrl+Shift+T` |
| 关闭当前标签页 | `Ctrl+W` |
| 后退 | `Alt+Left` |
| 前进 | `Alt+Right` |
| 刷新 | `Ctrl+R` |
| 聚焦地址栏 | `Ctrl+L` |
| 收藏当前页面 | `Ctrl+D` |
| 打开历史记录 | `Ctrl+H` |
| 打开下载记录 | `Ctrl+J` |
| 页面内查找 | `Ctrl+F` |
| 放大 / 缩小 / 重置 | `Ctrl+=` / `Ctrl+-` / `Ctrl+0` |

## 功能

### 快速切换与工作区

- 按 `Ctrl+K` 打开快速切换面板，搜索当前标签页、最近关闭、工作区、会话快照、书签和历史记录。
- 支持中文标题拼音检索、键盘上下选择和回车跳转。
- 地址栏支持 `t` 标签页、`b` 书签、`h` 历史记录来源检索，并可复制、关闭或移动标签页。
- 点左上角工作区名称，可新建、切换、重命名或删除工作区；当前工作区会持久化保存。
- 在工作区菜单选择「存为快照」，或打开右上角菜单中的「标签管理」保存当前标签组。之后可在快速切换面板搜索快照，就地恢复或在新工作区恢复。
- 右上角菜单进入「标签管理」，可按标题、网址和域名筛选，并批量收藏、静音、移动或关闭标签页。

### 随叫随收

- 全局快捷键即时显示或隐藏窗口
- 隐藏时自动静音网页媒体
- 收起后自动恢复之前的前台窗口
- 支持托盘运行、关闭时最小化到托盘

### 浏览会话

- 单窗口多标签页
- 标签页、历史记录、书签、最近关闭页面、工作区和会话快照持久化保存
- 地址栏支持网址直达、搜索，以及标签页 / 书签 / 历史来源检索
- 支持普通网页链接，以及在当前浏览器会话中打开新标签
- 支持下载、页面缩放和后台标签休眠

### 浏览器能力

- 基于 Windows WebView2
- 支持未打包浏览器扩展的安装、启用、停用和卸载（不支持直接安装 CRX）
- 设置中可配置标签休眠、网络代理、应用锁、主题、快捷站点和签名自动更新
- 可选标签休眠：释放长时间未使用的后台 WebView，切换回来时重新加载
- 可选应用锁，使用 Argon2id 保护界面入口；不加密整个 WebView2 数据目录
- 可选随 Windows 启动，启动后只驻留后台

## 安装要求

- Windows 10 或 Windows 11
- Windows WebView2 Evergreen Runtime
- x64 Windows

安装器会检测 WebView2 Runtime；如果系统缺少运行环境，会按需下载并安装。

## 常见问题

### QuickPane 是完整浏览器吗？

QuickPane 是一个轻量级的快捷浏览器，适合临时查资料、查看文档和保留短期浏览会话。它不试图替代 Chrome、Edge 或 Firefox 的完整生态。

### 我的数据保存在哪里？

标签页、设置、历史记录、书签、工作区和会话快照保存在 `%APPDATA%\QuickPane\quickpane.json`。网页缓存、Cookie 和登录态使用同一应用数据目录下的独立 WebView2 数据目录。

### 锁屏后数据会泄露吗？

应用锁只保护界面入口。锁定或首次设置密码时，前端快照会脱敏，历史、书签、标签页和会话快照不会通过 IPC 发给未解锁界面。它不加密磁盘上的 `quickpane.json` 或 WebView2 数据目录。

### 为什么需要 WebView2？

QuickPane 使用 Windows 原生 WebView2 渲染网页。这样可以复用系统浏览器内核，同时保持应用体积和资源占用较小。

### 如何安装扩展？

只支持未打包扩展文件夹（目录内需有 `manifest.json`）。在扩展页选择文件夹安装；CRX 需要先自行解包。详见 [docs/extensions.md](docs/extensions.md)。

### 如何报告问题？

请在 [GitHub Issues](https://github.com/zxbdzh/QuickPane/issues) 提交问题，并附上 Windows 版本、QuickPane 版本、复现步骤和相关截图。

## 开发

环境要求：Node.js 20+、pnpm、Rust 工具链和 WebView2 Runtime。

```bash
npm install
npm run tauri dev
```

常用检查：

```bash
npm test
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

开发模式下，主窗口 WebView2 默认打开 CDP：`http://127.0.0.1:9222/json`。用 `QUICKPANE_CDP_PORT` 改端口，设为 `0` 则关闭。生产构建默认不开启。标签页 WebView 与主窗口是两套环境，不能共用该端口。

项目架构边界和目录约定见 [AGENTS.md](AGENTS.md)。发布与自动更新流程见 [docs/release.md](docs/release.md)。

## 参与贡献

欢迎提交 Bug 修复、文档改进和功能建议。提交较大改动前，建议先通过 [Issues](https://github.com/zxbdzh/QuickPane/issues) 讨论方案。

## 许可证

[MIT](LICENSE)
