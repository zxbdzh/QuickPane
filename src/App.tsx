import { LoaderCircle } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import {
  type FormEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { listen } from "@tauri-apps/api/event";

import { api } from "./api";
import type { AppSnapshot, ShellSection, TabRecord } from "./types";
import { pageFade } from "./lib/motion";
import {
  getAddressSuggestions,
  type AddressSuggestion,
} from "./lib/address-suggestions";
import { browserShortcutFromKey } from "./lib/browser-shortcuts";
import { TooltipProvider } from "./components/ui/tooltip";
import { TabStrip } from "./components/tab-strip";
import { NavigationBar } from "./components/navigation-bar";
import { NewTabPage } from "./components/new-tab-page";
import {
  BookmarksPage,
  DownloadsPage,
  HistoryPage,
} from "./components/data-page";
import { ExtensionsPage } from "./components/extensions-page";
import { SettingsPage } from "./components/settings-page";
import { LockScreen } from "./components/lock-screen";
import { ErrorBanner } from "./components/error-banner";

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
      autoLockAfterHideSeconds: 0,
      quickLinks: [],
      proxyMode: "system",
      proxyUrl: "",
      pinnedExtensions: [],
    },
  },
  locked: false,
  firstRun: true,
  hasPassword: false,
  windowVisible: true,
  pinnedExtensions: [],
  recoveryMessage: null,
};

function App() {
  const [snapshot, setSnapshot] = useState<AppSnapshot>(EMPTY_SNAPSHOT);
  const [section, setSection] = useState<ShellSection>("newtab");
  const [ready, setReady] = useState(false);
  const [address, setAddress] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [zoom, setZoom] = useState(1);
  const addressRef = useRef<HTMLInputElement>(null);
  const lastActiveUrl = useRef<string | null>(null);

  const activeTab = useMemo(
    () =>
      snapshot.data.tabs.find((tab) => tab.id === snapshot.data.activeTabId) ??
      null,
    [snapshot.data.activeTabId, snapshot.data.tabs],
  );
  const addressSuggestions = useMemo(
    () =>
      getAddressSuggestions({
        query: address,
        quickLinks: snapshot.data.settings.quickLinks,
        bookmarks: snapshot.data.bookmarks,
        history: snapshot.data.history,
        tabs: snapshot.data.tabs,
      }),
    [
      address,
      snapshot.data.bookmarks,
      snapshot.data.history,
      snapshot.data.settings.quickLinks,
      snapshot.data.tabs,
    ],
  );

  const applySnapshot = useCallback((next: AppSnapshot) => {
    setSnapshot(next);
    setError(next.recoveryMessage ?? null);
    const active = next.data.tabs.find(
      (tab) => tab.id === next.data.activeTabId,
    );
    const activeUrl = active?.url ?? null;
    setSection((current) => {
      if (next.locked) return "lock";
      if (current === "lock" && !next.firstRun) return "newtab";
      if (
        activeUrl === "quickpane://newtab" &&
        lastActiveUrl.current !== activeUrl
      ) {
        return "newtab";
      }
      return current;
    });
    lastActiveUrl.current = activeUrl;
  }, []);

  const run = useCallback(
    async <T,>(action: () => Promise<T>): Promise<T | undefined> => {
      try {
        setError(null);
        return await action();
      } catch (reason) {
        const message =
          reason instanceof Error ? reason.message : String(reason);
        setError(message);
        return undefined;
      }
    },
    [],
  );

  useEffect(() => {
    void api
      .snapshot()
      .then((next) => {
        applySnapshot(next);
        setReady(true);
        if (next.locked) setSection("lock");
        else if (next.firstRun) setSection("lock");
        else {
          const active = next.data.tabs.find(
            (tab) => tab.id === next.data.activeTabId,
          );
          setSection(
            active?.url === "quickpane://newtab" ? "newtab" : "newtab",
          );
          if (active?.url && active.url !== "quickpane://newtab") {
            void api
              .selectTab(active.id)
              .then(applySnapshot)
              .catch((reason) => setError(setErrorFromUnknown(reason)));
          }
        }
      })
      .catch((reason) => {
        setError(setErrorFromUnknown(reason));
        setReady(true);
      });

    const cleanups = Promise.all([
      listen<AppSnapshot>("app-snapshot", (event) =>
        applySnapshot(event.payload),
      ),
      listen<string>("open-section", (event) => {
        const next = event.payload as ShellSection;
        setSection(next);
        void api.showShell(true);
      }),
      listen<string>("shortcut-error", (event) => setError(event.payload)),
      listen("focus-address", () => {
        window.requestAnimationFrame(() => {
          if (document.querySelector('[role="dialog"], [role="alertdialog"]'))
            return;
          addressRef.current?.focus();
          addressRef.current?.select();
        });
      }),
    ]);
    return () => {
      void cleanups.then((items) => items.forEach((cleanup) => cleanup()));
    };
  }, [applySnapshot]);

  useEffect(() => {
    setAddress(
      activeTab?.url === "quickpane://newtab" ? "" : (activeTab?.url ?? ""),
    );
  }, [activeTab?.id, activeTab?.url]);

  const openMenu = useCallback(
    (anchor: { x: number; y: number }) => {
      void run(() => api.showMenuWindow(anchor.x, anchor.y));
    },
    [run],
  );

  const openSection = useCallback(
    (next: ShellSection) => {
      setSection(next);
      void run(async () => applySnapshot(await api.showShell(true)));
    },
    [applySnapshot, run],
  );

  const selectTab = useCallback(
    (tab: TabRecord) => {
      setSection(tab.url === "quickpane://newtab" ? "newtab" : "newtab");
      void run(async () => applySnapshot(await api.selectTab(tab.id)));
    },
    [applySnapshot, run],
  );

  const cycleTab = useCallback(
    (direction: 1 | -1) => {
      if (snapshot.data.tabs.length < 2) return;
      const currentIndex = snapshot.data.tabs.findIndex(
        (tab) => tab.id === snapshot.data.activeTabId,
      );
      const baseIndex = currentIndex >= 0 ? currentIndex : 0;
      const nextIndex =
        (baseIndex + direction + snapshot.data.tabs.length) %
        snapshot.data.tabs.length;
      const next = snapshot.data.tabs[nextIndex];
      if (next) selectTab(next);
    },
    [selectTab, snapshot.data.activeTabId, snapshot.data.tabs],
  );

  const createTab = useCallback(
    (url?: string) => {
      setSection("newtab");
      void run(async () =>
        applySnapshot(
          url && activeTab?.url === "quickpane://newtab"
            ? await api.navigate(activeTab.id, url)
            : await api.newTab(url, true),
        ),
      );
    },
    [activeTab, applySnapshot, run],
  );

  const navigateActiveTab = useCallback(
    (input: string) => {
      if (!activeTab || !input.trim()) return;
      setAddress(input);
      setSection("newtab");
      void run(async () =>
        applySnapshot(await api.navigate(activeTab.id, input)),
      );
    },
    [activeTab, applySnapshot, run],
  );

  const selectSuggestion = useCallback(
    (suggestion: AddressSuggestion) => {
      if (suggestion.tabId) {
        const tab = snapshot.data.tabs.find(
          (item) => item.id === suggestion.tabId,
        );
        if (tab) {
          selectTab(tab);
          return;
        }
      }
      navigateActiveTab(suggestion.url);
    },
    [navigateActiveTab, selectTab, snapshot.data.tabs],
  );

  const submitAddress = useCallback(
    (event?: FormEvent) => {
      event?.preventDefault();
      navigateActiveTab(address);
    },
    [address, navigateActiveTab],
  );

  const setAddressSuggestionsOpen = useCallback(
    (open: boolean) => {
      if (section !== "newtab" || activeTab?.url === "quickpane://newtab")
        return;
      void run(() => api.showShell(open));
    },
    [activeTab?.url, run, section],
  );

  const handleShortcut = useCallback(
    (shortcut: string, event?: globalThis.KeyboardEvent) => {
      if (event?.defaultPrevented || snapshot.locked || snapshot.firstRun)
        return;
      switch (shortcut) {
        case "escape":
          event?.preventDefault();
          void run(api.hide);
          break;
        case "next-tab":
        case "previous-tab":
          event?.preventDefault();
          cycleTab(shortcut === "previous-tab" ? -1 : 1);
          break;
        case "restore-tab":
          if (!snapshot.data.recentlyClosed.length) return;
          event?.preventDefault();
          void run(async () => applySnapshot(await api.restoreClosedTab()));
          break;
        case "new-tab":
          event?.preventDefault();
          createTab();
          break;
        case "focus-address":
          event?.preventDefault();
          addressRef.current?.focus();
          addressRef.current?.select();
          break;
        case "close-tab":
          if (!activeTab) return;
          event?.preventDefault();
          void run(async () =>
            applySnapshot(await api.removeTab(activeTab.id)),
          );
          break;
        case "history":
        case "downloads":
          event?.preventDefault();
          openSection(shortcut);
          break;
        case "bookmark":
          if (!activeTab?.url.startsWith("http")) return;
          event?.preventDefault();
          void run(async () =>
            applySnapshot(
              await api.addBookmark(activeTab.title, activeTab.url),
            ),
          );
          break;
        case "find":
          event?.preventDefault();
          void run(api.find);
          break;
        case "zoom-in": {
          event?.preventDefault();
          const next = Math.min(5, zoom + 0.1);
          setZoom(next);
          void run(() => api.zoom(next));
          break;
        }
        case "zoom-out": {
          event?.preventDefault();
          const next = Math.max(0.25, zoom - 0.1);
          setZoom(next);
          void run(() => api.zoom(next));
          break;
        }
        case "zoom-reset":
          event?.preventDefault();
          setZoom(1);
          void run(() => api.zoom(1));
          break;
      }
    },
    [
      activeTab,
      applySnapshot,
      createTab,
      cycleTab,
      openSection,
      run,
      snapshot.data.recentlyClosed.length,
      snapshot.firstRun,
      snapshot.locked,
      zoom,
    ],
  );

  useEffect(() => {
    const handler = (event: globalThis.KeyboardEvent) => {
      const shortcut = browserShortcutFromKey(event);
      if (shortcut) handleShortcut(shortcut, event);
    };
    window.addEventListener("keydown", handler);
    const cleanup = listen<string>("browser-shortcut", (event) =>
      handleShortcut(event.payload),
    );
    return () => {
      window.removeEventListener("keydown", handler);
      void cleanup.then((dispose) => dispose());
    };
  }, [handleShortcut]);

  if (!ready) {
    return (
      <div className="flex h-full items-center justify-center gap-3 bg-background text-sm text-muted-foreground">
        <LoaderCircle className="size-4 animate-spin" />
        正在打开 QuickPane
      </div>
    );
  }

  const locked = snapshot.locked || snapshot.firstRun;
  const browsing =
    !locked && section === "newtab" && activeTab?.url !== "quickpane://newtab";

  return (
    <TooltipProvider>
      <main className="flex h-full flex-col bg-surface">
        {locked ? null : (
          <>
            <TabStrip
              tabs={snapshot.data.tabs}
              activeId={snapshot.data.activeTabId}
              onSelect={selectTab}
              onClose={(id) =>
                void run(async () => applySnapshot(await api.removeTab(id)))
              }
              recentlyClosed={snapshot.data.recentlyClosed}
              onRestoreClosed={(id) =>
                void run(async () =>
                  applySnapshot(await api.restoreClosedTab(id)),
                )
              }
              onNew={() => createTab()}
            />
            <NavigationBar
              activeTab={activeTab}
              address={address}
              onAddress={setAddress}
              onSubmit={submitAddress}
              addressRef={addressRef}
              suggestions={addressSuggestions}
              onSuggestion={selectSuggestion}
              onSuggestionsOpenChange={setAddressSuggestionsOpen}
              windowVisible={snapshot.windowVisible}
              bookmarked={Boolean(
                activeTab &&
                  snapshot.data.bookmarks.some(
                    (item) => item.url === activeTab.url,
                  ),
              )}
              onBookmark={() => {
                if (!activeTab?.url.startsWith("http")) return;
                void run(async () =>
                  applySnapshot(
                    await api.addBookmark(activeTab.title, activeTab.url),
                  ),
                );
              }}
              onBack={() => void run(api.back)}
              onForward={() => void run(api.forward)}
              onReload={() => void run(api.reload)}
              onHome={() => createTab(snapshot.data.settings.homeUrl)}
              onHide={() => void run(api.hide)}
              pinnedExtensions={snapshot.pinnedExtensions ?? []}
              onExtensionClick={(extension, anchor) => {
                if (!extension.popupUrl) return;
                void run(() =>
                  api.showExtensionPopup(
                    extension.popupUrl as string,
                    anchor.x,
                    anchor.y,
                  ),
                );
              }}
              onOpenMenu={openMenu}
            />
          </>
        )}

        <section
          className={`min-h-0 flex-1 ${locked || browsing ? "overflow-hidden" : "overflow-y-auto"}`}
        >
          <AnimatePresence mode="wait" initial={false}>
            {locked ? (
              <motion.div
                key="lock"
                className="h-full"
                variants={pageFade}
                initial="initial"
                animate="animate"
                exit="exit"
              >
                <LockScreen
                  snapshot={snapshot}
                  applySnapshot={applySnapshot}
                  run={run}
                />
              </motion.div>
            ) : browsing ? null : (
              <motion.div
                key={section}
                className="min-h-full"
                variants={pageFade}
                initial="initial"
                animate="animate"
                exit="exit"
              >
                {section === "newtab" &&
                activeTab?.url === "quickpane://newtab" ? (
                  <NewTabPage
                    snapshot={snapshot}
                    onNavigate={(url) =>
                      void run(async () =>
                        applySnapshot(await api.navigate(activeTab.id, url)),
                      )
                    }
                    onSection={openSection}
                    onUpdateQuickLinks={(quickLinks) =>
                      void run(async () =>
                        applySnapshot(
                          await api.updateSettings({
                            ...snapshot.data.settings,
                            quickLinks,
                          }),
                        ),
                      )
                    }
                  />
                ) : section === "history" ? (
                  <HistoryPage
                    snapshot={snapshot}
                    onOpen={createTab}
                    onClear={() =>
                      void run(async () =>
                        applySnapshot(await api.clearHistory()),
                      )
                    }
                  />
                ) : section === "bookmarks" ? (
                  <BookmarksPage
                    snapshot={snapshot}
                    onOpen={createTab}
                    onRemove={(id) =>
                      void run(async () =>
                        applySnapshot(await api.removeBookmark(id)),
                      )
                    }
                  />
                ) : section === "downloads" ? (
                  <DownloadsPage
                    snapshot={snapshot}
                    onOpen={(path) => void run(() => api.openDownload(path))}
                    onClear={() =>
                      void run(async () =>
                        applySnapshot(await api.clearDownloads()),
                      )
                    }
                  />
                ) : section === "extensions" ? (
                  <ExtensionsPage
                    run={run}
                    applySnapshot={applySnapshot}
                    pinnedIds={snapshot.data.settings.pinnedExtensions ?? []}
                    onOpen={(url) =>
                      void run(() => api.showExtensionPopup(url))
                    }
                  />
                ) : section === "settings" ? (
                  <SettingsPage
                    snapshot={snapshot}
                    applySnapshot={applySnapshot}
                    run={run}
                  />
                ) : null}
              </motion.div>
            )}
          </AnimatePresence>
        </section>

        <ErrorBanner error={error} onDismiss={() => setError(null)} />
      </main>
    </TooltipProvider>
  );
}

function setErrorFromUnknown(reason: unknown) {
  return reason instanceof Error ? reason.message : String(reason);
}

export default App;
