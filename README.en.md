# QuickPane

**A lightweight Windows browser built for instant show/hide.** One global hotkey summons the whole browser — tabs, sessions and all — and the same hotkey makes it vanish just as fast.

[中文](README.md)

## Features

- ⚡ **Instant show/hide** — configurable global shortcut with modifier-key validation; the previous foreground window is restored when QuickPane hides
- 🗂 **Persistent multi-tab session** — one window, tabs survive restarts
- 🧩 **Browser extensions** — install, enable, disable and remove unpacked extensions from the settings page (WebView2 Profile based, see [docs/extensions.md](docs/extensions.md))
- 🔍 **Full browsing basics** — URL-or-search address bar, history, bookmarks, downloads, recently closed tabs
- 🔇 **Media auto-mute while hidden** — plus on-demand session loading after background start
- 📌 **Tray integration** — tray toggle, close-to-tray
- 🔒 **Optional app lock** — Argon2id lock on cold start and after Windows lock
- 🚀 **Optional startup with Windows** — with window-state persistence
- 🔄 **Signed auto-update** — GitHub Releases with minisign updater signatures, optional self-hosted MinIO mirror (see [docs/release.md](docs/release.md))

## Install

Download the latest NSIS installer from [Releases](https://github.com/zxbdzh/QuickPane/releases). The installer detects the Windows WebView2 Evergreen Runtime and downloads it when needed.

## Development

Requirements: Node.js 20+, pnpm, Rust toolchain, WebView2 Runtime (Windows 10/11).

```bash
pnpm install
pnpm tauri dev
```

## License

[MIT](./LICENSE)
