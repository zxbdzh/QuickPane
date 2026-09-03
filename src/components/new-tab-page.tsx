import {
  Check,
  Clock3,
  FileDown,
  Pencil,
  Plus,
  Search,
  Star,
  Trash2,
  X,
} from "lucide-react";
import { useState } from "react";
import { motion } from "motion/react";

import type { AppSnapshot, QuickLink, ShellSection } from "../types";
import { hostOf } from "../lib/format";
import { listItem } from "../lib/motion";
import { Kbd } from "./ui/kbd";

function NewTabPage({
  snapshot,
  onNavigate,
  onSection,
  onUpdateQuickLinks,
}: {
  snapshot: AppSnapshot;
  onNavigate: (url: string) => void;
  onSection: (section: ShellSection) => void;
  onUpdateQuickLinks: (quickLinks: QuickLink[]) => void;
}) {
  const [query, setQuery] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draft, setDraft] = useState({ title: "", url: "" });

  const beginEdit = (link: QuickLink) => {
    setEditingId(link.id);
    setDraft({ title: link.title, url: link.url });
  };

  const saveEdit = () => {
    const title = draft.title.trim();
    const url = draft.url.trim();
    if (!editingId || !title || !url) return;
    onUpdateQuickLinks(
      snapshot.data.settings.quickLinks.map((link) =>
        link.id === editingId ? { ...link, title, url } : link,
      ),
    );
    setEditingId(null);
  };

  const removeLink = (id: string) => {
    onUpdateQuickLinks(
      snapshot.data.settings.quickLinks.filter((link) => link.id !== id),
    );
    if (editingId === id) setEditingId(null);
  };

  return (
    <div className="mx-auto w-full max-w-[760px] px-6 pt-[clamp(48px,10vh,104px)] pb-16">
      <header className="mb-7 flex items-center justify-center gap-3.5">
        <img
          src="/quickpane-mark.svg"
          alt=""
          aria-hidden="true"
          className="size-11 shrink-0"
        />
        <div>
          <h1 className="text-xl font-semibold leading-tight">QuickPane</h1>
          <p className="text-xs text-muted-foreground">你的轻量浏览空间</p>
        </div>
      </header>

      <form
        className="mx-auto flex h-12 w-full items-center gap-2.5 rounded-lg border border-input bg-surface px-4 transition-[border-color,box-shadow] focus-within:border-ring focus-within:shadow-glow-strong"
        onSubmit={(event) => {
          event.preventDefault();
          if (query.trim()) onNavigate(query);
        }}
      >
        <Search className="size-[18px] shrink-0 text-muted-foreground" />
        <input
          autoFocus
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="搜索或输入网址"
          aria-label="搜索或输入网址"
          spellCheck={false}
          className="h-full min-w-0 flex-1 bg-transparent text-base text-foreground outline-none placeholder:text-faint"
        />
        <Kbd>Enter</Kbd>
      </form>

      <div className="mt-7 grid grid-cols-2 gap-2 sm:grid-cols-4">
        {snapshot.data.settings.quickLinks.map((link, index) =>
          editingId === link.id ? (
            <motion.form
              key={link.id}
              variants={listItem}
              custom={index}
              initial="initial"
              animate="animate"
              onSubmit={(event) => {
                event.preventDefault();
                saveEdit();
              }}
              className="flex h-20 min-w-0 flex-col justify-center gap-1 rounded-md border border-ring bg-surface px-3 focus-within:shadow-glow"
            >
              <input
                autoFocus
                value={draft.title}
                onChange={(event) =>
                  setDraft({ ...draft, title: event.target.value })
                }
                aria-label="快捷站点名称"
                className="h-6 w-full min-w-0 bg-transparent text-xs font-semibold outline-none"
              />
              <input
                value={draft.url}
                onChange={(event) =>
                  setDraft({ ...draft, url: event.target.value })
                }
                aria-label="快捷站点网址"
                className="h-6 w-full min-w-0 bg-transparent text-[11px] text-muted-foreground outline-none"
              />
              <span className="flex justify-end gap-0.5">
                <button
                  type="button"
                  aria-label="取消编辑"
                  onClick={() => setEditingId(null)}
                  className="grid size-5 place-items-center rounded-sm text-muted-foreground hover:bg-muted"
                >
                  <X className="size-3" />
                </button>
                <button
                  type="submit"
                  aria-label="保存快捷站点"
                  disabled={!draft.title.trim() || !draft.url.trim()}
                  className="grid size-5 place-items-center rounded-sm text-primary hover:bg-soft disabled:opacity-40"
                >
                  <Check className="size-3" />
                </button>
              </span>
            </motion.form>
          ) : (
            <motion.div
              key={link.id}
              variants={listItem}
              custom={index}
              initial="initial"
              animate="animate"
              className="group relative flex h-20 min-w-0 items-center gap-2.5 rounded-md border bg-surface px-3 transition-colors hover:border-gray-300 hover:bg-muted/60 dark:hover:border-gray-700"
            >
              <button
                type="button"
                onClick={() => onNavigate(link.url)}
                aria-label={`打开 ${link.title}`}
                className="flex min-w-0 flex-1 items-center gap-2.5 text-left outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
              >
                <span className="grid size-9 shrink-0 place-items-center rounded-md bg-soft text-sm font-bold text-on-soft">
                  {link.title.slice(0, 1).toUpperCase()}
                </span>
                <span className="min-w-0">
                  <strong className="block truncate text-sm font-semibold">
                    {link.title}
                  </strong>
                  <small className="block truncate text-xs text-muted-foreground">
                    {hostOf(link.url)}
                  </small>
                </span>
              </button>
              <span className="absolute right-1 top-1 flex gap-0.5 opacity-0 transition-opacity group-hover:opacity-100 group-focus-within:opacity-100">
                <button
                  type="button"
                  aria-label={`编辑 ${link.title}`}
                  title="编辑快捷站点"
                  onClick={() => beginEdit(link)}
                  className="grid size-5 place-items-center rounded-sm bg-surface text-muted-foreground hover:bg-muted hover:text-foreground"
                >
                  <Pencil className="size-3" />
                </button>
                <button
                  type="button"
                  aria-label={`删除 ${link.title}`}
                  title="删除快捷站点"
                  onClick={() => removeLink(link.id)}
                  className="grid size-5 place-items-center rounded-sm bg-surface text-muted-foreground hover:bg-destructive-soft hover:text-destructive"
                >
                  <Trash2 className="size-3" />
                </button>
              </span>
            </motion.div>
          ),
        )}
        <motion.button
          variants={listItem}
          custom={snapshot.data.settings.quickLinks.length}
          initial="initial"
          animate="animate"
          onClick={() => onSection("settings")}
          className="flex h-20 min-w-0 items-center gap-2.5 rounded-md border border-dashed bg-transparent px-3 text-left outline-none transition-colors hover:bg-muted/60 focus-visible:ring-[3px] focus-visible:ring-ring/50"
        >
          <span className="grid size-9 shrink-0 place-items-center rounded-md bg-muted text-muted-foreground">
            <Plus className="size-4.5" />
          </span>
          <span className="min-w-0">
            <strong className="block truncate text-sm font-semibold">
              添加站点
            </strong>
            <small className="block truncate text-xs text-muted-foreground">
              在设置中管理
            </small>
          </span>
        </motion.button>
      </div>

      <div className="mt-6 grid grid-cols-3 border-t border-border">
        <button
          onClick={() => onSection("history")}
          className="flex h-11 cursor-default items-center justify-center gap-2 text-xs text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50"
        >
          <Clock3 className="size-3.5" />
          最近访问
          <span className="text-faint">{snapshot.data.history.length}</span>
        </button>
        <button
          onClick={() => onSection("bookmarks")}
          className="flex h-11 cursor-default items-center justify-center gap-2 text-xs text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50"
        >
          <Star className="size-3.5" />
          已收藏
          <span className="text-faint">{snapshot.data.bookmarks.length}</span>
        </button>
        <button
          onClick={() => onSection("downloads")}
          className="flex h-11 cursor-default items-center justify-center gap-2 text-xs text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50"
        >
          <FileDown className="size-3.5" />
          下载记录
          <span className="text-faint">{snapshot.data.downloads.length}</span>
        </button>
      </div>
    </div>
  );
}

export { NewTabPage };
