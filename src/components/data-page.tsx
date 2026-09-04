import {
    Check,
    CircleAlert,
    Copy,
    Download,
    ExternalLink,
    History,
    Search,
    LoaderCircle,
    Star,
    Trash2,
} from "lucide-react";
import type { ReactNode } from "react";
import { useState } from "react";

import type { AppSnapshot } from "../types";
import { cn } from "../lib/utils";
import { matchesTextQuery } from "../lib/text-search";
import { formatDate, hostOf } from "../lib/format";
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogTitle,
    AlertDialogTrigger,
} from "./ui/alert-dialog";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuTrigger,
} from "./ui/context-menu";

/** 页面容器 + 标题 */
function PageShell({
    title,
    count,
    clearLabel,
    onClear,
    children,
}: {
    title: string;
    count: number;
    clearLabel?: string;
    onClear?: () => void;
    children: ReactNode;
}) {
    return (
        <div className="mx-auto w-full max-w-[880px] px-6 pt-7 pb-16">
            <header className="flex items-end justify-between gap-4 pb-4">
                <div>
                    <h1 className="text-xl font-semibold">{title}</h1>
                    <p className="mt-0.5 text-xs text-muted-foreground">
                        <span className="font-mono">{count}</span> 项
                    </p>
                </div>
                {onClear && count > 0 ? (
                    <AlertDialog>
                        <AlertDialogTrigger asChild>
                            <Button
                                variant="ghost"
                                size="sm"
                                className="text-destructive hover:bg-destructive-soft hover:text-destructive"
                            >
                                <Trash2 className="size-4" />
                                {clearLabel ?? "清除记录"}
                            </Button>
                        </AlertDialogTrigger>
                        <AlertDialogContent>
                            <AlertDialogTitle>
                                清除全部{title}？
                            </AlertDialogTitle>
                            <AlertDialogDescription>
                                将删除 {count} 项记录，此操作不可撤销。
                            </AlertDialogDescription>
                            <AlertDialogFooter>
                                <AlertDialogCancel>取消</AlertDialogCancel>
                                <AlertDialogAction
                                    variant="destructive"
                                    onClick={onClear}
                                >
                                    清除
                                </AlertDialogAction>
                            </AlertDialogFooter>
                        </AlertDialogContent>
                    </AlertDialog>
                ) : null}
            </header>
            {count > 0 ? (
                <div className="overflow-hidden rounded-lg border bg-surface">
                    {children}
                </div>
            ) : (
                children
            )}
        </div>
    );
}

/** 空状态：图标 + 主文案 + 引导文案 */
function EmptyState({
    icon,
    title,
    hint,
}: {
    icon: ReactNode;
    title: string;
    hint: string;
}) {
    return (
        <div className="flex flex-col items-center justify-center gap-3 rounded-lg border border-dashed py-20 text-center">
            <span className="grid size-12 place-items-center rounded-md bg-soft text-on-soft [&_svg]:size-5">
                {icon}
            </span>
            <div>
                <p className="text-sm font-medium">{title}</p>
                <p className="mt-1 text-xs text-muted-foreground">{hint}</p>
            </div>
        </div>
    );
}

/** 数据行：整行 hover、右键菜单、行尾操作 */
function DataRow({
    icon,
    title,
    subtitle,
    meta,
    url,
    onOpen,
    onRemove,
    removeLabel,
}: {
    icon: ReactNode;
    title: string;
    subtitle: string;
    meta: string;
    url?: string;
    onOpen?: () => void;
    onRemove?: () => void;
    removeLabel?: string;
}) {
    const copyLink = () => {
        if (url)
            void navigator.clipboard?.writeText(url).catch(() => undefined);
    };

    const row = (
        <div
            onClick={onOpen}
            onKeyDown={(event) => {
                if (event.target !== event.currentTarget) return;
                if (onOpen && (event.key === "Enter" || event.key === " ")) {
                    event.preventDefault();
                    onOpen();
                }
            }}
            tabIndex={onOpen ? 0 : undefined}
            role={onOpen ? "button" : undefined}
            className={cn(
                "group grid min-h-[56px] grid-cols-[34px_minmax(0,1fr)_auto_auto] items-center gap-3 border-b border-border/70 px-3 py-2 transition-colors last:border-b-0",
                "outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:ring-inset",
                onOpen && "cursor-default hover:bg-muted/70",
            )}
        >
            <span className="grid size-[30px] place-items-center rounded-md bg-muted text-muted-foreground [&_svg]:size-4">
                {icon}
            </span>
            <div className="min-w-0">
                <p className="truncate text-sm font-medium">{title}</p>
                <p className="truncate text-xs text-muted-foreground">
                    {subtitle}
                </p>
            </div>
            <time className="font-mono text-xs text-muted-foreground">{meta}</time>
            <div className="flex items-center gap-0.5">
                {onRemove ? (
                    <Button
                        variant="ghost"
                        size="iconSm"
                        aria-label={removeLabel ?? "删除"}
                        onClick={(event) => {
                            event.stopPropagation();
                            onRemove();
                        }}
                        className="text-muted-foreground hover:bg-destructive-soft hover:text-destructive"
                    >
                        <Trash2 className="size-3.5" />
                    </Button>
                ) : null}
                {onOpen ? (
                    <ExternalLink className="size-4 text-faint opacity-0 transition-opacity group-hover:opacity-100" />
                ) : null}
            </div>
        </div>
    );

    if (!url) return row;

    return (
        <ContextMenu>
            <ContextMenuTrigger asChild>{row}</ContextMenuTrigger>
            <ContextMenuContent>
                {onOpen ? (
                    <ContextMenuItem onSelect={() => onOpen()}>
                        <ExternalLink />
                        打开链接
                    </ContextMenuItem>
                ) : null}
                <ContextMenuItem onSelect={copyLink}>
                    <Copy />
                    复制链接
                </ContextMenuItem>
                {onRemove ? (
                    <>
                        <ContextMenuSeparator />
                        <ContextMenuItem onSelect={() => onRemove()}>
                            <Trash2 />
                            {removeLabel ?? "删除"}
                        </ContextMenuItem>
                    </>
                ) : null}
            </ContextMenuContent>
        </ContextMenu>
    );
}

function PageSearch({
    value,
    onChange,
    placeholder,
}: {
    value: string;
    onChange: (value: string) => void;
    placeholder: string;
}) {
    return (
        <label className="relative block pb-3">
            <Search className="pointer-events-none absolute top-1/2 left-3 size-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input
                value={value}
                onChange={(event) => onChange(event.target.value)}
                placeholder={placeholder}
                aria-label={placeholder}
                className="h-8 pl-8 text-xs"
            />
        </label>
    );
}

function HistoryPage({
    snapshot,
    onOpen,
    onClear,
}: {
    snapshot: AppSnapshot;
    onOpen: (url: string) => void;
    onClear: () => void;
}) {
    const [query, setQuery] = useState("");
    const history = snapshot.data.history.filter((item) =>
        matchesTextQuery(query, item.title, item.url),
    );

    return (
        <PageShell
            title="历史记录"
            count={snapshot.data.history.length}
            onClear={onClear}
        >
            <PageSearch
                value={query}
                onChange={setQuery}
                placeholder="搜索历史记录"
            />
            {history.length === 0 ? (
                <EmptyState
                    icon={<History />}
                    title="还没有访问记录"
                    hint="通过地址栏浏览的网页会出现在这里，点击即可重新打开。"
                />
            ) : (
                history.map((item) => (
                    <DataRow
                        key={item.id}
                        icon={<History />}
                        title={item.title || hostOf(item.url)}
                        subtitle={item.url}
                        url={item.url}
                        meta={formatDate(item.visitedAt)}
                        onOpen={() => onOpen(item.url)}
                    />
                ))
            )}
        </PageShell>
    );
}

function BookmarksPage({
    snapshot,
    onOpen,
    onRemove,
}: {
    snapshot: AppSnapshot;
    onOpen: (url: string) => void;
    onRemove: (id: string) => void;
}) {
    const [query, setQuery] = useState("");
    const bookmarks = snapshot.data.bookmarks.filter((item) =>
        matchesTextQuery(query, item.title, item.url),
    );

    return (
        <PageShell title="书签" count={snapshot.data.bookmarks.length}>
            <PageSearch
                value={query}
                onChange={setQuery}
                placeholder="搜索书签"
            />
            {bookmarks.length === 0 ? (
                <EmptyState
                    icon={<Star />}
                    title="收藏的网页会显示在这里"
                    hint="浏览时点击地址栏右侧的星标即可收藏当前页面。"
                />
            ) : (
                bookmarks.map((item) => (
                    <DataRow
                        key={item.id}
                        icon={<Star />}
                        title={item.title || hostOf(item.url)}
                        subtitle={item.url}
                        url={item.url}
                        meta={formatDate(item.createdAt)}
                        onOpen={() => onOpen(item.url)}
                        onRemove={() => onRemove(item.id)}
                        removeLabel="删除书签"
                    />
                ))
            )}
        </PageShell>
    );
}

function DownloadsPage({
    snapshot,
    onOpen,
    onClear,
}: {
    snapshot: AppSnapshot;
    onOpen: (path: string) => void;
    onClear: () => void;
}) {
    return (
        <PageShell
            title="下载"
            count={snapshot.data.downloads.length}
            onClear={onClear}
        >
            {snapshot.data.downloads.length === 0 ? (
                <EmptyState
                    icon={<Download />}
                    title="下载记录会显示在这里"
                    hint="完成的下载可以直接从列表中打开。"
                />
            ) : (
                snapshot.data.downloads.map((item) => (
                    <DataRow
                        key={item.id}
                        icon={
                            item.state === "downloading" ? (
                                <LoaderCircle className="animate-spin" />
                            ) : item.state === "failed" ? (
                                <CircleAlert className="text-destructive" />
                            ) : (
                                <Check className="text-success" />
                            )
                        }
                        title={item.fileName}
                        subtitle={item.path ?? item.url}
                        meta={
                            item.state === "completed"
                                ? "已完成"
                                : item.state === "failed"
                                  ? "失败"
                                  : "下载中"
                        }
                        onOpen={
                            item.path ? () => onOpen(item.path!) : undefined
                        }
                    />
                ))
            )}
        </PageShell>
    );
}

export { HistoryPage, BookmarksPage, DownloadsPage };
