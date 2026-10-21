# QuickPane 内置用户脚本管理器

- 上游：Violentmonkey 2.49.0（Manifest V2 兼容版）
- 来源：https://github.com/violentmonkey/violentmonkey/releases/tag/v2.49.0
- 原始发行文件：Violentmonkey-mv2-v2.49.0.crx
- 许可：MIT，见 LICENSE。

当前 WebView2 155 实测：MV2 可执行用户脚本及 GM 存储；MV3 的 chrome.userScripts 未开放。保留原扩展实现，不自行实现 GM API。

QuickPane 在 manifest.json 增加 quickpane-compat.js：让后台定位最近网页，并在现有扩展页面内完成内部导航，适配 WebView2 的窗口与 tabs.create 差异。该兼容脚本的自动检查在 scripts/extension-compat.test.mjs。

内置包移除了上游 Google Drive OAuth client_id/client_secret，因此不注册 Google Drive 云同步提供商；本地用户脚本、导入导出和 GM 存储不受影响。

升级时先在隔离应用标识下实测脚本安装、执行、禁用、重新启用、GM 存储和重启保留。切换到 MV3 前必须确认 userScripts API 在最低支持的 WebView2 Runtime 可用。
