<div align="center">

<img src="src-tauri/icons/128x128.png" alt="QuickPane" width="96" height="96" />

# QuickPane

[中文](README.md)

**A lightweight Windows browser built for instant show/hide.** One global hotkey summons the whole browser — tabs, sessions and all — and the same hotkey makes it vanish just as fast.

<sub>// Tauri 2 + WebView2 · Windows 10/11</sub>

<br />

![Tauri](https://img.shields.io/badge/Tauri-2-24c8db?logo=tauri&logoColor=white)
![React](https://img.shields.io/badge/React-19-61dafb?logo=react&logoColor=111)
![TypeScript](https://img.shields.io/badge/TypeScript-strict-3178c6?logo=typescript&logoColor=white)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078d4?logo=windows&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green)

</div>

---

## The problem

Digging the browser out of the taskbar, waiting for it to load after an app switch — QuickPane turns the whole browser into a hotkey-level citizen: one press and everything is there, another press and it's gone, with media muted and your previous foreground window restored.

## Features

- ⚡ **Instant show/hide** — configurable global shortcut with modifier-key validation; the previous foreground window is restored when QuickPane hides
- 🗂 **Persistent multi-tab session** — one window, tabs survive restarts
- 🧩 **Browser extensions** — install, enable, disable and remove unpacked extensions from the settings page (WebView2 Profile based, see [extension support](docs/extensions.md))
- 🔍 **Full browsing basics** — URL-or-search address bar, history, bookmarks, downloads, recently closed tabs
- 🔇 **Media auto-mute while hidden** — plus on-demand session loading after background start
- 📌 **Tray integration** — tray toggle, close-to-tray
- 🔒 **Optional app lock** — Argon2id lock on cold start and after Windows lock
- 🚀 **Optional startup with Windows** — with window-state persistence
- 🔄 **Signed auto-update** — GitHub Releases with minisign updater signatures, optional self-hosted MinIO mirror (see [release & auto-update](docs/release.md))

## Install

Download the latest NSIS installer from [Releases](https://github.com/zxbdzh/QuickPane/releases). The installer detects the Windows WebView2 Evergreen Runtime and downloads it when needed.

## Development

Requirements: Node.js 20+, pnpm, Rust toolchain, WebView2 Runtime (Windows 10/11).

```bash
pnpm install
pnpm tauri dev
```

Architecture boundaries and conventions: [AGENTS.md](AGENTS.md).

## License

[MIT](./LICENSE)
