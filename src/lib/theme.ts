import { useCallback, useEffect, useState } from "react";

/**
 * 明暗主题管理（纯前端状态，存 localStorage；class 策略挂在 <html> 上）。
 * 跟随系统时监听 prefers-color-scheme 变化。
 */

export type ThemePreference = "system" | "light" | "dark";

const STORAGE_KEY = "quickpane.theme";
const QUERY = "(prefers-color-scheme: dark)";

function systemPrefersDark() {
  return typeof window !== "undefined" && window.matchMedia(QUERY).matches;
}

export function readThemePreference(): ThemePreference {
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    return value === "light" || value === "dark" ? value : "system";
  } catch {
    return "system";
  }
}

export function applyTheme(preference: ThemePreference) {
  const dark = preference === "dark" || (preference === "system" && systemPrefersDark());
  document.documentElement.classList.toggle("dark", dark);
}

export function writeThemePreference(preference: ThemePreference) {
  try {
    localStorage.setItem(STORAGE_KEY, preference);
  } catch {
    /* 忽略持久化失败，主题仍在当前会话生效 */
  }
  applyTheme(preference);
}

/** 运行期同步主题：偏好变化、系统明暗变化都实时生效 */
export function useThemePreference(): [ThemePreference, (value: ThemePreference) => void] {
  const [preference, setPreference] = useState<ThemePreference>(readThemePreference);

  useEffect(() => {
    applyTheme(preference);
    if (preference !== "system") return;
    const media = window.matchMedia(QUERY);
    const onChange = () => applyTheme("system");
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, [preference]);

  const update = useCallback((value: ThemePreference) => {
    writeThemePreference(value);
    setPreference(value);
  }, []);

  return [preference, update];
}
