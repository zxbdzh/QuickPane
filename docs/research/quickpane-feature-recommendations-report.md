# QuickPane 功能建议：相关产品调研报告

## 1. 结论摘要

建议 QuickPane 优先建设一条“Windows 全局快捷键 + 键盘优先查阅”的产品路径，再逐步加入轻量工作区和多账号服务入口。具体优先级如下：

1. **P0：全局快捷启动与快速切换**：提供可配置的全局快捷键，呼出后直接搜索应用、网页、书签、标签和常用命令，并支持历史结果。
2. **P1：垂直标签页与轻量标签组织**：以垂直标签页降低标签管理成本，补充固定、分组、搜索和快速切换。
3. **P1：服务工作区与多账号登录**：允许按工作区组织站点，并支持同一服务的多个登录会话。
4. **P2：通知聚合、专注模式与资源控制**：集中查看未读信息，提供通知隐私控制和闲置页面休眠。
5. **P2：Windows/WebView2 运行可靠性**：围绕 Runtime 分发、版本策略和错误可见性完善工程能力。

上述结论主要来自产品官网的功能自述，**不包含独立的性能、稳定性或用户规模验证**。因此，以下产品能力应视为“竞品公开声称”，而不是实测结论。

## 2. 相关产品观察

### Sidekick：垂直标签页的直接参照

Sidekick 将自己定位为“Vertical tabs for Chrome”，说明垂直标签页可以作为浏览器的核心产品叙事，而不只是一个设置项（https://sidekick.io/）。它还提供 Chrome 扩展安装方式：用户需在 Chrome 中启用 Developer mode，再使用 “Load unpacked” 加载下载的文件夹（https://sidekick.io/）。

Sidekick 的桌面应用页面标注为“MacOS Apple Silicon Only”，且该页面未列出 Windows 版本或 Windows 全局快捷键（https://sidekick.io/）。这只能说明该页面的公开支持范围，**不能据此断言 Sidekick 完全不支持其他平台或不存在其他实现**。对 QuickPane 而言，较明确的机会是：在 Windows 上提供垂直标签页，并把它与全局快捷呼出结合起来。

### Wavebox：工作区、多账号和通知工作流

Wavebox 官方首页将其描述为桌面浏览器，并列出 macOS、Windows 和 Linux 支持（https://wavebox.io/）。它支持组织应用和标签页，并可让同一服务的多个账户保持登录，避免在不同浏览器或 Chrome 配置文件之间切换；官网举例包括多个 Microsoft Teams、Asana、Slack 或 ClickUp 账户（https://wavebox.io/）。

Wavebox 的公开功能还包括未读徽章和桌面通知、标签页分组、分屏/多窗口、Chrome Web Store 扩展、Profiles、Quick Switch、Keyboard Shortcuts 和 Focus Mode（https://wavebox.io/）。其 Mini menu 可将 Webmail、消息和协作工具集中到一个列表中，以便查看新内容（https://wavebox.io/）。

这些能力支持 QuickPane 增加“工作区/服务入口/快速切换”方向，但不建议一次复制完整桌面浏览器的全部复杂度。

### Ferdium：服务容器与本地隐私取舍

Ferdium 将自己描述为“all your services in one place”，并称其由社区构建（https://ferdium.org/）。它的 Services 功能可集中多个服务以便快速访问，并允许同一服务添加多次以登录多个账户（https://ferdium.org/）。

Ferdium 还列出 Workspaces、内置 Todo 面板、Custom Services、Anonymous Access、Cloud Sync、Save Resources 和 Privacy；其中 Anonymous Access 声称应用数据留在用户本地，Save Resources 通过休眠服务节省资源，Privacy 可隐藏通知具体信息（https://ferdium.org/）。

对 QuickPane 最有价值的借鉴是：服务入口应支持重复实例、工作区和通知隐私；资源休眠应作为可验证的工程功能，而不是仅仅隐藏窗口。

### PowerToys Run：Windows 快捷启动器的交互参照

Microsoft 将 PowerToys Run 定位为面向 Windows 高级用户的免费、开源、模块化快速启动器，可访问应用、文件、计算器功能、系统命令，并支持额外插件（https://learn.microsoft.com/en-us/windows/powertoys/run）。它默认使用 `Alt + Space` 呼出，快捷键可在设置中修改；PowerToys 必须在后台运行并启用 Run 功能（https://learn.microsoft.com/en-us/windows/powertoys/run）。

PowerToys Run 支持搜索应用、文件夹、文件和正在运行的进程，以及计算、系统命令、时间日期、单位换算、哈希计算、GUID 生成、打开网页和 Web 搜索（https://learn.microsoft.com/en-us/windows/powertoys/run）。它还包含 History 插件，默认直接激活命令为 `!!`，用于查找过去从已启用插件中选择过且匹配查询词的结果（https://learn.microsoft.com/en-us/windows/powertoys/run）。

其设置包括清除上次查询、结果排序、结果数量、主题、文本大小、多显示器位置和实验性的 Pinyin 选项（https://learn.microsoft.com/en-us/windows/powertoys/run）。因此，QuickPane 的快速入口可以优先采用“输入即搜、键盘完成、可回看历史”的模式，但不必扩展成通用系统命令执行器。

## 3. 推荐功能方案

### P0：全局快捷键快速入口

**建议内容：**

- 全局快捷键呼出 QuickPane 后，自动聚焦地址栏或统一搜索框。
- 统一搜索当前标签、历史、书签、快捷站点和可选服务入口。
- 支持 `Enter` 打开、方向键选择、`Esc` 隐藏，并保留最近选择结果。
- 快捷键允许用户修改，并提供冲突提示。

**依据与边界：** PowerToys Run 已公开验证“后台运行 + 可配置快捷键 + 快速搜索”的交互范式（https://learn.microsoft.com/en-us/windows/powertoys/run）。但现有材料没有提供 QuickPane 与 PowerToys Run 的延迟、资源占用或用户偏好对比数据，需通过 Windows 实测验证。

### P1：垂直标签页和有限的标签组织

**建议内容：**

- 增加可切换的垂直标签页布局。
- 支持固定标签、标签分组、标签搜索和最近关闭恢复。
- 用紧凑的键盘操作完成标签切换，避免引入无限层级的复杂工作区。

**依据与边界：** Sidekick 的公开定位直接围绕垂直标签页展开（https://sidekick.io/）。现有材料没有提供 Sidekick 垂直标签页的可用性数据，也没有证据证明垂直布局对所有用户都更高效，因此应先做可选布局并进行使用验证。

### P1：工作区、服务入口和多账号

**建议内容：**

- 提供少量、明确用途的工作区，例如“工作”“个人”“临时查阅”。
- 每个工作区可保存站点入口和标签集合。
- 同一服务允许添加多个实例，以支持多个账户。
- 增加 Quick Switch，在工作区、服务和标签之间快速跳转。

**依据与边界：** Wavebox 公开列出 Profiles、Quick Switch、标签组织和多账户保持登录能力（https://wavebox.io/）；Ferdium 也支持 Services 重复添加、Workspaces 和 Custom Services（https://ferdium.org/）。但这些页面没有说明其 cookie 隔离实现、性能成本或安全审计结果，QuickPane 应先明确会话隔离模型，再决定是否承诺“多账号同时登录”。

### P2：通知聚合、隐私和资源控制

**建议内容：**

- 为服务入口显示未读徽章，并提供可选桌面通知。
- 增加 Focus Mode，集中控制通知和干扰。
- 允许隐藏通知具体内容，仅显示来源或数量。
- 对闲置服务/标签进行可观测的休眠与恢复，并展示失败或恢复状态。

**依据与边界：** Wavebox 公开列出未读徽章、桌面通知和 Focus Mode（https://wavebox.io/）；Ferdium 声称可隐藏通知具体信息，并通过休眠服务节省资源（https://ferdium.org/）。目前没有独立数据证明这些机制在 QuickPane 的 WebView 场景下能达到特定节省比例，资源收益应以 CPU、内存和恢复耗时测试为准。

## 4. Windows 与 WebView2 实施约束

WebView2 可在本机应用中嵌入 HTML、CSS 和 JavaScript，使用 Microsoft Edge（Chromium）作为绘制引擎，并支持复用 Web 生态、访问本机功能、代码共享以及常青和固定版本分发（https://developer.microsoft.com/zh-cn/microsoft-edge/webview2/；https://learn.microsoft.com/zh-cn/microsoft-edge/webview2/）。这使其适合承载“Windows 原生窗口 + Web 浏览界面 + 全局快捷键”的混合应用路径。

WebView2 支持常青分发和固定版本分发：常青分发依赖定期平台更新和安全修补的最新版 Chromium；固定版本分发则将特定 Chromium 版本随应用打包，适合严格兼容性要求（https://developer.microsoft.com/zh-cn/microsoft-edge/webview2/；https://learn.microsoft.com/zh-cn/microsoft-edge/webview2/）。WebView2 Runtime 官方分发方式包括常青引导程序、可离线安装的常青独立安装程序和固定版本运行时，独立安装程序提供 x86、x64 和 ARM64 架构版本（https://developer.microsoft.com/zh-cn/microsoft-edge/webview2/）。

官方资料列出 WebView2 对 Windows 10/11 的若干客户端版本支持，也列出 Windows Server 2016、2019 和 2022 等支持版本（https://developer.microsoft.com/zh-cn/microsoft-edge/webview2/；https://learn.microsoft.com/zh-cn/microsoft-edge/webview2/）。支持的开发技术包括 Win32 C/C++、.NET Framework 4.6.2 或更高版本、.NET Core 3.1 或更高版本、.NET 5 或更高版本、WinUI 2.0 和 WinUI 3.0（https://learn.microsoft.com/zh-cn/microsoft-edge/webview2/）。

**实施建议：** QuickPane 应优先采用常青 Runtime，并保留固定版本作为严格兼容性问题的备选；同时在安装和启动阶段明确 Runtime 缺失、版本不兼容、页面恢复失败等状态。这里的优先级是工程建议，支持材料本身没有给出 QuickPane 的兼容性故障率或两种分发策略的成本数据。

## 5. 暂不建议新增的范围

在当前证据范围内，不建议将以下能力列为近期核心承诺：

- 完整的跨平台桌面浏览器：Wavebox 虽列出 macOS、Windows 和 Linux 支持（https://wavebox.io/），但这不能直接证明 QuickPane 跨平台的需求或投入回报。
- 云同步、团队协作和复杂企业工作区：Ferdium 列出 Cloud Sync（https://ferdium.org/），但材料没有提供同步冲突、安全性或团队需求证据。
- 通用系统命令、计算器和插件平台：PowerToys Run 已覆盖这些方向（https://learn.microsoft.com/en-us/windows/powertoys/run），QuickPane 若复制全部能力，产品边界会扩大；这是范围控制建议，不是对市场需求的否定。

## 6. 验证清单

1. 用 Windows 10/11 测量快捷键到可输入状态的延迟、标签切换延迟和恢复失败率。
2. 以同一服务的两个账户验证会话是否真正隔离，并测试退出、重启和工作区切换后的登录状态。
3. 用多个标签/服务测量休眠前后的 CPU、内存和恢复耗时；不要把“窗口隐藏”直接等同于“资源已释放”。
4. 验证通知隐私选项是否能避免泄露消息具体内容。
5. 记录 WebView2 Evergreen 与 Fixed Version 两种方案的安装成功率、更新影响和回滚路径。

## 7. 证据边界

本报告仅使用所给出的产品官网和 Microsoft 官方资料。证据主要证明产品公开定位、功能自述和 WebView2 的官方技术能力；**没有足够证据比较竞品的真实性能、稳定性、资源占用、用户规模、商业效果或安全性**。因此，建议先以 P0 的快捷入口和 P1 的标签/工作区原型进行可用性与 Windows 实测，再决定是否投入通知聚合、多账号隔离和资源休眠等较高复杂度功能。
