# QuickPane

QuickPane is a lightweight Windows browser built for instant show/hide switching.

## Current scope

- Tauri 2 + WebView2, Windows 10/11
- One browser window with persistent multi-tab session
- Configurable global shortcut with modifier-key validation
- Tray toggle, close-to-tray behavior, and restore of the previous foreground window
- Media mute while hidden, plus on-demand session loading after background start
- URL-or-search address bar, history, bookmarks, downloads, and recently closed tabs
- Optional Argon2id application lock on cold start and after Windows lock
- Optional Windows startup, window state persistence, and signed-update-ready installer configuration

## Development

```bash
npm install
npm run tauri dev
```

The Windows WebView2 Evergreen Runtime is detected by the installer and downloaded when needed.
