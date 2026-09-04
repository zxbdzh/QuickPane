import {
  ArrowLeft,
  ArrowRight,
  Bookmark,
  Clock3,
  Download,
  Globe2,
  History,
  Home,
  LoaderCircle,
  Lock,
  Menu,
  PanelTopClose,
  Puzzle,
  RefreshCw,
  Settings as SettingsIcon,
  ShieldCheck,
  Star,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useState } from "react";
import type { FormEvent, KeyboardEvent, ReactNode, RefObject } from "react";

import type { AddressSuggestion } from "../lib/address-suggestions";
import type { ShellSection, BrowserExtension, TabRecord } from "../types";
import { cn } from "../lib/utils";
import { overlay } from "../lib/motion";
import { Button } from "./ui/button";
import { IconButton } from "./icon-button";
import { Kbd } from "./ui/kbd";

/** 导航栏：高 54px（与标签栏合计 86px，对应 Rust 侧 CHROME_HEIGHT） */
function NavigationBar({ activeTab, address, onAddress, onSubmit, addressRef, suggestions, onSuggestion, onOverlayOpenChange, windowVisible, bookmarked, onBookmark, onBack, onForward, onReload, onHome, onHide, hasPassword, onOpenSection, onLockNow, pinnedExtensions, onExtensionClick }: {
  activeTab: TabRecord | null;
  address: string;
  onAddress: (value: string) => void;
  onSubmit: (event: FormEvent) => void;
  addressRef: RefObject<HTMLInputElement | null>;
  suggestions: AddressSuggestion[];
  onSuggestion: (suggestion: AddressSuggestion) => void;
  /** 地址下拉或菜单任一浮层开合时上报：驱动 main WebView 扩幅/收缩。 */
  onOverlayOpenChange: (open: boolean) => void;
  windowVisible: boolean;
  bookmarked: boolean;
  onBookmark: () => void;
  onBack: () => void;
  onForward: () => void;
  onReload: () => void;
  onHome: () => void;
  onHide: () => void;
  hasPassword: boolean;
  onOpenSection: (section: ShellSection) => void;
  onLockNow: () => void;
  pinnedExtensions: BrowserExtension[];
  onExtensionClick: (extension: BrowserExtension, anchor: { x: number; y: number }) => void;
}) {
  const [suggestionsRequested, setSuggestionsRequested] = useState(false);
  const [selectedSuggestion, setSelectedSuggestion] = useState(-1);
  const [menuOpen, setMenuOpen] = useState(false);
  const suggestionsOpen = suggestionsRequested && suggestions.length > 0;
  const secure = address.startsWith("https://");

  useEffect(() => {
    setSuggestionsRequested(false);
    setSelectedSuggestion(-1);
    setMenuOpen(false);
  }, [activeTab?.id, activeTab?.url]);

  useEffect(() => {
    if (!windowVisible) {
      setSuggestionsRequested(false);
      setSelectedSuggestion(-1);
      setMenuOpen(false);
    }
  }, [windowVisible]);

  // 任一浮层（地址下拉/菜单）开合都上报：true → main WebView 扩幅盖网页；false → 收缩还网页。
  const overlayOpen = suggestionsOpen || menuOpen;
  useEffect(() => {
    onOverlayOpenChange(overlayOpen);
  }, [onOverlayOpenChange, overlayOpen]);

  // 菜单下拉：点击外部即收起（下拉本体在 UI 层内，天然位于网页之上）。
  useEffect(() => {
    if (!menuOpen) return;
    const onPointerDown = (event: PointerEvent) => {
      if (event.target instanceof Element && !event.target.closest("[data-qp-menu]")) {
        setMenuOpen(false);
      }
    };
    document.addEventListener("pointerdown", onPointerDown);
    return () => document.removeEventListener("pointerdown", onPointerDown);
  }, [menuOpen]);

  const menuItems: Array<{ key: string; icon: ReactNode; label: string; hint?: string; action: () => void }> = [
    { key: "history", icon: <History className="size-4" />, label: "历史记录", hint: "Ctrl+H", action: () => onOpenSection("history") },
    { key: "bookmarks", icon: <Bookmark className="size-4" />, label: "书签", action: () => onOpenSection("bookmarks") },
    { key: "downloads", icon: <Download className="size-4" />, label: "下载", hint: "Ctrl+J", action: () => onOpenSection("downloads") },
    ...(hasPassword ? [{ key: "lock", icon: <Lock className="size-4" />, label: "立即锁定", action: onLockNow }] : []),
    { key: "extensions", icon: <Puzzle className="size-4" />, label: "扩展", action: () => onOpenSection("extensions") },
    { key: "settings", icon: <SettingsIcon className="size-4" />, label: "设置", action: () => onOpenSection("settings") },
  ];

  const closeSuggestions = () => {
    setSuggestionsRequested(false);
    setSelectedSuggestion(-1);
  };

  const chooseSuggestion = (suggestion: AddressSuggestion) => {
    closeSuggestions();
    onSuggestion(suggestion);
  };

  const handleAddressKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Escape" && suggestionsOpen) {
      event.preventDefault();
      event.stopPropagation();
      closeSuggestions();
      return;
    }
    if ((event.key === "ArrowDown" || event.key === "ArrowUp") && suggestions.length > 0) {
      event.preventDefault();
      setSuggestionsRequested(true);
      setSelectedSuggestion((current) => {
        if (event.key === "ArrowDown") return current < suggestions.length - 1 ? current + 1 : 0;
        return current > 0 ? current - 1 : suggestions.length - 1;
      });
      return;
    }
    if (event.key === "Enter" && suggestionsOpen && selectedSuggestion >= 0) {
      event.preventDefault();
      chooseSuggestion(suggestions[selectedSuggestion]);
    }
  };

  return (
    <nav className="flex h-[54px] shrink-0 items-center gap-1.5 border-b border-border bg-surface px-2">
      <div className="flex select-none items-center gap-0.5">
        <IconButton label="后退" shortcut="Alt+←" onClick={onBack}><ArrowLeft className="size-4" /></IconButton>
        <IconButton label="前进" shortcut="Alt+→" onClick={onForward}><ArrowRight className="size-4" /></IconButton>
        <IconButton label="刷新" shortcut="Ctrl+R" onClick={onReload}><RefreshCw className="size-4" /></IconButton>
        <IconButton label="主页" onClick={onHome}><Home className="size-4" /></IconButton>
      </div>

      <form
        className="relative z-40 mx-auto flex h-9 w-full min-w-0 max-w-[560px] items-center gap-2 rounded-md border border-input bg-background px-2.5 transition-[border-color,box-shadow] focus-within:border-ring focus-within:shadow-glow-strong"
        onKeyDown={(event) => {
          if (event.key === "Escape" && suggestionsOpen) {
            event.preventDefault();
            event.stopPropagation();
            closeSuggestions();
          }
        }}
        onSubmit={(event) => {
          closeSuggestions();
          onSubmit(event);
        }}
        onBlur={(event) => {
          if (!event.currentTarget.contains(event.relatedTarget)) closeSuggestions();
        }}
      >
        <ShieldCheck className={cn("size-4 shrink-0", secure ? "text-primary" : "text-faint")} />
        <input
          ref={addressRef}
          value={address}
          onChange={(event) => {
            onAddress(event.target.value);
            setSuggestionsRequested(true);
            setSelectedSuggestion(-1);
          }}
          onKeyDown={handleAddressKeyDown}
          placeholder="输入网址或搜索内容"
          spellCheck={false}
          autoComplete="off"
          role="combobox"
          aria-label="地址与搜索"
          aria-haspopup="listbox"
          aria-autocomplete="list"
          aria-controls="address-suggestions"
          aria-expanded={suggestionsOpen}
          aria-activedescendant={selectedSuggestion >= 0 ? `address-suggestion-${selectedSuggestion}` : undefined}
          className="h-full min-w-0 flex-1 bg-transparent font-mono text-sm text-foreground outline-none placeholder:font-sans placeholder:text-faint"
        />
        {activeTab?.loading ? <LoaderCircle className="size-4 shrink-0 animate-spin text-accent2" /> : null}
        <button
          type="button"
          onClick={onBookmark}
          aria-label="收藏此页"
          title="收藏此页 (Ctrl+D)"
          className={cn(
            "grid size-6 shrink-0 place-items-center rounded-sm transition-colors hover:bg-muted focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none",
            bookmarked ? "text-primary" : "text-muted-foreground",
          )}
        >
          <Star className="size-4" fill={bookmarked ? "currentColor" : "none"} />
        </button>

        <AnimatePresence initial={false}>
          {suggestionsOpen ? (
            <motion.div
              id="address-suggestions"
              role="listbox"
              aria-label="本地浏览建议"
              variants={overlay}
              initial="initial"
              animate="animate"
              exit="exit"
              className="absolute top-[calc(100%+6px)] right-0 left-0 max-h-[360px] overflow-y-auto rounded-md border bg-popover p-1 shadow-popover"
            >
              {suggestions.map((suggestion, index) => (
                <button
                  id={`address-suggestion-${index}`}
                  key={`${suggestion.source}:${suggestion.url}`}
                  type="button"
                  role="option"
                  tabIndex={-1}
                  aria-selected={index === selectedSuggestion}
                  onMouseDown={(event) => event.preventDefault()}
                  onMouseEnter={() => setSelectedSuggestion(index)}
                  onClick={() => chooseSuggestion(suggestion)}
                  className={cn(
                    "grid h-11 w-full grid-cols-[20px_minmax(0,1fr)_auto] items-center gap-2 rounded-sm px-2 text-left outline-none",
                    index === selectedSuggestion && "bg-soft text-on-soft",
                  )}
                >
                  {suggestion.source === "tab" ? (
                    <Globe2 className="size-4 text-muted-foreground" />
                  ) : suggestion.source === "history" ? (
                    <Clock3 className="size-4 text-muted-foreground" />
                  ) : suggestion.source === "bookmark" ? (
                    <Star className="size-4 text-muted-foreground" />
                  ) : (
                    <Home className="size-4 text-muted-foreground" />
                  )}
                  <span className="min-w-0">
                    <strong className="block truncate text-sm font-medium">{suggestion.title}</strong>
                    <small className="block truncate font-mono text-xs text-muted-foreground">{suggestion.url}</small>
                  </span>
                  <span className="max-w-28 truncate font-mono text-xs text-faint">{suggestion.host}</span>
                </button>
              ))}
            </motion.div>
          ) : null}
        </AnimatePresence>
      </form>

      <div className="flex select-none items-center gap-0.5">
        {pinnedExtensions.map((extension) => (
          <button
            type="button"
            key={extension.id}
            title={extension.name}
            aria-label={`打开 ${extension.name} 面板`}
            onClick={(event) => {
              const rect = event.currentTarget.getBoundingClientRect();
              onExtensionClick(extension, { x: rect.left, y: rect.bottom + 4 });
            }}
            className="grid size-8 shrink-0 place-items-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
          >
            {extension.icon ? (
              <img src={extension.icon} alt="" className="size-[18px]" />
            ) : (
              <Puzzle className="size-[18px]" />
            )}
          </button>
        ))}
        <IconButton label="隐藏窗口" onClick={onHide}><PanelTopClose className="size-4" /></IconButton>
        <div
          data-qp-menu
          className="relative"
          onKeyDown={(event) => {
            if (event.key === "Escape" && menuOpen) {
              event.preventDefault();
              event.stopPropagation();
              setMenuOpen(false);
            }
          }}
        >
          <Button
            variant="ghost"
            size="icon"
            aria-label="QuickPane 菜单"
            aria-haspopup="menu"
            aria-expanded={menuOpen}
            className={cn(
              "text-muted-foreground hover:text-foreground",
              menuOpen && "bg-muted text-foreground",
            )}
            onClick={() => setMenuOpen((open) => !open)}
          >
            <Menu className="size-[18px]" />
          </Button>
          <AnimatePresence initial={false}>
            {menuOpen ? (
              <motion.div
                role="menu"
                aria-label="QuickPane 菜单"
                variants={overlay}
                initial="initial"
                animate="animate"
                exit="exit"
                className="absolute top-[calc(100%+6px)] right-0 z-40 w-56 overflow-hidden rounded-lg border border-border bg-popover p-1.5 text-popover-foreground shadow-popover"
              >
                {menuItems.map((item) => (
                  <button
                    type="button"
                    role="menuitem"
                    key={item.key}
                    onClick={() => {
                      setMenuOpen(false);
                      item.action();
                    }}
                    className="flex h-9 w-full items-center gap-2.5 rounded-md px-2.5 text-sm outline-none transition-colors hover:bg-muted focus-visible:bg-muted"
                  >
                    {item.icon}
                    {item.label}
                    {item.hint ? <Kbd className="ml-auto">{item.hint}</Kbd> : null}
                  </button>
                ))}
              </motion.div>
            ) : null}
          </AnimatePresence>
        </div>
      </div>
    </nav>
  );
}

export { NavigationBar };
