// WebView2 将每个 WebView 当成独立窗口；弹窗聚焦后仍需定位最近访问的网页。
(() => {
  const query = chrome.tabs.query.bind(chrome.tabs);
  const create = chrome.tabs.create.bind(chrome.tabs);
  const root = chrome.runtime.getURL('');
  const deliver = (promise, callback) => {
    if (!callback) return promise;
    promise.then(callback, error => { console.error('[QuickPane]', error); callback(); });
  };
  chrome.tabs.query = (options, callback) => {
    if (!options.active || !options.currentWindow) return query(options, callback);
    return deliver(query({}).then(tabs => {
      const pages = tabs.filter(tab => /^https?:/.test(tab.url));
      pages.sort((a, b) => b.lastAccessed - a.lastAccessed);
      return pages.length ? [pages[0]] : tabs.filter(tab => tab.active).slice(-1);
    }), callback);
  };
  // WebView2 的 tabs.create 可能创建空白页却不返回 Tab；在已有管理页继续该操作。
  chrome.tabs.create = (options, callback) => deliver((async () => {
    const tabs = await query({});
    const current = tabs.filter(tab => tab.url?.startsWith(root) && !tab.url.includes('_generated_'))
      .sort((a, b) => b.lastAccessed - a.lastAccessed)[0];
    if (current && options.url?.startsWith(root)) {
      await chrome.tabs.update(current.id, { url: options.url, active: true });
      return { ...current, url: options.url };
    }
    return create(options);
  })(), callback);
})();
