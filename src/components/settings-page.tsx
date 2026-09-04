import { listen } from "@tauri-apps/api/event";
import { Check, Download, Eye, EyeOff, LoaderCircle, Plus, X } from "lucide-react";
import type { KeyboardEvent, ReactNode } from "react";
import { useEffect, useState } from "react";

import { api, type UpdateInfo, type UpdateProgress } from "../api";
import type { AppSnapshot, ProxyMode, QuickLink } from "../types";
import { useThemePreference, type ThemePreference } from "../lib/theme";
import { cn } from "../lib/utils";
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
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./ui/card";
import { Input } from "./ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";
import { Switch } from "./ui/switch";

function SettingsPage({ snapshot, applySnapshot, run }: {
  snapshot: AppSnapshot;
  applySnapshot: (value: AppSnapshot) => void;
  run: <T>(action: () => Promise<T>) => Promise<T | undefined>;
}) {
  const settings = snapshot.data.settings;
  const hasPassword = snapshot.hasPassword;
  const [theme, setTheme] = useThemePreference();
  const [shortcut, setShortcut] = useState(settings.shortcut ?? "");
  const [autostart, setAutostart] = useState(settings.autostart);
  const [homeUrl, setHomeUrl] = useState(settings.homeUrl);
  const [searchTemplate, setSearchTemplate] = useState(settings.searchTemplate);
  const [historyDays, setHistoryDays] = useState(settings.historyDays);
  const [lockOnSystemLock, setLockOnSystemLock] = useState(settings.lockOnSystemLock);
  const [autoLockAfterHideSeconds, setAutoLockAfterHideSeconds] = useState(settings.autoLockAfterHideSeconds ?? 0);
  const [quickLinks, setQuickLinks] = useState<QuickLink[]>(settings.quickLinks);
  const [proxyMode, setProxyMode] = useState<ProxyMode>(settings.proxyMode ?? "system");
  const [proxyUrl, setProxyUrl] = useState(settings.proxyUrl ?? "");
  const [password, setPassword] = useState("");
  const [currentPassword, setCurrentPassword] = useState("");
  const [revealPassword, setRevealPassword] = useState(false);
  const [saveState, setSaveState] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [installingUpdate, setInstallingUpdate] = useState(false);
  const [updateProgress, setUpdateProgress] = useState<UpdateProgress | null>(null);
  const [updateMessage, setUpdateMessage] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    let cleanup: (() => void) | undefined;
    void listen<UpdateProgress>("update-progress", (event) => {
      if (active) setUpdateProgress(event.payload);
    }).then((unlisten) => {
      if (active) cleanup = unlisten;
      else unlisten();
    });
    return () => {
      active = false;
      cleanup?.();
    };
  }, []);

  const checkForUpdate = async () => {
    setCheckingUpdate(true);
    setUpdate(null);
    setUpdateMessage(null);
    setUpdateProgress(null);
    const result = await run(api.checkUpdate);
    if (result === null) setUpdateMessage("当前已是最新版本");
    else if (result) {
      setUpdate(result);
      setUpdateMessage(`发现新版本 ${result.version}`);
    }
    setCheckingUpdate(false);
  };

  const installAvailableUpdate = async () => {
    setInstallingUpdate(true);
    setUpdateMessage(null);
    const result = await run(() => api.installUpdate().then(() => true));
    if (result) setUpdateMessage("更新已下载，应用即将重启");
    setInstallingUpdate(false);
  };

  const save = async () => {
    setSaveState("saving");
    const next = await run(() => api.updateSettings({ autostart, homeUrl, searchTemplate, historyDays, lockOnSystemLock, autoLockAfterHideSeconds, quickLinks, proxyMode, proxyUrl }));
    if (!next) {
      setSaveState("error");
      return;
    }
    applySnapshot(next);
    if (shortcut.trim() && shortcut !== settings.shortcut) {
      const withShortcut = await run(() => api.setShortcut(shortcut.trim()));
      if (!withShortcut) {
        setSaveState("error");
        return;
      }
      applySnapshot(withShortcut);
    }
    setSaveState("saved");
    window.setTimeout(() => setSaveState("idle"), 1600);
  };

  const captureShortcut = (event: KeyboardEvent<HTMLInputElement>) => {
    event.preventDefault();
    const modifiers = [event.ctrlKey ? "Ctrl" : "", event.altKey ? "Alt" : "", event.shiftKey ? "Shift" : "", event.metaKey ? "Super" : ""].filter(Boolean);
    if (["Control", "Alt", "Shift", "Meta"].includes(event.key)) return;
    const key = event.key === " " ? "Space" : event.key.length === 1 ? event.key.toUpperCase() : event.key;
    setShortcut([...modifiers, key].join("+"));
  };

  const updateLink = (index: number, field: "title" | "url", value: string) =>
    setQuickLinks((items) => items.map((item, itemIndex) => (itemIndex === index ? { ...item, [field]: value } : item)));

  return (
    <div className="pt-7 pb-0">
      <div className="mx-auto w-full max-w-[820px] px-6">
        <header className="pb-4">
          <h1 className="text-xl font-semibold">设置</h1>
        </header>

        <div className="flex flex-col gap-4">
        <SettingsGroup title="外观" description="跟随系统明暗，也可以固定为亮色或暗色，仅作用于 QuickPane 界面。">
          <FieldRow label="主题">
            <Select value={theme} onValueChange={(value) => setTheme(value as ThemePreference)}>
              <SelectTrigger size="sm" className="w-40" aria-label="界面主题">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="system">跟随系统</SelectItem>
                <SelectItem value="light">亮色</SelectItem>
                <SelectItem value="dark">暗色</SelectItem>
              </SelectContent>
            </Select>
          </FieldRow>
        </SettingsGroup>

        <SettingsGroup title="老板键" description="组合键仅在 QuickPane 运行或驻留托盘时有效。">
          <Field label="显示 / 隐藏快捷键">
            <Input value={shortcut} onKeyDown={captureShortcut} onChange={() => {}} placeholder="点击后按下组合键" className="w-full max-w-[280px]" />
          </Field>
          <ToggleRow
            id="autostart"
            label="随 Windows 启动"
            description="启动后只驻留后台，不预加载网页。"
            checked={autostart}
            onCheckedChange={setAutostart}
          />
        </SettingsGroup>

        <SettingsGroup title="浏览" description="地址栏会识别网址，其余内容使用搜索引擎查询。">
          <Field label="主页">
            <Input value={homeUrl} onChange={(event) => setHomeUrl(event.target.value)} />
          </Field>
          <Field label="搜索地址">
            <Input value={searchTemplate} onChange={(event) => setSearchTemplate(event.target.value)} />
            <p className="text-xs text-muted-foreground">使用 {"{query}"} 作为搜索词占位符。</p>
          </Field>
          <Field label="历史记录保留">
            <div className="flex items-center gap-2">
              <Input
                type="number"
                min={1}
                max={3650}
                step={1}
                value={historyDays}
                onChange={(event) => {
                  const value = Number(event.target.value);
                  if (Number.isInteger(value) && value >= 1 && value <= 3650) setHistoryDays(value);
                }}
                className="w-24"
              />
              <span className="text-xs text-muted-foreground">天</span>
            </div>
          </Field>
        </SettingsGroup>

        <SettingsGroup
          title="网络代理"
          description="代理在标签页创建时生效，更改后已打开的页面会自动重新加载连接。"
        >
          <Field label="代理模式">
            <Select value={proxyMode} onValueChange={(value) => setProxyMode(value as ProxyMode)}>
              <SelectTrigger size="sm" className="w-40" aria-label="代理模式">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="system">跟随系统代理</SelectItem>
                <SelectItem value="direct">不使用代理</SelectItem>
                <SelectItem value="custom">自定义代理</SelectItem>
              </SelectContent>
            </Select>
          </Field>
          {proxyMode === "custom" ? (
            <Field label="代理地址">
              <Input
                value={proxyUrl}
                onChange={(event) => setProxyUrl(event.target.value)}
                placeholder="http://127.0.0.1:7890 或 socks5://127.0.0.1:1080"
              />
              <p className="text-xs text-muted-foreground">支持 http、https、socks4、socks5。</p>
            </Field>
          ) : null}
        </SettingsGroup>

        <SettingsGroup title="快捷站点" description="最多 12 个，显示在新标签页。">
          <div className="flex flex-col gap-2">
            {quickLinks.map((link, index) => (
              <div className="grid grid-cols-1 gap-2 sm:grid-cols-[130px_minmax(0,1fr)_auto]" key={link.id}>
                <Input aria-label="站点名称" value={link.title} onChange={(event) => updateLink(index, "title", event.target.value)} />
                <Input aria-label="站点网址" value={link.url} onChange={(event) => updateLink(index, "url", event.target.value)} />
                <Button
                  variant="ghost"
                  size="iconSm"
                  aria-label="删除站点"
                  onClick={() => setQuickLinks((items) => items.filter((_, itemIndex) => itemIndex !== index))}
                  className="self-center text-muted-foreground hover:bg-destructive-soft hover:text-destructive"
                >
                  <X className="size-3.5" />
                </Button>
              </div>
            ))}
          </div>
          {quickLinks.length < 12 ? (
            <Button
              variant="ghost"
              size="sm"
              className="self-start text-primary hover:bg-soft hover:text-primary"
              onClick={() => setQuickLinks((items) => [...items, { id: crypto.randomUUID(), title: "新站点", url: "https://" }])}
            >
              <Plus className="size-4" />添加站点
            </Button>
          ) : null}
        </SettingsGroup>

        <SettingsGroup title="应用锁" description="应用密码只保护界面入口，不加密整个 WebView2 数据目录。">
          <ToggleRow
            id="system-lock"
            label="Windows 锁屏时自动锁定"
            description="再次呼出 QuickPane 时要求应用密码。"
            checked={lockOnSystemLock}
            onCheckedChange={setLockOnSystemLock}
            disabled={!hasPassword}
          />
          <FieldRow label="隐藏后自动锁定">
            <Select
              value={String(autoLockAfterHideSeconds)}
              onValueChange={(value) => setAutoLockAfterHideSeconds(Number(value))}
              disabled={!hasPassword}
            >
              <SelectTrigger size="sm" className="w-40" aria-label="隐藏后自动锁定时间">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="0">关闭</SelectItem>
                <SelectItem value="60">1 分钟</SelectItem>
                <SelectItem value="300">5 分钟</SelectItem>
                <SelectItem value="900">15 分钟</SelectItem>
              </SelectContent>
            </Select>
          </FieldRow>
          <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
            {hasPassword ? (
              <Field label="当前密码">
                <Input type={revealPassword ? "text" : "password"} value={currentPassword} onChange={(event) => setCurrentPassword(event.target.value)} />
              </Field>
            ) : null}
            <Field label={hasPassword ? "新密码" : "设置密码"}>
              <div className="relative">
                <Input
                  type={revealPassword ? "text" : "password"}
                  value={password}
                  onChange={(event) => setPassword(event.target.value)}
                  placeholder="至少 4 个字符"
                  className="pr-9"
                />
                <button
                  type="button"
                  onClick={() => setRevealPassword(!revealPassword)}
                  aria-label="显示或隐藏密码"
                  className="absolute top-1/2 right-1 grid size-7 -translate-y-1/2 place-items-center rounded-sm text-muted-foreground transition-colors hover:bg-muted focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
                >
                  {revealPassword ? <EyeOff className="size-4" /> : <Eye className="size-4" />}
                </button>
              </div>
            </Field>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="secondary"
              disabled={!password}
              onClick={() => void run(() => api.setPassword(password, currentPassword || undefined)).then((next) => {
                if (next) {
                  applySnapshot(next);
                  setPassword("");
                  setCurrentPassword("");
                }
              })}
            >
              {hasPassword ? "更改密码" : "启用应用锁"}
            </Button>
            {hasPassword ? (
              <Button
                variant="ghost"
                disabled={!currentPassword}
                className="text-destructive hover:bg-destructive-soft hover:text-destructive"
                onClick={() => void run(() => api.disablePassword(currentPassword)).then((next) => {
                  if (next) applySnapshot(next);
                })}
              >
                停用应用锁
              </Button>
            ) : null}
          </div>
        </SettingsGroup>

        <SettingsGroup title="软件更新" description="从配置的更新源检查并安装 QuickPane 新版本。">
          <div className="flex flex-wrap items-center gap-2">
            <Button variant="secondary" disabled={checkingUpdate || installingUpdate} onClick={() => void checkForUpdate()}>
              {checkingUpdate ? <LoaderCircle className="size-4 animate-spin" /> : <Download className="size-4" />}
              {checkingUpdate ? "检查中" : "检查更新"}
            </Button>
            {update ? (
              <Button disabled={checkingUpdate || installingUpdate} onClick={() => void installAvailableUpdate()}>
                {installingUpdate ? <LoaderCircle className="size-4 animate-spin" /> : null}
                {installingUpdate ? "下载中" : `安装 ${update.version}`}
              </Button>
            ) : null}
          </div>
          {updateProgress ? (
            <div className="flex flex-col gap-1.5">
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>下载更新</span>
                <span>{updateProgress.total ? `${Math.round((updateProgress.downloaded / updateProgress.total) * 100)}%` : "下载中"}</span>
              </div>
              <div
                className="relative h-1.5 overflow-hidden rounded-full bg-muted"
                role="progressbar"
                aria-label="更新下载进度"
                aria-valuemin={0}
                aria-valuemax={updateProgress.total || undefined}
                aria-valuenow={updateProgress.total ? updateProgress.downloaded : undefined}
                aria-valuetext={updateProgress.total ? `${Math.round((updateProgress.downloaded / updateProgress.total) * 100)}%` : "下载中"}
              >
                <div className="relative h-full overflow-hidden rounded-full bg-gradient-to-r from-primary to-accent2 transition-[width]" style={{ width: updateProgress.total ? `${Math.min(100, (updateProgress.downloaded / updateProgress.total) * 100)}%` : "35%" }}>
                  <span aria-hidden className="absolute inset-0 animate-shimmer bg-gradient-to-r from-transparent via-white/25 to-transparent dark:via-white/15" />
                </div>
              </div>
            </div>
          ) : null}
          {updateMessage ? <p className="text-xs text-muted-foreground">{updateMessage}</p> : null}
        </SettingsGroup>

        <SettingsGroup title="浏览数据" description="清除站点数据会退出所有网站，并删除 WebView2 缓存。">
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button variant="secondary" className="self-start border-destructive-border text-destructive hover:bg-destructive-soft hover:text-destructive">
                清除 Cookie 和站点数据
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogTitle>清除 Cookie 和站点数据？</AlertDialogTitle>
              <AlertDialogDescription>将退出所有已登录的网站并删除 WebView2 缓存，此操作不可撤销。</AlertDialogDescription>
              <AlertDialogFooter>
                <AlertDialogCancel>取消</AlertDialogCancel>
                <AlertDialogAction variant="destructive" onClick={() => void run(api.clearSiteData)}>清除</AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </SettingsGroup>
      </div>
      </div>

      <div className="sticky bottom-0 mt-6 flex items-center justify-end gap-3 border-t border-border bg-background/85 px-6 py-3 backdrop-blur">
        {saveState === "saved" ? <span className="text-xs text-success">设置已保存</span> : null}
        {saveState === "error" ? <span role="alert" className="text-xs text-destructive">保存失败，请检查输入后重试</span> : null}
        <Button disabled={saveState === "saving"} onClick={() => void save()} className="min-w-[104px]">
          {saveState === "saving" ? <><LoaderCircle className="size-4 animate-spin" />保存中</> : saveState === "saved" ? <><Check className="size-4" />已保存</> : "保存更改"}
        </Button>
      </div>
    </div>
  );
}

/** 分组卡片：左侧标题说明，右侧控件（窄屏折叠为单列） */
function SettingsGroup({ title, description, children }: { title: string; description: string; children: ReactNode }) {
  return (
    <Card className="grid gap-3 p-5 md:grid-cols-[minmax(150px,210px)_minmax(0,1fr)] md:gap-6">
      <CardHeader className="gap-1">
        <CardTitle className="text-sm">{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3.5">{children}</CardContent>
    </Card>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="flex flex-col gap-1.5">
      <span className="text-xs font-medium text-foreground">{label}</span>
      {children}
    </label>
  );
}

function FieldRow({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-xs font-medium text-foreground">{label}</span>
      {children}
    </div>
  );
}

function ToggleRow({ id, label, description, checked, onCheckedChange, disabled = false }: {
  id: string;
  label: string;
  description: string;
  checked: boolean;
  onCheckedChange: (value: boolean) => void;
  disabled?: boolean;
}) {
  const labelId = `toggle-${id}`;
  const descriptionId = `${labelId}-description`;
  return (
    <div className={cn("flex items-center justify-between gap-4", disabled && "opacity-50")}>
      <div className="min-w-0">
        <p id={labelId} className="text-xs font-medium">{label}</p>
        <p id={descriptionId} className="mt-0.5 text-xs text-muted-foreground">{description}</p>
      </div>
      <Switch checked={checked} onCheckedChange={onCheckedChange} disabled={disabled} aria-labelledby={labelId} aria-describedby={descriptionId} />
    </div>
  );
}

export { SettingsPage };
