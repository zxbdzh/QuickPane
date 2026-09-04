import { invoke } from "@tauri-apps/api/core";
import type {
  AppSnapshot,
  BrowserExtension,
  ProxyMode,
  QuickLink,
} from "./types";

export type UpdateInfo = {
  version: string;
  notes: string | null;
  pubDate: string | null;
};

export type UpdateProgress = {
  downloaded: number;
  total: number | null;
};

export const api = {
  snapshot: () => invoke<AppSnapshot>("get_snapshot"),
  newTab: (url?: string, activate = true) =>
    invoke<AppSnapshot>("new_tab", { url: url ?? null, activate }),
  selectTab: (tabId: string) => invoke<AppSnapshot>("select_tab", { tabId }),
  removeTab: (tabId: string) => invoke<AppSnapshot>("remove_tab", { tabId }),
  setTabPinned: (tabId: string, pinned: boolean) =>
    invoke<AppSnapshot>("set_tab_pinned", { tabId, pinned }),
  setTabMuted: (tabId: string, muted: boolean) =>
    invoke<AppSnapshot>("set_tab_muted", { tabId, muted }),
  restoreClosedTab: (tabId?: string) =>
    invoke<AppSnapshot>("restore_closed_tab", { tabId: tabId ?? null }),
  navigate: (tabId: string, input: string) =>
    invoke<AppSnapshot>("navigate", { tabId, input }),
  reload: () => invoke<void>("reload"),
  back: () => invoke<void>("go_back"),
  forward: () => invoke<void>("go_forward"),
  find: () => invoke<void>("find_in_page"),
  zoom: (scale: number) => invoke<void>("zoom_page", { scale }),
  showShell: (visible: boolean) =>
    invoke<AppSnapshot>("show_shell", { visible }),
  hide: () => invoke<void>("hide_to_tray"),
  exit: () => invoke<void>("exit_app"),
  setShortcut: (shortcut: string) =>
    invoke<AppSnapshot>("set_global_shortcut", { shortcut }),
  updateSettings: (update: {
    shortcut: string | null;
    tabSearchShortcut: string;
    recentlyClosedShortcut: string;
    autostart: boolean;
    homeUrl: string;
    searchTemplate: string;
    historyDays: number;
    lockOnSystemLock: boolean;
    autoLockAfterHideSeconds: number;
    quickLinks: QuickLink[];
    proxyMode: ProxyMode;
    proxyUrl: string;
  }) => invoke<AppSnapshot>("update_settings", { update }),
  addBookmark: (title: string, url: string) =>
    invoke<AppSnapshot>("add_bookmark", { title, url }),
  removeBookmark: (bookmarkId: string) =>
    invoke<AppSnapshot>("remove_bookmark", { bookmarkId }),
  clearHistory: () => invoke<AppSnapshot>("clear_history"),
  clearDownloads: () => invoke<AppSnapshot>("clear_downloads"),
  openDownload: (path: string) => invoke<void>("open_download", { path }),
  clearSiteData: () => invoke<void>("clear_site_data"),
  setPassword: (newPassword: string, currentPassword?: string) =>
    invoke<AppSnapshot>("set_app_password", {
      newPassword,
      currentPassword: currentPassword ?? null,
    }),
  disablePassword: (currentPassword: string) =>
    invoke<AppSnapshot>("disable_app_password", { currentPassword }),
  unlock: (password: string) => invoke<AppSnapshot>("unlock_app", { password }),
  skipPasswordSetup: () => invoke<AppSnapshot>("skip_password_setup"),
  lockNow: () => invoke<void>("lock_now"),
  listExtensions: () => invoke<BrowserExtension[]>("list_extensions"),
  installExtension: () => invoke<BrowserExtension[]>("install_extension"),
  removeExtension: (extensionId: string) =>
    invoke<BrowserExtension[]>("remove_extension", { extensionId }),
  setExtensionEnabled: (extensionId: string, enabled: boolean) =>
    invoke<BrowserExtension[]>("set_extension_enabled", {
      extensionId,
      enabled,
    }),
  showTabMenuWindow: (tabId: string, x: number, y: number) =>
    invoke<void>("show_tab_menu_window", { tabId, x, y }),
  setShellExpanded: (expanded: boolean) =>
    invoke<void>("set_shell_expanded", { expanded }),
  showExtensionPopup: (url: string, x?: number, y?: number) =>
    invoke<void>("show_extension_popup", { url, x: x ?? null, y: y ?? null }),
  toggleExtensionPin: (extensionId: string, pinned: boolean) =>
    invoke<AppSnapshot>("toggle_extension_pin", { extensionId, pinned }),
  checkUpdate: () => invoke<UpdateInfo | null>("check_update"),
  installUpdate: () => invoke<void>("install_update"),
};
