# QuickPane

**为「随叫随收」而生的轻量 Windows 浏览器。** 一个全局快捷键，整个浏览器连同所有标签页瞬间出现，再按一下瞬间消失。

[English](README.en.md)

[![Tauri](https://img.shields.io/badge/Tauri-2-24c8db?logo=tauri&logoColor=white)](https://v2.tauri.app)
[![React](https://img.shields.io/badge/React-19-61dafb?logo=react&logoColor=111)](https://react.dev)
[![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078d4?logo=windows&logoColor=white)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](./LICENSE)

## 功能

- ⚡ **即时显隐** — 全局快捷键可自定义（带修饰键校验）；隐藏时自动还原之前的前台窗口
- 🗂 **多标签会话持久化** — 单窗口多标签，重启后会话完整恢复
- 🧩 **浏览器扩展支持** — 设置页可安装、启停、卸载未打包扩展（基于 WebView2 Profile，见 [docs/extensions.md](docs/extensions.md)）
- 🔍 **完整浏览能力** — 地址栏支持 URL 直达或搜索，历史记录、书签、下载、最近关闭标签页
- 🔇 **隐藏时自动静音媒体** — 后台启动后按需加载会话
- 📌 **托盘集成** — 托盘开关、关闭最小化到托盘
- 🔒 **可选应用锁** — 冷启动与 Windows 锁屏后需 Argon2id 密码解锁
- 🚀 **可选开机自启** — 并持久化窗口状态
- 🔄 **签名自动更新** — GitHub Release + minisign 更新签名，可选自建 MinIO 镜像（见 [docs/release.md](docs/release.md)）

## 安装

从 [Releases](https://github.com/zxbdzh/QuickPane/releases) 下载最新安装包（NSIS）。安装器会自动检测并按需下载 Windows WebView2 Evergreen Runtime。

## 开发

环境要求：Node.js 20+、pnpm、Rust 工具链、WebView2 Runtime（Windows 10/11）。

```bash
pnpm install
pnpm tauri dev
```

## 许可证

[MIT](./LICENSE)
