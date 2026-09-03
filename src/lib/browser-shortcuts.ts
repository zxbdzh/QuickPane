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
  | "zoom-reset";

type ShortcutKeyEvent = Pick<KeyboardEvent, "key" | "ctrlKey" | "shiftKey">;

export function browserShortcutFromKey(
  event: ShortcutKeyEvent,
): BrowserShortcut | null {
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
