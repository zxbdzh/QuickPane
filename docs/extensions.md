# QuickPane 浏览器扩展与内置油猴

QuickPane 内置 Violentmonkey（暴力猴）2.49.0，提供油猴用户脚本安装、管理、启停与 GM API。首次启动自动安装并固定到导航栏；禁用、取消固定或卸载后不会在下次启动强制恢复。第三方扩展仍通过「扩展 → 添加扩展」选择含 `manifest.json` 的未打包文件夹，CRX 需自行解包。

## 加载与面板

- 网页标签、隐藏扩展宿主及扩展弹窗共享 `browser_data_dir` 与代理配置，React 壳使用独立数据目录。
- `extensions::load_profile` 串行调用 WebView2 `ICoreWebView2Profile7::AddBrowserExtension`，等待回调并记录实际 ID、加载结果。目录哈希仅用于本地安装标识，不作为打开页面的权威 ID。
- 没有网页时，隐藏的 `extension-host-*` 窗口也可初始化 Profile，让扩展页核对真实加载状态。
- 弹窗先创建 `about:blank`，确认扩展注册完成后才导航。否则首次加载 `chrome-extension://` 页面可能被 Edge 阻止或跳转到无效新标签页。
- 打开面板/设置只传本地扩展 ID，由后端校验已启用、加载成功并从 manifest 取得地址；拒绝跨源页面地址。
- 扩展页显示内置标记、版本、已加载/已停用/加载失败，以及「重新检查」「打开面板」「设置」。内置油猴提供「管理脚本」，在完整标签页打开控制台。

## 安装位置与启停

```text
%APPDATA%/com.quickpane.quickpane/
├── Extensions/<id>/manifest.json
├── ExtensionsDisabled/<id>.disabled
├── ExtensionsDisabled/<id>/...    # 兼容旧版本的禁用目录
├── violentmonkey-installed        # 首次内置安装标记
└── WebView2/                      # Cookie、扩展与用户脚本数据
```

启停调用原生 `BrowserExtension.Enable`；不移动扩展目录，防止无 key 扩展的路径 ID 变化和脚本数据关联丢失。安装/卸载/启停后重建页面 WebView，Cookie 保留。卸载调用原生 `Remove` 再清理本地文件。

## 暴力猴兼容与许可

资源位于 `src-tauri/resources/violentmonkey/`，通过 Tauri bundle resources 随包分发。上游 MIT 许可与来源说明同目录保留。官方 Tampermonkey 的再分发需要其作者书面授权，未将其打入安装包。

当前 WebView2 155 实测：MV2 兼容版可执行脚本及 GM 存储；MV3 的 `chrome.userScripts` 未开放，不能将扩展加载成功等同于脚本执行成功。

`quickpane-compat.js` 在上游后台脚本之前加载，适配两点：

1. WebView2 将每个 WebView 当作独立窗口，面板聚焦后查询当前页应定位最近使用的网页。
2. WebView2 `tabs.create` 对部分扩展内部操作不返回有效 Tab；在已有扩展页面中继续内部导航，保留安装确认与管理流程。

内置发行包已移除上游自带的 Google Drive OAuth 凭据，不提供 Google Drive 云同步；本地脚本能力不依赖这些凭据。

## 验证与限制

- `npm test`：包括兼容脚本选页、内部导航测试。
- `cargo test --lib`：包括扩展页面同源校验、禁用目录/ID 保持测试。
- 实际验收使用独立应用标识及数据目录，验证首次安装、打开面板、安装脚本、运行、GM 存储、启停和重启保留，不修改正式版数据。
- WebView2 仅支持浏览器扩展 API 子集；侧边栏、扩展商店、浏览器内部页不保证可用。兼容版依赖 Runtime 继续支持 MV2，升级 Runtime 或内置扩展版本时须重新实测。
- 应用锁不加密扩展目录或 WebView2 数据。用户脚本按匹配规则具有网页访问权限，应只安装可信脚本。
- 尚未实现扩展商店、云同步、自动升级内置扩展或通用 CRX 安装。
