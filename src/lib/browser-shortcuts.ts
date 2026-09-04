export type BrowserShortcut =
  | "escape"
  | "next-tab"
  | "previous-tab"
  | "restore-tab"
  | "new-tab"
  | "focus-address"
  | "close-tab"
  | "history"
  | "downloads"
  | "bookmark"
  | "find"
  | "zoom-in"
  | "zoom-out"
  | "zoom-reset"
  | "tab-search"
  | "recently-closed";

export type ConfiguredBrowserShortcuts = {
  tabSearch: string;
  recentlyClosed: string;
};

type ShortcutKeyEvent = Pick<
  KeyboardEvent,
  "key" | "ctrlKey" | "shiftKey" | "altKey" | "metaKey"
>;

const FIXED_SHORTCUTS: Array<[string, string]> = [
  ["Ctrl+Tab", "切换到下一个标签页"],
  ["Ctrl+Shift+Tab", "切换到上一个标签页"],
  ["Ctrl+T", "新建标签页"],
  ["Ctrl+Shift+T", "恢复关闭的标签页"],
  ["Ctrl+L", "聚焦地址栏"],
  ["Ctrl+W", "关闭当前标签页"],
  ["Ctrl+H", "打开历史记录"],
  ["Ctrl+J", "打开下载"],
  ["Ctrl+D", "收藏当前页面"],
  ["Ctrl+F", "页面内查找"],
  ["Ctrl+=", "放大页面"],
  ["Ctrl+-", "缩小页面"],
  ["Ctrl+0", "重置缩放"],
];

function normalizedShortcut(shortcut: string): string | null {
  const parts = shortcut
    .trim()
    .toLowerCase()
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean);
  if (parts.length < 2) return null;
  const modifiers = new Set<string>();
  let key: string | null = null;
  for (const part of parts) {
    const modifier =
      part === "control" ? "ctrl" : part === "meta" ? "super" : part;
    if (["ctrl", "alt", "shift", "super"].includes(modifier)) {
      if (modifiers.has(modifier)) return null;
      modifiers.add(modifier);
    } else if (key) {
      return null;
    } else {
      key = part;
    }
  }
  if (!key || !modifiers.size) return null;
  return ["ctrl", "alt", "shift", "super"]
    .filter((modifier) => modifiers.has(modifier))
    .concat(key)
    .join("+");
}

function eventShortcut(event: ShortcutKeyEvent): string | null {
  if (!(event.ctrlKey || event.altKey || event.shiftKey || event.metaKey))
    return null;
  const key =
    event.key.length === 1 ? event.key.toLowerCase() : event.key.toLowerCase();
  return normalizedShortcut(
    [
      event.ctrlKey ? "Ctrl" : "",
      event.altKey ? "Alt" : "",
      event.shiftKey ? "Shift" : "",
      event.metaKey ? "Super" : "",
      key,
    ]
      .filter(Boolean)
      .join("+"),
  );
}

export function findShortcutConflict(bindings: {
  showHide: string;
  tabSearch: string;
  recentlyClosed: string;
}): string | null {
  const configured: Array<[string, string]> = [
    [bindings.showHide, "显示 / 隐藏 QuickPane"],
    [bindings.tabSearch, "搜索标签页"],
    [bindings.recentlyClosed, "最近关闭的标签页"],
  ];
  const seen = new Map<string, string>();
  for (const [shortcut, label] of configured) {
    if (!shortcut.trim()) continue;
    const normalized = normalizedShortcut(shortcut);
    if (!normalized) return `${label}的快捷键格式无效`;
    const previous = seen.get(normalized);
    if (previous) return `${label}与${previous}使用了相同快捷键`;
    seen.set(normalized, label);
    const fixed = FIXED_SHORTCUTS.find(
      ([value]) => normalizedShortcut(value) === normalized,
    );
    if (fixed) return `${label}与${fixed[1]}使用了相同快捷键`;
  }
  return null;
}

export function browserShortcutFromKey(
  event: ShortcutKeyEvent,
  configured: ConfiguredBrowserShortcuts = {
    tabSearch: "Ctrl+Shift+A",
    recentlyClosed: "Ctrl+Shift+Y",
  },
): BrowserShortcut | null {
  const normalized = eventShortcut(event);
  if (normalized === normalizedShortcut(configured.tabSearch))
    return "tab-search";
  if (normalized === normalizedShortcut(configured.recentlyClosed))
    return "recently-closed";

  const key = event.key.toLowerCase();
  if (key === "escape") return "escape";
  if (!event.ctrlKey) return null;
  if (key === "tab") return event.shiftKey ? "previous-tab" : "next-tab";
  if (key === "t") return event.shiftKey ? "restore-tab" : "new-tab";
  if (key === "l") return "focus-address";
  if (key === "w") return "close-tab";
  if (key === "h") return "history";
  if (key === "j") return "downloads";
  if (key === "d") return "bookmark";
  if (key === "f") return "find";
  if (["+", "="].includes(event.key)) return "zoom-in";
  if (event.key === "-") return "zoom-out";
  if (event.key === "0") return "zoom-reset";
  return null;
}
