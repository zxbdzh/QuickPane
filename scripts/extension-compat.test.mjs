import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';
import test from 'node:test';

const code = await readFile(new URL('../src-tauri/resources/violentmonkey/quickpane-compat.js', import.meta.url), 'utf8');

test('油猴弹窗定位最近网页，内部管理操作复用已有页面', async () => {
  const root = 'chrome-extension://builtin/';
  const tabs = [
    { id: 1, url: 'https://older.test/', lastAccessed: 1 },
    { id: 2, url: 'https://current.test/', lastAccessed: 3 },
    { id: 3, url: root + 'popup/index.html', active: true, lastAccessed: 4 },
  ];
  const updates = [];
  const chrome = {
    runtime: { getURL: () => root },
    tabs: {
      query: async () => tabs,
      create: async options => ({ id: 9, ...options }),
      update: async (id, options) => { updates.push({ id, ...options }); },
    },
  };
  vm.runInNewContext(code, { chrome, console });
  const active = await chrome.tabs.query({ active: true, currentWindow: true });
  assert.equal(active[0].id, 2);
  const created = await chrome.tabs.create({ url: root + 'confirm/index.html#123' });
  assert.equal(created.id, 3);
  assert.equal(updates[0].url, root + 'confirm/index.html#123');
  assert.equal((await chrome.tabs.create({ url: 'https://external.test/' })).id, 9);
});

test("内置包不携带上游 Google OAuth 凭据", async () => {
  const background = await readFile(new URL("../src-tauri/resources/violentmonkey/background/index.js", import.meta.url), "utf8");
  assert.doesNotMatch(background, /[\w.-]+\.apps\.googleusercontent\.com|GOCSPX-[\w-]+/);
  assert.match(background, /client_id:"",client_secret:""/);
});
