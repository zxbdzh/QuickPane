<div align="center">

<img src="src-tauri/icons/128x128.png" alt="QuickPane" width="96" height="96" />

# QuickPane

[English](README.en.md)

**为「随叫随收」而生的轻量 Windows 浏览器。** 一个全局快捷键，整个浏览器连同所有标签页瞬间出现，再按一下瞬间消失。

<sub>// Tauri 2 + WebView2 · Windows 10/11</sub>

<br />

![Tauri](https://img.shields.io/badge/Tauri-2-24c8db?logo=tauri&logoColor=white)
![React](https://img.shields.io/badge/React-19-61dafb?logo=react&logoColor=111)
![TypeScript](https://img.shields.io/badge/TypeScript-strict-3178c6?logo=typescript&logoColor=white)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078d4?logo=windows&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green)

</div>

---

## 它解决什么

浏览器窗口在任务栏里翻来翻去、切应用回来还要等它加载——QuickPane 把整个浏览器变成一个快捷键级的存在：按一下就全在了，再按一下干干净净消失，媒体自动静音，前台窗口自动还原。

## 功能

- ⚡ **即时显隐** — 全局快捷键可自定义（带修饰键校验）；隐藏时自动还原之前的前台窗口
- 🗂 **多标签会话持久化** — 单窗口多标签，重启后会话完整恢复
- 🧩 **浏览器扩展支持** — 设置页可安装、启停、卸载未打包扩展（基于 WebView2 Profile，见 [扩展支持设计](docs/extensions.md)）
- 🔍 **完整浏览能力** — 地址栏支持 URL 直达或搜索；历史记录、书签、下载、最近关闭标签页
- 🔇 **隐藏时自动静音媒体** — 后台启动后按需加载会话
- 📌 **托盘集成** — 托盘开关、关闭最小化到托盘
- 🔒 **可选应用锁** — 冷启动与 Windows 锁屏后需 Argon2id 密码解锁
- 🚀 **可选开机自启** — 并持久化窗口状态
- 🔄 **签名自动更新** — GitHub Release + minisign 更新签名，可选自建 MinIO 镜像（见 [发布与自动更新](docs/release.md)）

## 安装

从 [Releases](https://github.com/zxbdzh/QuickPane/releases) 下载最新安装包（NSIS）。安装器会自动检测并按需下载 Windows WebView2 Evergreen Runtime。

## 开发

环境要求：Node.js 20+、pnpm、Rust 工具链、WebView2 Runtime（Windows 10/11）。

```bash
pnpm install
pnpm tauri dev
```

架构边界与约定见 [AGENTS.md](AGENTS.md)。

## 许可证

[MIT](./LICENSE)
