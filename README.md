# QuickPane

**A lightweight Windows browser built for instant show/hide.** One global hotkey summons the whole browser — tabs, sessions and all — and the same hotkey makes it vanish just as fast.

一个为「随叫随收」打造的轻量 Windows 浏览器：一个全局快捷键，整个浏览器连同所有标签页瞬间出现，再按一下瞬间消失。

[![Tauri](https://img.shields.io/badge/Tauri-2-24c8db?logo=tauri&logoColor=white)](https://v2.tauri.app)
[![React](https://img.shields.io/badge/React-19-61dafb?logo=react&logoColor=111)](https://react.dev)
[![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078d4?logo=windows&logoColor=white)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](./LICENSE)

## Features / 功能

- ⚡ **Instant show/hide / 即时显隐** — configurable global shortcut with modifier-key validation; the previous foreground window is restored when QuickPane hides
- 🗂 **Persistent multi-tab session / 多标签会话持久化** — one window, tabs survive restarts
- 🧩 **Browser extensions / 扩展支持** — install, enable, disable and remove unpacked extensions from the settings page (WebView2 Profile based, see [docs/extensions.md](docs/extensions.md))
- 🔍 **Full browsing basics / 完整浏览能力** — URL-or-search address bar, history, bookmarks, downloads, recently closed tabs
- 🔇 **Media auto-mute while hidden / 隐藏时自动静音** — plus on-demand session loading after background start
- 📌 **Tray integration / 托盘集成** — tray toggle, close-to-tray
- 🔒 **Optional app lock / 可选应用锁** — Argon2id lock on cold start and after Windows lock
- 🚀 **Optional startup with Windows / 可选开机自启** — with window-state persistence
- 🔄 **Signed auto-update / 签名自动更新** — GitHub Releases with minisign updater signatures, optional self-hosted MinIO mirror (see [docs/release.md](docs/release.md))

## Install / 安装

Download the latest NSIS installer from [Releases](https://github.com/zxbdzh/QuickPane/releases). The installer detects the Windows WebView2 Evergreen Runtime and downloads it when needed.

从 [Releases](https://github.com/zxbdzh/QuickPane/releases) 下载最新安装包（NSIS）。安装器会自动检测并按需下载 WebView2 Evergreen Runtime。

## Development / 开发

Requirements: Node.js 20+, pnpm, Rust toolchain, WebView2 Runtime (Windows 10/11).

```bash
pnpm install
pnpm tauri dev
```

## License

[MIT](./LICENSE)
