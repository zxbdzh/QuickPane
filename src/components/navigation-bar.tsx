import {
  ArrowLeft,
  ArrowRight,
  Home,
  LoaderCircle,
  Menu,
  PanelTopClose,
  Puzzle,
  RefreshCw,
  ShieldCheck,
  Star,
} from "lucide-react";
import type { FormEvent, RefObject } from "react";

import type { BrowserExtension, TabRecord } from "../types";
import { cn } from "../lib/utils";
import { Button } from "./ui/button";
import { IconButton } from "./icon-button";

/** 导航栏：高 54px（与标签栏合计 86px，对应 Rust 侧 CHROME_HEIGHT） */
function NavigationBar({ activeTab, address, onAddress, onSubmit, addressRef, bookmarked, onBookmark, onBack, onForward, onReload, onHome, onHide, onOpenMenu, pinnedExtensions, onExtensionClick }: {
  activeTab: TabRecord | null;
  address: string;
  onAddress: (value: string) => void;
  onSubmit: (event: FormEvent) => void;
  addressRef: RefObject<HTMLInputElement | null>;
  bookmarked: boolean;
  onBookmark: () => void;
  onBack: () => void;
  onForward: () => void;
  onReload: () => void;
  onHome: () => void;
  onHide: () => void;
  onOpenMenu: (anchor: { x: number; y: number }) => void;
  pinnedExtensions: BrowserExtension[];
  onExtensionClick: (extension: BrowserExtension, anchor: { x: number; y: number }) => void;
}) {
  const secure = address.startsWith("https://");

  return (
    <nav className="flex h-[54px] shrink-0 items-center gap-1.5 border-b border-border bg-surface px-2">
      <div className="flex select-none items-center gap-0.5">
        <IconButton label="后退" shortcut="Alt+←" onClick={onBack}><ArrowLeft className="size-4" /></IconButton>
        <IconButton label="前进" shortcut="Alt+→" onClick={onForward}><ArrowRight className="size-4" /></IconButton>
        <IconButton label="刷新" shortcut="Ctrl+R" onClick={onReload}><RefreshCw className="size-4" /></IconButton>
        <IconButton label="主页" onClick={onHome}><Home className="size-4" /></IconButton>
      </div>

      <form
        className="mx-auto flex h-9 w-full min-w-0 max-w-[560px] items-center gap-2 rounded-md border border-input bg-background px-2.5 transition-[border-color,box-shadow] focus-within:border-ring focus-within:shadow-glow-strong"
        onSubmit={onSubmit}
      >
        <ShieldCheck className={cn("size-4 shrink-0", secure ? "text-primary" : "text-faint")} />
        <input
          ref={addressRef}
          value={address}
          onChange={(event) => onAddress(event.target.value)}
          placeholder="输入网址或搜索内容"
          spellCheck={false}
          aria-label="地址与搜索"
          className="h-full min-w-0 flex-1 bg-transparent text-sm text-foreground outline-none placeholder:text-faint"
        />
        {activeTab?.loading ? <LoaderCircle className="size-4 shrink-0 animate-spin text-muted-foreground" /> : null}
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
        <Button
          variant="ghost"
          size="icon"
          aria-label="QuickPane 菜单"
          className="text-muted-foreground hover:text-foreground"
          onClick={(event) => {
            const rect = event.currentTarget.getBoundingClientRect();
            onOpenMenu({ x: rect.left, y: rect.bottom + 6 });
          }}
        >
          <Menu className="size-[18px]" />
        </Button>
      </div>
    </nav>
  );
}

export { NavigationBar };
