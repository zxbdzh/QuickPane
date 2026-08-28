import {
  ArrowLeft,
  ArrowRight,
  Bookmark as BookmarkIcon,
  Check,
  CircleAlert,
  Clock3,
  Download,
  ExternalLink,
  Eye,
  EyeOff,
  FileDown,
  Globe2,
  History,
  Home,
  KeyRound,
  LoaderCircle,
  Lock,
  Menu,
  PanelTopClose,
  Plus,
  RefreshCw,
  Search,
  Settings as SettingsIcon,
  ShieldCheck,
  Star,
  Trash2,
  X,
} from "lucide-react";
import { FormEvent, KeyboardEvent, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "./api";
import type { AppSnapshot, QuickLink, ShellSection, TabRecord } from "./types";
import "./App.css";

const EMPTY_SNAPSHOT: AppSnapshot = {
  data: {
    tabs: [],
    activeTabId: null,
    recentlyClosed: [],
    history: [],
    bookmarks: [],
    downloads: [],
    settings: {
      shortcut: null,
      autostart: false,
      homeUrl: "https://kaodes.com",
      searchTemplate: "https://cn.bing.com/search?q={query}",
      historyDays: 90,
      passwordHash: null,
      lockOnSystemLock: true,
      quickLinks: [],
    },
  },
  locked: false,
  firstRun: true,
  windowVisible: true,
};

function App() {
  const [snapshot, setSnapshot] = useState<AppSnapshot>(EMPTY_SNAPSHOT);
  const [section, setSection] = useState<ShellSection>("newtab");
  const [ready, setReady] = useState(false);
  const [address, setAddress] = useState("");
  const [menuOpen, setMenuOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [zoom, setZoom] = useState(1);
  const addressRef = useRef<HTMLInputElement>(null);

  const activeTab = useMemo(
    () => snapshot.data.tabs.find((tab) => tab.id === snapshot.data.activeTabId) ?? null,
    [snapshot.data.activeTabId, snapshot.data.tabs],
  );

  const applySnapshot = useCallback((next: AppSnapshot) => {
    setSnapshot(next);
    setError(null);
    const active = next.data.tabs.find((tab) => tab.id === next.data.activeTabId);
    if (active?.url === "quickpane://newtab") setSection("newtab");
    else if (!next.locked && !next.firstRun) setSection((current) =>
      current === "lock" || current === "newtab" ? "newtab" : current,
    );
  }, []);

  const run = useCallback(async <T,>(action: () => Promise<T>): Promise<T | undefined> => {
    try {
      setError(null);
      return await action();
    } catch (reason) {
      const message = reason instanceof Error ? reason.message : String(reason);
      setError(message);
      return undefined;
    }
  }, []);

  useEffect(() => {
    void api.snapshot().then((next) => {
      setSnapshot(next);
      setReady(true);
      if (next.locked) setSection("lock");
      else if (next.firstRun) setSection("lock");
      else {
        const active = next.data.tabs.find((tab) => tab.id === next.data.activeTabId);
        setSection(active?.url === "quickpane://newtab" ? "newtab" : "newtab");
        if (active?.url && active.url !== "quickpane://newtab") {
          void api.selectTab(active.id).then(setSnapshot).catch(setErrorFromUnknown);
        }
      }
    }).catch((reason) => {
      setError(setErrorFromUnknown(reason));
      setReady(true);
    });

    const cleanups = Promise.all([
      listen<AppSnapshot>("app-snapshot", (event) => setSnapshot(event.payload)),
      listen<string>("open-section", (event) => {
        const next = event.payload as ShellSection;
        setSection(next);
        void api.showShell(true);
      }),
      listen<string>("new-window-requested", (event) => {
        void api.newTab(event.payload, true).then((next) => {
          setSnapshot(next);
          setSection("newtab");
        }).catch((reason) => setError(setErrorFromUnknown(reason)));
      }),
      listen<string>("shortcut-error", (event) => setError(event.payload)),
    ]);
    return () => { void cleanups.then((items) => items.forEach((cleanup) => cleanup())); };
  }, []);

  useEffect(() => {
    setAddress(activeTab?.url === "quickpane://newtab" ? "" : activeTab?.url ?? "");
  }, [activeTab?.id, activeTab?.url]);

  const openSection = useCallback((next: ShellSection) => {
    setMenuOpen(false);
    setSection(next);
    void run(async () => applySnapshot(await api.showShell(true)));
  }, [applySnapshot, run]);

  const selectTab = useCallback((tab: TabRecord) => {
    setMenuOpen(false);
    setSection(tab.url === "quickpane://newtab" ? "newtab" : "newtab");
    void run(async () => applySnapshot(await api.selectTab(tab.id)));
  }, [applySnapshot, run]);

  const createTab = useCallback((url?: string) => {
    setSection("newtab");
    void run(async () => applySnapshot(await api.newTab(url, true)));
  }, [applySnapshot, run]);

  const submitAddress = useCallback((event?: FormEvent) => {
    event?.preventDefault();
    if (!activeTab || !address.trim()) return;
    setSection("newtab");
    void run(async () => applySnapshot(await api.navigate(activeTab.id, address)));
  }, [activeTab, address, applySnapshot, run]);

  useEffect(() => {
    const handler = (event: globalThis.KeyboardEvent) => {
      const key = event.key.toLowerCase();
      if (event.ctrlKey && key === "l") {
        event.preventDefault();
        addressRef.current?.focus();
        addressRef.current?.select();
      } else if (event.ctrlKey && key === "t") {
        event.preventDefault();
        createTab();
      } else if (event.ctrlKey && key === "w" && activeTab) {
        event.preventDefault();
        void run(async () => applySnapshot(await api.removeTab(activeTab.id)));
      } else if (event.ctrlKey && event.shiftKey && key === "t") {
        event.preventDefault();
        const last = snapshot.data.recentlyClosed[0];
        if (last) createTab(last.url);
      } else if (event.ctrlKey && key === "h") {
        event.preventDefault();
        openSection("history");
      } else if (event.ctrlKey && key === "j") {
        event.preventDefault();
        openSection("downloads");
      } else if (event.ctrlKey && key === "d" && activeTab?.url.startsWith("http")) {
        event.preventDefault();
        void run(async () => applySnapshot(await api.addBookmark(activeTab.title, activeTab.url)));
      } else if (event.ctrlKey && key === "f") {
        event.preventDefault();
        void run(api.find);
      } else if (event.ctrlKey && ["+", "="].includes(event.key)) {
        event.preventDefault();
        const next = Math.min(5, zoom + 0.1);
        setZoom(next);
        void run(() => api.zoom(next));
      } else if (event.ctrlKey && event.key === "-") {
        event.preventDefault();
        const next = Math.max(0.25, zoom - 0.1);
        setZoom(next);
        void run(() => api.zoom(next));
      } else if (event.ctrlKey && event.key === "0") {
        event.preventDefault();
        setZoom(1);
        void run(() => api.zoom(1));
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [activeTab, applySnapshot, createTab, openSection, run, snapshot.data.recentlyClosed, zoom]);

  if (!ready) return <div className="boot-screen"><LoaderCircle className="spin" />正在打开 QuickPane</div>;

  const locked = snapshot.locked || snapshot.firstRun;
  const shellVisible = locked || activeTab?.url === "quickpane://newtab" || section !== "newtab";

  return (
    <main className={`app ${shellVisible ? "shell-visible" : "browsing"}`}>
      {!locked ? (
        <>
          <TabStrip
            tabs={snapshot.data.tabs}
            activeId={snapshot.data.activeTabId}
            onSelect={selectTab}
            onClose={(id) => void run(async () => applySnapshot(await api.removeTab(id)))}
            onNew={() => createTab()}
          />
          <NavigationBar
            activeTab={activeTab}
            address={address}
            onAddress={setAddress}
            onSubmit={submitAddress}
            addressRef={addressRef}
            bookmarked={Boolean(activeTab && snapshot.data.bookmarks.some((item) => item.url === activeTab.url))}
            onBookmark={() => {
              if (!activeTab?.url.startsWith("http")) return;
              void run(async () => applySnapshot(await api.addBookmark(activeTab.title, activeTab.url)));
            }}
            onBack={() => void run(api.back)}
            onForward={() => void run(api.forward)}
            onReload={() => void run(api.reload)}
            onHome={() => createTab(snapshot.data.settings.homeUrl)}
            onHide={() => void run(api.hide)}
            menuOpen={menuOpen}
            setMenuOpen={setMenuOpen}
            openSection={openSection}
            onLock={() => void run(api.lockNow)}
            hasPassword={Boolean(snapshot.data.settings.passwordHash)}
          />
        </>
      ) : null}

      {error ? <div className="error-toast" role="alert"><CircleAlert size={16} />{error}<button onClick={() => setError(null)} aria-label="关闭错误"><X size={15} /></button></div> : null}

      <section className={`content ${locked ? "content-locked" : ""}`}>
        {locked ? (
          <LockScreen snapshot={snapshot} applySnapshot={applySnapshot} run={run} />
        ) : section === "newtab" && activeTab?.url === "quickpane://newtab" ? (
          <NewTabPage snapshot={snapshot} onNavigate={(url) => void run(async () => applySnapshot(await api.navigate(activeTab.id, url)))} onSection={openSection} />
        ) : section === "history" ? (
          <HistoryPage snapshot={snapshot} onOpen={createTab} onClear={() => void run(async () => applySnapshot(await api.clearHistory()))} />
        ) : section === "bookmarks" ? (
          <BookmarksPage snapshot={snapshot} onOpen={createTab} onRemove={(id) => void run(async () => applySnapshot(await api.removeBookmark(id)))} />
        ) : section === "downloads" ? (
          <DownloadsPage snapshot={snapshot} onOpen={(path) => void run(() => api.openDownload(path))} onClear={() => void run(async () => applySnapshot(await api.clearDownloads()))} />
        ) : section === "settings" ? (
          <SettingsPage snapshot={snapshot} applySnapshot={applySnapshot} run={run} />
        ) : null}
      </section>
    </main>
  );
}

function TabStrip({ tabs, activeId, onSelect, onClose, onNew }: {
  tabs: TabRecord[];
  activeId: string | null;
  onSelect: (tab: TabRecord) => void;
  onClose: (id: string) => void;
  onNew: () => void;
}) {
  return (
    <div className="tab-strip">
      <div className="tab-scroll">
        {tabs.map((tab) => (
          <button className={`tab ${tab.id === activeId ? "active" : ""}`} key={tab.id} onClick={() => onSelect(tab)}>
            {tab.loading ? <LoaderCircle className="spin" size={14} /> : <Globe2 size={14} />}
            <span>{tab.title || "新标签页"}</span>
            <i
              role="button"
              tabIndex={0}
              aria-label={`关闭 ${tab.title}`}
              onClick={(event) => { event.stopPropagation(); onClose(tab.id); }}
              onKeyDown={(event) => { if (event.key === "Enter") onClose(tab.id); }}
            ><X size={13} /></i>
          </button>
        ))}
      </div>
      <IconButton title="新建标签页 (Ctrl+T)" onClick={onNew}><Plus size={16} /></IconButton>
      <div className="brand-mark" title="QuickPane"><span>Q</span></div>
    </div>
  );
}

function NavigationBar({ activeTab, address, onAddress, onSubmit, addressRef, bookmarked, onBookmark, onBack, onForward, onReload, onHome, onHide, menuOpen, setMenuOpen, openSection, onLock, hasPassword }: {
  activeTab: TabRecord | null;
  address: string;
  onAddress: (value: string) => void;
  onSubmit: (event: FormEvent) => void;
  addressRef: React.RefObject<HTMLInputElement | null>;
  bookmarked: boolean;
  onBookmark: () => void;
  onBack: () => void;
  onForward: () => void;
  onReload: () => void;
  onHome: () => void;
  onHide: () => void;
  menuOpen: boolean;
  setMenuOpen: (open: boolean) => void;
  openSection: (section: ShellSection) => void;
  onLock: () => void;
  hasPassword: boolean;
}) {
  return (
    <nav className="navigation-bar">
      <div className="nav-actions">
        <IconButton title="后退 (Alt+左箭头)" onClick={onBack}><ArrowLeft size={17} /></IconButton>
        <IconButton title="前进 (Alt+右箭头)" onClick={onForward}><ArrowRight size={17} /></IconButton>
        <IconButton title="刷新 (Ctrl+R)" onClick={onReload}><RefreshCw size={16} /></IconButton>
        <IconButton title="主页" onClick={onHome}><Home size={16} /></IconButton>
      </div>
      <form className="address-form" onSubmit={onSubmit}>
        <ShieldCheck size={15} className={address.startsWith("https://") ? "secure" : "muted"} />
        <input ref={addressRef} value={address} onChange={(event) => onAddress(event.target.value)} placeholder="输入网址或搜索内容" spellCheck={false} />
        {activeTab?.loading ? <LoaderCircle className="spin" size={15} /> : null}
        <button type="button" className={bookmarked ? "starred" : ""} onClick={onBookmark} title="收藏此页 (Ctrl+D)" aria-label="收藏此页"><Star size={16} fill={bookmarked ? "currentColor" : "none"} /></button>
      </form>
      <div className="nav-actions right">
        <IconButton title="立即隐藏" onClick={onHide}><PanelTopClose size={17} /></IconButton>
        <div className="menu-anchor">
          <IconButton title="QuickPane 菜单" onClick={() => setMenuOpen(!menuOpen)}><Menu size={18} /></IconButton>
          {menuOpen ? (
            <div className="app-menu">
              <MenuAction icon={<History size={16} />} label="历史记录" hint="Ctrl+H" onClick={() => openSection("history")} />
              <MenuAction icon={<BookmarkIcon size={16} />} label="书签" onClick={() => openSection("bookmarks")} />
              <MenuAction icon={<Download size={16} />} label="下载" hint="Ctrl+J" onClick={() => openSection("downloads")} />
              <div className="menu-divider" />
              {hasPassword ? <MenuAction icon={<Lock size={16} />} label="立即锁定" onClick={onLock} /> : null}
              <MenuAction icon={<SettingsIcon size={16} />} label="设置" onClick={() => openSection("settings")} />
            </div>
          ) : null}
        </div>
      </div>
    </nav>
  );
}

function NewTabPage({ snapshot, onNavigate, onSection }: { snapshot: AppSnapshot; onNavigate: (url: string) => void; onSection: (section: ShellSection) => void }) {
  const [query, setQuery] = useState("");
  return (
    <div className="new-tab-page page-width">
      <header className="new-tab-header">
        <div className="monogram">Q</div>
        <div><h1>QuickPane</h1><p>你的轻量浏览空间</p></div>
      </header>
      <form className="new-tab-search" onSubmit={(event) => { event.preventDefault(); if (query.trim()) onNavigate(query); }}>
        <Search size={19} />
        <input autoFocus value={query} onChange={(event) => setQuery(event.target.value)} placeholder="搜索或输入网址" />
        <kbd>Enter</kbd>
      </form>
      <div className="quick-links">
        {snapshot.data.settings.quickLinks.map((link) => (
          <button key={link.id} onClick={() => onNavigate(link.url)}>
            <span>{link.title.slice(0, 1).toUpperCase()}</span>
            <strong>{link.title}</strong>
            <small>{hostOf(link.url)}</small>
          </button>
        ))}
        <button className="add-link" onClick={() => onSection("settings")}><span><Plus size={18} /></span><strong>添加站点</strong><small>在设置中管理</small></button>
      </div>
      <div className="recent-row">
        <button onClick={() => onSection("history")}><Clock3 size={16} />最近访问<span>{snapshot.data.history.length}</span></button>
        <button onClick={() => onSection("bookmarks")}><Star size={16} />已收藏<span>{snapshot.data.bookmarks.length}</span></button>
        <button onClick={() => onSection("downloads")}><FileDown size={16} />下载记录<span>{snapshot.data.downloads.length}</span></button>
      </div>
    </div>
  );
}

function PageHeader({ title, count, action }: { title: string; count?: number; action?: React.ReactNode }) {
  return <header className="page-header"><div><h1>{title}</h1>{count !== undefined ? <p>{count} 项</p> : null}</div>{action}</header>;
}

function HistoryPage({ snapshot, onOpen, onClear }: { snapshot: AppSnapshot; onOpen: (url: string) => void; onClear: () => void }) {
  return <DataPage title="历史记录" count={snapshot.data.history.length} onClear={onClear} emptyIcon={<History />} emptyText="还没有访问记录">
    {snapshot.data.history.map((item) => <DataRow key={item.id} icon={<Globe2 size={17} />} title={item.title || hostOf(item.url)} subtitle={item.url} meta={formatDate(item.visitedAt)} onOpen={() => onOpen(item.url)} />)}
  </DataPage>;
}

function BookmarksPage({ snapshot, onOpen, onRemove }: { snapshot: AppSnapshot; onOpen: (url: string) => void; onRemove: (id: string) => void }) {
  return <DataPage title="书签" count={snapshot.data.bookmarks.length} emptyIcon={<Star />} emptyText="收藏的网页会显示在这里">
    {snapshot.data.bookmarks.map((item) => <DataRow key={item.id} icon={<Star size={17} />} title={item.title || hostOf(item.url)} subtitle={item.url} meta={formatDate(item.createdAt)} onOpen={() => onOpen(item.url)} action={<IconButton title="删除书签" onClick={() => onRemove(item.id)}><Trash2 size={15} /></IconButton>} />)}
  </DataPage>;
}

function DownloadsPage({ snapshot, onOpen, onClear }: { snapshot: AppSnapshot; onOpen: (path: string) => void; onClear: () => void }) {
  return <DataPage title="下载" count={snapshot.data.downloads.length} onClear={onClear} emptyIcon={<Download />} emptyText="下载记录会显示在这里">
    {snapshot.data.downloads.map((item) => <DataRow key={item.id} icon={item.state === "downloading" ? <LoaderCircle className="spin" size={17} /> : item.state === "failed" ? <CircleAlert size={17} /> : <Check size={17} />} title={item.fileName} subtitle={item.path ?? item.url} meta={item.state === "completed" ? "已完成" : item.state === "failed" ? "失败" : "下载中"} onOpen={item.path ? () => onOpen(item.path!) : undefined} />)}
  </DataPage>;
}

function DataPage({ title, count, onClear, emptyIcon, emptyText, children }: { title: string; count: number; onClear?: () => void; emptyIcon: React.ReactNode; emptyText: string; children: React.ReactNode }) {
  return <div className="data-page page-width">
    <PageHeader title={title} count={count} action={onClear && count ? <button className="text-button danger" onClick={onClear}><Trash2 size={15} />清除记录</button> : undefined} />
    <div className="data-list">{count ? children : <div className="empty-state"><span>{emptyIcon}</span><h2>{emptyText}</h2></div>}</div>
  </div>;
}

function DataRow({ icon, title, subtitle, meta, onOpen, action }: { icon: React.ReactNode; title: string; subtitle: string; meta: string; onOpen?: () => void; action?: React.ReactNode }) {
  return <div className="data-row"><span className="row-icon">{icon}</span><button className="row-main" onClick={onOpen} disabled={!onOpen}><strong>{title}</strong><small>{subtitle}</small></button><time>{meta}</time>{action ?? (onOpen ? <ExternalLink size={15} className="row-open" /> : null)}</div>;
}

function SettingsPage({ snapshot, applySnapshot, run }: { snapshot: AppSnapshot; applySnapshot: (value: AppSnapshot) => void; run: <T>(action: () => Promise<T>) => Promise<T | undefined> }) {
  const settings = snapshot.data.settings;
  const [shortcut, setShortcut] = useState(settings.shortcut ?? "");
  const [autostart, setAutostart] = useState(settings.autostart);
  const [homeUrl, setHomeUrl] = useState(settings.homeUrl);
  const [searchTemplate, setSearchTemplate] = useState(settings.searchTemplate);
  const [historyDays, setHistoryDays] = useState(settings.historyDays);
  const [lockOnSystemLock, setLockOnSystemLock] = useState(settings.lockOnSystemLock);
  const [quickLinks, setQuickLinks] = useState<QuickLink[]>(settings.quickLinks);
  const [password, setPassword] = useState("");
  const [currentPassword, setCurrentPassword] = useState("");
  const [revealPassword, setRevealPassword] = useState(false);
  const [saved, setSaved] = useState(false);

  const save = async () => {
    const next = await run(() => api.updateSettings({ autostart, homeUrl, searchTemplate, historyDays, lockOnSystemLock, quickLinks }));
    if (next) {
      applySnapshot(next);
      if (shortcut.trim() && shortcut !== settings.shortcut) {
        const withShortcut = await run(() => api.setShortcut(shortcut.trim()));
        if (withShortcut) applySnapshot(withShortcut);
      }
      setSaved(true);
      window.setTimeout(() => setSaved(false), 1600);
    }
  };

  const captureShortcut = (event: KeyboardEvent<HTMLInputElement>) => {
    event.preventDefault();
    const modifiers = [event.ctrlKey ? "Ctrl" : "", event.altKey ? "Alt" : "", event.shiftKey ? "Shift" : "", event.metaKey ? "Super" : ""].filter(Boolean);
    if (["Control", "Alt", "Shift", "Meta"].includes(event.key)) return;
    const key = event.key === " " ? "Space" : event.key.length === 1 ? event.key.toUpperCase() : event.key;
    setShortcut([...modifiers, key].join("+"));
  };

  const updateLink = (index: number, field: "title" | "url", value: string) => setQuickLinks((items) => items.map((item, itemIndex) => itemIndex === index ? { ...item, [field]: value } : item));

  return <div className="settings-page page-width">
    <PageHeader title="设置" />
    <div className="settings-layout">
      <SettingsGroup title="老板键" description="组合键仅在 QuickPane 运行或驻留托盘时有效。">
        <label className="field"><span>显示 / 隐藏快捷键</span><input value={shortcut} onKeyDown={captureShortcut} onChange={() => {}} placeholder="点击后按下组合键" /></label>
        <Toggle label="随 Windows 启动" description="启动后只驻留后台，不预加载网页。" checked={autostart} onChange={setAutostart} />
      </SettingsGroup>
      <SettingsGroup title="浏览" description="地址栏会识别网址，其余内容使用搜索引擎查询。">
        <label className="field"><span>主页</span><input value={homeUrl} onChange={(event) => setHomeUrl(event.target.value)} /></label>
        <label className="field"><span>搜索地址</span><input value={searchTemplate} onChange={(event) => setSearchTemplate(event.target.value)} /><small>使用 {"{query}"} 作为搜索词占位符。</small></label>
        <label className="field compact"><span>历史记录保留</span><div><input type="number" min={1} max={3650} value={historyDays} onChange={(event) => setHistoryDays(Number(event.target.value))} /><em>天</em></div></label>
      </SettingsGroup>
      <SettingsGroup title="快捷站点" description="最多 12 个，显示在新标签页。">
        <div className="link-editor">{quickLinks.map((link, index) => <div className="link-line" key={link.id}><input aria-label="站点名称" value={link.title} onChange={(event) => updateLink(index, "title", event.target.value)} /><input aria-label="站点网址" value={link.url} onChange={(event) => updateLink(index, "url", event.target.value)} /><IconButton title="删除站点" onClick={() => setQuickLinks((items) => items.filter((_, itemIndex) => itemIndex !== index))}><X size={15} /></IconButton></div>)}</div>
        {quickLinks.length < 12 ? <button className="text-button" onClick={() => setQuickLinks((items) => [...items, { id: crypto.randomUUID(), title: "新站点", url: "https://" }])}><Plus size={15} />添加站点</button> : null}
      </SettingsGroup>
      <SettingsGroup title="应用锁" description="应用密码只保护界面入口，不加密整个 WebView2 数据目录。">
        <Toggle label="Windows 锁屏时自动锁定" description="再次呼出 QuickPane 时要求应用密码。" checked={lockOnSystemLock} onChange={setLockOnSystemLock} disabled={!settings.passwordHash} />
        <div className="password-grid">
          {settings.passwordHash ? <label className="field"><span>当前密码</span><input type={revealPassword ? "text" : "password"} value={currentPassword} onChange={(event) => setCurrentPassword(event.target.value)} /></label> : null}
          <label className="field"><span>{settings.passwordHash ? "新密码" : "设置密码"}</span><div className="password-field"><input type={revealPassword ? "text" : "password"} value={password} onChange={(event) => setPassword(event.target.value)} placeholder="至少 4 个字符" /><button onClick={() => setRevealPassword(!revealPassword)} aria-label="显示或隐藏密码">{revealPassword ? <EyeOff size={16} /> : <Eye size={16} />}</button></div></label>
        </div>
        <div className="inline-actions">
          <button className="secondary-button" disabled={!password} onClick={() => void run(() => api.setPassword(password, currentPassword || undefined)).then((next) => { if (next) { applySnapshot(next); setPassword(""); setCurrentPassword(""); } })}>{settings.passwordHash ? "更改密码" : "启用应用锁"}</button>
          {settings.passwordHash ? <button className="text-button danger" disabled={!currentPassword} onClick={() => void run(() => api.disablePassword(currentPassword)).then((next) => { if (next) applySnapshot(next); })}>停用应用锁</button> : null}
        </div>
      </SettingsGroup>
      <SettingsGroup title="浏览数据" description="清除站点数据会退出所有网站，并删除 WebView2 缓存。">
        <button className="secondary-button danger-outline" onClick={() => void run(api.clearSiteData)}>清除 Cookie 和站点数据</button>
      </SettingsGroup>
    </div>
    <div className="settings-footer"><button className="primary-button" onClick={() => void save()}>{saved ? <><Check size={16} />已保存</> : "保存更改"}</button></div>
  </div>;
}

function SettingsGroup({ title, description, children }: { title: string; description: string; children: React.ReactNode }) {
  return <section className="settings-group"><header><h2>{title}</h2><p>{description}</p></header><div className="settings-controls">{children}</div></section>;
}

function Toggle({ label, description, checked, onChange, disabled = false }: { label: string; description: string; checked: boolean; onChange: (value: boolean) => void; disabled?: boolean }) {
  return <label className={`toggle-row ${disabled ? "disabled" : ""}`}><div><strong>{label}</strong><small>{description}</small></div><input type="checkbox" checked={checked} disabled={disabled} onChange={(event) => onChange(event.target.checked)} /><span className="toggle"><i /></span></label>;
}

function LockScreen({ snapshot, applySnapshot, run }: { snapshot: AppSnapshot; applySnapshot: (value: AppSnapshot) => void; run: <T>(action: () => Promise<T>) => Promise<T | undefined> }) {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [reveal, setReveal] = useState(false);
  const hasPassword = Boolean(snapshot.data.settings.passwordHash);
  const firstSetup = snapshot.firstRun && !hasPassword;

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (firstSetup) {
      if (password !== confirm) return;
      const next = await run(() => api.setPassword(password));
      if (next) applySnapshot(next);
    } else {
      const next = await run(() => api.unlock(password));
      if (next) applySnapshot(next);
    }
  };

  return <div className="lock-screen">
    <div className="lock-panel">
      <div className="lock-symbol">{firstSetup ? <KeyRound size={27} /> : <Lock size={27} />}</div>
      <h1>{firstSetup ? "保护 QuickPane" : "QuickPane 已锁定"}</h1>
      <p>{firstSetup ? "应用密码可选。启用后，冷启动和 Windows 锁屏后需要验证。" : "输入应用密码以恢复上次会话。"}</p>
      <form onSubmit={submit}>
        <div className="password-field large"><input autoFocus type={reveal ? "text" : "password"} value={password} onChange={(event) => setPassword(event.target.value)} placeholder={firstSetup ? "设置应用密码" : "应用密码"} /><button type="button" onClick={() => setReveal(!reveal)} aria-label="显示或隐藏密码">{reveal ? <EyeOff size={17} /> : <Eye size={17} />}</button></div>
        {firstSetup ? <div className="password-field large"><input type={reveal ? "text" : "password"} value={confirm} onChange={(event) => setConfirm(event.target.value)} placeholder="再次输入密码" /></div> : null}
        {firstSetup && confirm && password !== confirm ? <small className="field-error">两次输入的密码不一致</small> : null}
        <button className="primary-button wide" disabled={password.length < 4 || (firstSetup && password !== confirm)}>{firstSetup ? "启用并继续" : "解锁"}</button>
      </form>
      {firstSetup ? <button className="text-button skip" onClick={() => void run(api.skipPasswordSetup).then((next) => { if (next) applySnapshot(next); })}>暂不设置</button> : <small className="reset-note">忘记密码只能清除全部 QuickPane 数据后重置。</small>}
    </div>
  </div>;
}

function IconButton({ title, onClick, children }: { title: string; onClick: () => void; children: React.ReactNode }) {
  return <button className="icon-button" title={title} aria-label={title} onClick={onClick}>{children}</button>;
}

function MenuAction({ icon, label, hint, onClick }: { icon: React.ReactNode; label: string; hint?: string; onClick: () => void }) {
  return <button onClick={onClick}>{icon}<span>{label}</span>{hint ? <kbd>{hint}</kbd> : null}</button>;
}

function hostOf(url: string) {
  try { return new URL(url).hostname; } catch { return url; }
}

function formatDate(value: string) {
  const date = new Date(value);
  const today = new Date();
  return date.toDateString() === today.toDateString()
    ? date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })
    : date.toLocaleDateString("zh-CN", { month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit" });
}

function setErrorFromUnknown(reason: unknown) {
  return reason instanceof Error ? reason.message : String(reason);
}

export default App;
