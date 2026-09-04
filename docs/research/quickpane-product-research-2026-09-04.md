# QuickPane 产品功能调研

> 调研日期：2026-09-04
>
> 范围：Windows 10/11 上与“快捷呼出、键盘搜索、浏览会话和高密度标签管理”相关的产品。外部事实仅采用产品官网、官方文档或官方 GitHub；每项外部事实均附链接。成本为基于 QuickPane 当前架构的相对判断，不是外部事实。

## 1. 产品定位与核心功能对照

### 现状基线

README 将 QuickPane 定位为“按一下快捷键出现、再按一下隐藏”的轻量级 Windows 浏览器：后台运行、保留标签页和浏览会话、隐藏时静音媒体并恢复隐藏前的前台窗口。现有能力包括多标签页、历史、书签、最近关闭、下载、页面缩放、未打包扩展、应用锁、托盘、开机启动和签名自动更新。[QuickPane README](https://github.com/zxbdzh/QuickPane/blob/master/README.md)

| 产品 | 定位 | 呼出/搜索入口 | 内容与动作 | 标签/浏览组织 | 对 QuickPane 的启发 |
| --- | --- | --- | --- | --- | --- |
| **QuickPane** | 随叫随收的轻量浏览器，而非完整 Chrome/Edge 替代品 | 可配置全局快捷键显示/隐藏；地址栏支持 URL 或搜索 | 网页、历史、书签、下载；浏览器扩展 | 单窗口多标签；会话持久化；最近关闭 | 差异化应继续围绕“低打扰、立即恢复上下文”，不要复制完整浏览器生态。[README](https://github.com/zxbdzh/QuickPane/blob/master/README.md) |
| **Flow Launcher** | Windows 文件/应用启动器，强调社区插件和工作流 | 默认 `Alt+Space`，可配置；键盘驱动 | 应用、文件/文件内容、环境变量路径、URL/网页搜索、计算器、Shell、系统命令、设置；有预览和结果排序 | 可搜索浏览器书签；插件商店；支持自定义主题、便携模式和 Game Mode | “统一搜索 + 可扩展动作”成熟，但其核心仍是启动器，不应让 QuickPane 变成第二个系统启动器。[官方 GitHub README](https://github.com/Flow-Launcher/Flow.Launcher/blob/dev/README.md) |
| **PowerToys Run** | Microsoft PowerToys 中面向高级用户的快速启动器 | 默认 `Alt+Space`，可配置；可选全屏时不响应 | 应用/文件/进程、计算器、单位换算、时间日期、哈希/GUID、Shell、Windows 设置、服务、系统命令、网页搜索；插件可启停并设置直接激活关键字 | 有查询历史、结果排序、插件管理和多显示器位置 | 证明“统一搜索、直接关键字、结果上下文操作”有用户价值；插件边界和延迟设置也值得借鉴。[Microsoft Learn](https://learn.microsoft.com/en-us/windows/powertoys/run) |
| **Fluent Search** | Windows 全局搜索/窗口切换工具，避免离开当前工作流 | 全局搜索热键；可隐藏失焦、再次按热键隐藏、自动搜索最近复制文本 | Search Apps 聚合应用、文件、网页、浏览器书签/历史、打开窗口；支持窗口内搜索可交互元素、网页预览和多搜索引擎 | 浏览器搜索覆盖书签和历史；窗口/标签可作为可搜索对象 | 对 QuickPane 最相关的是“地址栏不只是导航框”：可搜索本地会话、书签和最近页面，并让用户直接跳转。[官网](https://fluentsearch.net/)、[Browser Search App](https://www.fluentsearch.net/docs/Search%20apps/Browser)、[Search options](https://www.fluentsearch.net/docs/Search/Search%20options) |
| **Arc** | 以 Spaces 和侧边栏组织不同生活/工作上下文的浏览器 | 侧边栏为主要导航；官方帮助也支持快捷键和命令式创建 Space | Space 有独立的 Pinned、Unpinned、主题和图标；适合将项目/生活领域分开 | 每个 Space 是独立浏览区域；侧栏承载固定和临时标签 | “上下文容器”比无限标签条更容易恢复任务；QuickPane 可先做轻量工作区，而不是复制 Arc 的完整 UI。[Arc Help：Spaces](https://resources.arc.net/hc/en-us/articles/19228064149143-Spaces-Distinct-Browsing-Areas) |
| **Vivaldi** | 高度可定制、面向重度浏览者的浏览器 | 快捷键、Quick Commands、可编辑工具栏 | 侧边 Web Panels；Windows and Tabs Panel；Tab Button 可搜索和恢复标签 | Workspaces 按主题隔离标签；Tab Stacks；Tab Tiling 可垂直、水平或网格并行查看 | “搜索全部标签 + 工作区 + 侧边常驻页面”是重度用户需求；应按成本逐步吸收，不追求 Vivaldi 的全量可定制。[Vivaldi Workspaces](https://help.vivaldi.com/desktop/tabs/workspaces/)、[Tab Button](https://help.vivaldi.com/desktop/tabs/tab-button/)、[Tab Tiling](https://help.vivaldi.com/desktop/tabs/tab-tiling/) |

### 关键模式

1. **快捷入口已是共同预期，但产品边界不同。** Flow Launcher 和 PowerToys Run 用全局快捷键打开搜索面板；QuickPane 已有同样的显示/隐藏基础能力。[Flow Launcher README](https://github.com/Flow-Launcher/Flow.Launcher/blob/dev/README.md) · [PowerToys Run](https://learn.microsoft.com/en-us/windows/powertoys/run)
2. **低成本高收益集中在“找到并恢复”。** PowerToys Run 提供历史插件，Vivaldi 的 Tab Button 可跨打开、最近关闭、同步标签搜索；Fluent Search 也把浏览器书签/历史纳入统一搜索。[PowerToys Run](https://learn.microsoft.com/en-us/windows/powertoys/run) · [Vivaldi Tab Button](https://help.vivaldi.com/desktop/tabs/tab-button/) · [Fluent Browser Search App](https://www.fluentsearch.net/docs/Search%20apps/Browser)
3. **组织方式从“列表”升级为“上下文”。** Arc 的 Spaces 和 Vivaldi 的 Workspaces 都把标签按工作/项目隔离；这直接对应 QuickPane “隐藏后稍后继续”的核心承诺。[Arc Spaces](https://resources.arc.net/hc/en-us/articles/19228064149143-Spaces-Distinct-Browsing-Areas) · [Vivaldi Workspaces](https://vivaldi.com/features/workspaces/)
4. **侧栏的价值是常驻入口，不是装饰。** Vivaldi Web Panels 将常用网站放入侧栏并支持移动版或桌面版视图；这适合聊天、文档、监控等短时高频页面。[Vivaldi Tab Management](https://vivaldi.com/features/tab-management/)

## 2. QuickPane 新增功能建议（按用户价值/实现成本排序）

评分：用户价值和实现成本均为 1–5；排序优先“高价值、低成本”，成本 1 最低。建议先做不改变 WebView2 核心边界的功能。

| 优先级 | 功能 | 用户价值 | 成本 | 为什么做 / 最小版本 |
| ---: | --- | ---: | ---: | --- |
| 1 | **统一快速切换面板** | 5 | 2 | 在现有地址栏或独立 `Ctrl+K` 面板中搜索当前标签、书签、历史、最近关闭；方向键选择、Enter 跳转/恢复。复用已有 snapshot、历史、书签和标签数据，避免先做全文索引。 |
| 2 | **标签工作区（Workspace）** | 5 | 3 | 允许创建/命名/切换少量工作区，每个工作区保存标签集合和激活标签；先不做跨设备同步。它直接增强 QuickPane 的“收起后恢复项目上下文”。 |
| 3 | **可配置侧边栏/固定页面** | 4 | 3 | 侧栏固定书签、工作区、常用页面；第一版只支持 QuickPane 内部页面和普通 URL，不做独立移动版渲染。常用页面可一键打开或聚焦。 |
| 4 | **标签休眠与会话级恢复** | 4 | 3 | 默认仅冻结长时间未使用的 WebView，恢复时保留 URL、标题和滚动前的可用状态；提供“一键恢复全部”。已有 `browser.rs` 已有冻结/激活边界，适合做可控增强。 |
| 5 | **地址栏动作关键字与上下文操作** | 4 | 2 | 例如 `b` 只搜书签、`h` 只搜历史、`t` 只搜标签、`!` 进入系统/应用动作；结果支持复制 URL、移入工作区、关闭标签。PowerToys Run 的直接激活命令验证了这种可发现的键盘入口。[官方文档](https://learn.microsoft.com/en-us/windows/powertoys/run) |
| 6 | **标签批量管理** | 4 | 3 | 按域名/标题筛选后批量关闭、移动工作区、静音、收藏；补充重复 URL 提示。先做显式用户操作，不做自动后台重排。Vivaldi 官方将标签搜索、重复标签处理和批量组织作为核心标签体验。[Tab Button](https://help.vivaldi.com/desktop/tabs/tab-button/) |
| 7 | **剪贴板即搜（可关闭）** | 3 | 2 | 呼出时读取最近一条文本剪贴板，若像 URL 则预填/导航，否则作为搜索建议；默认不上传内容，设置中明确关闭。Fluent Search 官方提供“打开时搜索最近复制文本”选项。[Search options](https://www.fluentsearch.net/docs/Search/Search%20options) |
| 8 | **基础双栏/分屏查看** | 3 | 4 | 仅支持两个标签并排、明确退出、保持各自 WebView；适合查文档时对照。不要一开始实现 Vivaldi 的任意网格、嵌套布局和跨窗口拖拽。[Vivaldi Tab Tiling](https://help.vivaldi.com/desktop/tabs/tab-tiling/) |
| 9 | **插件/动作扩展 API** | 3 | 5 | 允许受限本地插件贡献搜索结果和动作；需权限模型、版本兼容、崩溃隔离和安装来源。先完成内置关键字动作，再以真实需求驱动 API；Flow 和 PowerToys 的插件系统说明生态有价值，但也带来维护面。[Flow plugin manifest](https://github.com/Flow-Launcher/docs/blob/main/plugin.json.md) · [PowerToys plugin spec](https://github.com/microsoft/PowerToys/wiki/PowerToys-Run-Plugin-spec) |

### 推荐排序原则

- **先做 1–5：** 主要是现有状态的索引、筛选和 UI，能直接提高“按键呼出后马上继续工作”的成功率。
- **再做 6–8：** 解决标签规模和并行浏览，但会增加 WebView 生命周期或布局复杂度。
- **最后做 9：** 插件是生态项目，不是单一功能；没有明确用户案例前不应提前承担安全和兼容成本。

## 3. 明确不建议做的功能

1. **不做完整系统启动器。** 不建议加入应用/文件全文索引、Shell 管理员执行、服务启停、关机、注册表和大量系统命令。PowerToys Run 已在此领域成熟，QuickPane 的浏览器上下文会被稀释，且权限和安全审查成本高。[PowerToys Run 功能清单](https://learn.microsoft.com/en-us/windows/powertoys/run)
2. **不做跨浏览器数据抓取。** 不建议直接读取 Chrome/Edge/Firefox 的历史、Cookie、标签数据库。除隐私和文件锁定问题外，它会把 QuickPane 从自有会话产品变成浏览器数据迁移工具；优先搜索 QuickPane 自己的数据。
3. **不做首版云同步/账号体系。** 同步工作区、标签、扩展和设置会引入账号、加密、冲突合并、服务器和隐私政策；QuickPane 的本地即时性不依赖它。等本地工作区被实际使用后再评估。
4. **不做 Arc 式完整侧栏重构。** 侧栏固定入口值得做，但不建议复制 Space/收藏/临时标签/分屏等整套信息架构；这会破坏现有单窗口和地址栏心智模型。[Arc Spaces](https://resources.arc.net/hc/en-us/articles/19228064149143-Spaces-Distinct-Browsing-Areas)
5. **不做 Vivaldi 级“所有东西可配置”。** 工具栏任意编辑、鼠标手势、命令链、无限平铺和大量主题选项会扩大测试矩阵，直接损害轻量定位。只提供少量高频设置。[Vivaldi 7.6：可定制工具栏与命令](https://vivaldi.com/blog/vivaldi-on-desktop-7-6/)

## 4. 结论与后续 MVP

### 结论

QuickPane 不需要成为 Flow Launcher、PowerToys Run 或 Vivaldi 的总和。最有优势的空位是：**Windows 上一个可由全局快捷键唤起、默认不打扰、能立即恢复浏览上下文的轻量浏览器。** 竞品共同证明搜索、快捷动作和上下文组织有效；QuickPane 应把这些能力限定在浏览器会话内，从“呼出窗口”升级为“呼出并找到上次工作”。

### 后续 MVP（建议一个迭代完成）

1. **统一切换面板：** 当前标签 + 书签 + 历史 + 最近关闭，模糊匹配标题/URL，键盘导航，Enter 跳转或恢复。
2. **两个默认动作：** `Ctrl+K` 打开切换面板；`Esc` 关闭且不改变当前页面；动作入口可在设置中改键。
3. **最小工作区：** 新建、重命名、切换、删除；删除前明确提示关闭还是保留标签；每个工作区持久化标签 URL、标题和激活标签。
4. **可靠性验收：** 20 个标签下呼出后 300 ms 内显示已有本地结果（目标值）；隐藏/显示不丢标签；重启后工作区和最近关闭可恢复；应用锁开启时索引数据不绕过锁。
5. **暂不纳入：** 跨设备同步、外部浏览器导入、插件 SDK、任意分屏、系统命令和 AI 摘要。

MVP 验证重点不是功能数量，而是三个动作是否明显变快：**快捷键呼出 → 找到目标标签 → 继续工作**。完成后再根据真实使用数据决定侧栏、标签休眠或分屏的优先级。

## 来源清单

### 保留

- [QuickPane README](https://github.com/zxbdzh/QuickPane/blob/master/README.md) — 项目当前定位与已实现功能。
- [Flow Launcher 官方 GitHub README](https://github.com/Flow-Launcher/Flow.Launcher/blob/dev/README.md) — 官方功能、快捷键、插件、预览与主题。
- [Flow Launcher plugin manifest 文档](https://github.com/Flow-Launcher/docs/blob/main/plugin.json.md) — 官方插件扩展边界。
- [Microsoft Learn: PowerToys Run](https://learn.microsoft.com/en-us/windows/powertoys/run) — Microsoft 官方功能、设置、快捷键和插件机制。
- [PowerToys Run plugin spec](https://github.com/microsoft/PowerToys/wiki/PowerToys-Run-Plugin-spec) — 官方 GitHub 插件规范。
- [Fluent Search 官网](https://fluentsearch.net/) — 官方产品定位与 Search Apps 总览。
- [Fluent Search Browser Search App](https://www.fluentsearch.net/docs/Search%20apps/Browser) — 官方浏览器书签/历史搜索说明。
- [Fluent Search Search options](https://www.fluentsearch.net/docs/Search/Search%20options) — 官方搜索交互设置。
- [Arc Help: Spaces](https://resources.arc.net/hc/en-us/articles/19228064149143-Spaces-Distinct-Browsing-Areas) — Arc 官方 Space 信息架构。
- [Vivaldi Workspaces](https://help.vivaldi.com/desktop/tabs/workspaces/) — Vivaldi 官方工作区行为。
- [Vivaldi Tab Button](https://help.vivaldi.com/desktop/tabs/tab-button/) — 官方标签搜索、切换和恢复。
- [Vivaldi Tab Tiling](https://help.vivaldi.com/desktop/tabs/tab-tiling/) — 官方分屏能力与复杂度参考。
- [Vivaldi Tab Management](https://vivaldi.com/features/tab-management/) — 官方侧边栏、Web Panels 和标签管理总览。

### 舍弃

- 第三方插件商店、博客、论坛和搜索摘要未作为事实依据；它们不满足本任务“官方/第一方来源”限制。
- Arc 的营销页仅用于发现概念，功能断言优先引用 Arc Help；Vivaldi 营销页仅用于总览，具体行为优先引用 Vivaldi Help。

## 调研缺口

- 未有 QuickPane 用户量、标签数量分布、呼出延迟基线和工作区需求访谈数据，因此优先级是产品假设，应通过 MVP 埋点或用户访谈验证。
- 未评估 WebView2 多实例冻结/恢复在低内存 Windows 10 设备上的实测成本；实现标签休眠前应做 20/50/100 标签压力测试。
- 外部产品的版本持续变化；发布前应重新核对官方页面，尤其是 PowerToys Run 向 Command Palette v2 的迁移状态。[PowerToys Run](https://learn.microsoft.com/en-us/windows/powertoys/run)

## 5. Agent 产品与协议专题（官方资料核验）

### 5.1 已验证事实与产品建议的边界

- **已验证事实**：仅指下表和正文中由官方产品页、官方文档、W3C 规范或官方 GitHub 明确写出的能力；发布日期、地区和预览状态以链接页面为准。
- **产品建议**：关于 QuickPane 的价值、成本、风险、MVP 和权限，是基于当前仓库定位（Windows、Tauri 2、WebView2、全局快捷键、标签/历史/书签）作出的建议，不是外部产品事实。
- 不能把 Chrome 的云端 Gemini、Comet 或 Browser Use Cloud 的能力，误写成 QuickPane 已具备或无需模型/网络即可实现的能力。

### 5.2 浏览器内置/代理式 AI 产品对照

| 产品 | 官方已验证的公开能力 | 对 QuickPane 的可借鉴点 | 关键限制/风险 |
| --- | --- | --- | --- |
| **Perplexity Comet** | 官方页面称其提供统一 AI 搜索、即时上下文和跨站自动化；可点击、输入、提交和自动填表；还列出购物、会议安排、邮件跟进、研究、摘要、翻译，以及用 `@tab` 引用已打开标签。[Comet 官方产品页](https://www.perplexity.ai/comet/it) | 侧栏/面板作为当前标签的上下文入口；用显式标签引用而不是默认读取全部窗口；可复用的“技能”比自由提示更可控 | 这些能力依赖其 AI 服务和账户；购物结算、邮件、日历属于高影响外部动作。官方页面是能力宣传，不等于所有地区、版本和账户均可用，发布前需重新核对可用性。 |
| **Dia** | 官方入门页说明浏览上下文、聊天和历史用于回答；默认避免存储和处理银行、健康等敏感站点。[Dia Getting Started](https://www.diabrowser.com/getting-started) 官方安全公告说明其曾移除、重建 `fetch_web_content`，现在用 URL provenance 限制模型只能获取来自用户标签、粘贴、消息或明确提供文档的 URL，并在数据离开设备前拒绝攻击者构造的 URL。[Dia Security Bulletins](https://www.diabrowser.com/security/bulletins) 官方更新还说明 Skills 可保存并复用多步骤提示。[Dia 1.0.1](https://www.diabrowser.com/changelog/1-0-1) | “只处理用户明确给出的页面/标签”是 QuickPane Agent 的默认边界；技能应是可审计的固定模板，而不是无限制插件 | 上下文和历史本身是敏感数据；provenance 能降低外链数据外泄但不能消除提示注入。不要把“默认排除敏感站点”当成完整安全保证。 |
| **Microsoft Copilot/Edge** | Microsoft 说明 Edge 中的 Copilot 可在不离开页面的情况下提供 Microsoft 365 Copilot Chat；微软也公开了 Copilot Tasks：用户用自然语言描述任务，系统在后台通过自己的浏览器和应用执行，可一次性、定时或重复执行。[Edge for Business](https://enablement.microsoft.com/en-us/edge-for-business/) · [Copilot Tasks](https://www.microsoft.com/en-us/microsoft-copilot/blog/2026/02/26/copilot-tasks-from-answers-to-actions/) | 快速面板应支持“查看、草拟、建议”而不是直接发送；定时/后台 Agent 应延期到有审计和暂停机制后 | 邮件、日历、预约、联系商家等属于不可逆或高影响动作；后台运行会扩大凭据、隐私、通知和资源消耗面。 |
| **Google Gemini in Chrome** | Google 公开 Gemini in Chrome 可理解当前页和多标签上下文；Auto Browse 可执行多步网页任务，官方示例包括旅行研究、预约、表单、账单、报销和续证；敏感动作（购买、发帖等）需确认或交还用户。[Google Chrome auto browse](https://blog.google/products-and-platforms/products/chrome/gemini-3-auto-browse/) Google 还公开了可保存并一键运行、支持当前页和所选标签的 Skills。[Chrome Skills](https://blog.google/products-and-platforms/products/chrome/skills-in-chrome/) | “所选标签 + 可复用技能 + 每一步工作日志 + 随时暂停”是最值得移植的交互骨架 | Auto Browse 是预览/地区和订阅受限能力；模型仍会接触不可信页面内容，Google 自己也将安全描述为持续演进的防御体系。 |

### 5.3 自动化协议/项目对照

| 协议/项目 | 官方已验证的公开能力 | QuickPane 借鉴建议 | 不应直接照搬 |
| --- | --- | --- | --- |
| **Browser Use** | 官方文档称开源 Python 库可连接任意 LLM，在本地或自托管运行；Agent 负责复杂任务，Browser/Actor 提供导航、点击、填表、截图、提取等动作；官方还给出与 Playwright 通过 CDP 连接同一 Chrome 实例的示例。[Browser Use Open Source](https://docs.browser-use.com/open-source/introduction) · [Playwright Integration](https://docs.browser-use.com/open-source/examples/templates/playwright-integration) | 借鉴“规划层”和“确定性动作层”分离；本地模型可接入有限工具集；任务结果和步骤必须可见 | 不引入其云端浏览器、Cookie 注入、代理/CAPTCHA 绕过或通用任意站点 Agent。QuickPane 应先做本地只读任务。 |
| **Playwright MCP** | 官方 MCP 文档提供基于 accessibility snapshot 的结构化页面交互，工具包括 snapshot、click、type、form、tab 管理和代码执行；支持持久 profile、隔离 profile、CDP endpoint、localhost 绑定和 origin allow/block 配置。[Playwright Interaction](https://playwright.dev/mcp/tools/interaction) · [Profile & State](https://playwright.dev/mcp/configuration/user-profile) · [Playwright Configuration](https://github.com/microsoft/playwright-mcp) | accessibility snapshot 比截图坐标更适合可靠、可解释的“定位→确认→动作”；默认 localhost、隔离 profile 和 origin allowlist 可成为权限基线 | `browser_run_code`/任意 JavaScript、持久 Cookie 和 unrestricted file upload 都必须是显式高级权限，不能作为默认 Agent 工具。 |
| **Chrome DevTools Protocol（CDP）** | 官方协议提供 Chrome/Chromium 的调试与自动化域；tip-of-tree 版本能力完整但变化频繁且不保证向后兼容，stable 版本是较小子集。[CDP 官方文档](https://chromedevtools.github.io/devtools-protocol/tot/) | 在 QuickPane 内部，优先使用 WebView2/浏览器现有生命周期接口；若未来接外部浏览器，CDP 适合调试、网络、截图和性能诊断 | 不开放公网 CDP 端口；不把 CDP 当作安全隔离边界；不让模型直接获得全量 Runtime/Network/Storage 权限。 |
| **WebDriver BiDi** | W3C 规范定义双向 WebDriver 协议，允许浏览器向控制端流式发送事件，相比传统严格请求/响应更适合事件驱动的浏览器自动化；当前规范仍为 Working Draft。[W3C WebDriver BiDi](https://www.w3.org/TR/webdriver-bidi/) | 作为未来跨浏览器、事件化自动化的标准观察对象；设计 QuickPane Agent 事件总线时可采用“动作开始/等待/结果/失败”事件模型 | 目前不以 BiDi 作为 QuickPane 首版依赖；WebView2 集成和 Windows 专用目标优先于跨浏览器抽象。 |

## 6. QuickPane 最值得做的 Agent 功能

排序依据：价值高优先；成本和风险为相对 1–5 分，5 为最高。以下均为**产品建议**。

| 排名 | 功能 | 价值 | 成本 | 风险 | 首版范围 |
| ---: | --- | ---: | ---: | ---: | --- |
| 1 | **本地标签/历史/书签问答与定位** | 5 | 2 | 1 | 用户选择一个或多个标签，输入“找出关于 X 的页面”“比较这两个页面标题/价格字段”；只读提取标题、URL、可见文本摘要，结果带来源标签并可跳转。可先用本地规则/搜索，模型只是可选增强。 |
| 2 | **可复用本地技能（Skills）** | 5 | 2 | 2 | 内置 3–5 个模板：摘要当前页、提取表格、比较所选标签、生成待办草稿。技能是版本化 JSON/提示模板，显示输入范围和预计动作，不允许任意脚本。 |
| 3 | **Agent 工作日志与暂停/接管** | 5 | 2 | 1 | 每一步显示目标、来源标签、工具、结果；只提供 Stop、Pause、Take over。任何写入/提交前停住并要求确认。 |
| 4 | **本地页面操作建议（不自动提交）** | 4 | 3 | 2 | Agent 识别表单字段并生成“待填值预览”，用户逐项批准后填入；禁止自动点击提交、购买、发送、删除。先用 WebView2/DOM 可访问性信息，暂不接 CDP。 |
| 5 | **离线小模型摘要/分类** | 4 | 4 | 2 | 在设备有可用本地模型时处理用户明确选择的页面文本；无模型时退化到规则提取/关键词搜索；不上传历史、Cookie 或全文。模型适配器可后置。 |
| 6 | **外部自动化协议适配器** | 3 | 5 | 4 | 仅作为开发者实验开关，接本地 Playwright MCP/CDP 端点；默认关闭、localhost-only、临时会话和 origin allowlist。 |
| 7 | **后台定时/跨应用 Agent** | 2 | 5 | 5 | 不进入近期版本；只有在审计、凭据隔离、限额和用户验证成熟后再评估。 |

**结论**：QuickPane 的 Agent 差异化不是“替用户在互联网上工作”，而是“用快捷键立刻理解并恢复当前浏览上下文”。先做只读、局部、可暂停的 Agent，能覆盖最大价值且不破坏轻量浏览器定位。

## 7. 安全边界与权限设计（产品建议）

1. **上下文最小化**：默认只发送用户主动选择的当前页/标签；不读取全部标签、历史、Cookie、localStorage、下载目录或剪贴板。敏感 URL（银行、医疗、密码管理器、企业管理后台）默认拒绝进入 Agent 上下文，可由用户对单次任务明确解锁。
2. **读写分离**：工具分为 `read_page`、`list_tabs`、`extract_visible_text`、`draft_form_values`（低风险）和 `navigate`、`type`、`click`、`submit`、`download`、`run_script`（高风险）。高风险工具默认不存在；启用时逐项授权，不能用一个“允许 Agent”覆盖全部能力。
3. **来源与站点约束**：任务建立 readable origins 与 writable origins；模型不能因网页文字而自行新增站点。导航到新 origin 必须显示目标、原因和将暴露的数据，用户批准后才可继续。该设计直接借鉴 Chrome 对 Agent Origin Sets、敏感动作确认和提示注入防御的公开思路。[Google Agentic Security](https://blog.google/security/architecting-security-for-agentic/)
4. **确认点**：付款、购买、发帖、发邮件、发送消息、登录、改密码、删除数据、文件上传、下载可执行文件、提交表单必须二次确认；确认框展示最终 URL、字段/金额/收件人和即将发生的动作。密码由系统密码管理器或用户手动输入，模型不可读取密码。
5. **可见性与撤销**：面板显示步骤、来源、工具参数摘要和耗时；提供暂停、停止、接管；停止后不得继续后台动作。保留短期本地审计日志，应用锁启用时日志和上下文同样受锁保护。
6. **进程与网络隔离**：本地 Agent 进程使用最小权限账户；MCP/CDP 仅绑定 `127.0.0.1`，随机临时 token，任务结束即失效；网络默认 deny，按 origin allowlist 放行；不开放远程调试公网端口。敏感文本在发送给模型前做本地脱敏，明确提示“脱敏不是安全边界”。
7. **提示注入处理**：网页内容永远是“不可信数据”，不是系统指令。将页面文本与用户目标分栏传递；检测到“忽略之前指令、上传/转发秘密”等内容时只报告，不执行；不能承诺检测器能完全防住注入。

## 8. 不依赖云端 Agent 的最小 MVP（产品建议）

**目标**：不新增云端 Agent，不控制任意外部网站，不要求账号。约一周内可完成可运行原型，前提是复用现有标签、历史、书签和 WebView 状态。

1. **快捷入口**：沿用现有全局显示/隐藏；在地址栏增加 `Ctrl+K` Agent 面板，Esc 关闭。面板列出当前标签、最近关闭、书签和历史，全部本地检索。
2. **显式上下文**：用户勾选 1–3 个标签，QuickPane 读取标题、URL 和页面可见文本（默认上限，例如每页 20,000 字符）；不读取 Cookie、表单密码、跨站存储或未选择标签。
3. **三种本地任务**：`摘要`、`比较`、`提取链接/表格`。优先规则/DOM 提取；若设备已安装本地模型，再通过可插拔本地模型适配器生成自然语言结果。无模型时仍返回结构化结果，不显示“失败”。
4. **结果可追溯**：每段结果显示来源标签和 URL；点击来源激活标签。只生成复制到剪贴板的草稿，不自动写回网页、发送或保存到云端。
5. **最低验收**：20 个标签下本地面板打开目标 300 ms；用户未勾选的标签不会进入上下文；银行/医疗 URL 默认阻断；关闭面板立即终止任务；应用锁开启时无法通过 Agent 旁路读取索引；重启后不保存页面全文，只保留任务标题和来源 URL（若用户选择保存）。

**MVP 暂不需要**：Playwright MCP、CDP、WebDriver BiDi、插件 SDK、在线模型账号、后台调度、自动填写和任意 JavaScript。先证明“快捷键→选择标签→得到可追溯答案”是否比手工切换更快。

## 9. 明确不建议做的高风险能力

1. **无人值守购买、支付、转账、报税、医疗预约或账号变更**：后果不可逆且涉及金融/健康/身份；最多做到信息收集和用户确认前的草稿。
2. **自动读取并使用全部浏览器凭据**：Cookie、密码、session token、localStorage 和企业 SSO 一旦被页面注入或工具误用，影响是跨站的；不做“把现有登录态交给模型”的默认方案。
3. **任意网页 JavaScript、任意文件读写和 Shell 执行**：这会把 Agent 变成远程代码执行入口；不作为产品工具，开发调试也只能在隔离实验开关下使用。
4. **后台持续监控所有标签/历史/剪贴板并主动执行**：违反最小上下文原则，造成隐私泄露、资源消耗和难以解释的动作；QuickPane 的快捷呼出定位不需要常驻观察者。
5. **CAPTCHA/反自动化绕过、代理伪装和批量账号操作**：安全、合规和账户封禁风险高，也不是 QuickPane 的核心价值。

## 10. 专题结论

官方产品正在把 AI 从“回答当前页面”推进到“跨标签理解、可复用技能、分步操作和敏感动作确认”。对 QuickPane 最有价值、风险最低的抽象是：**显式选择上下文 + 本地优先只读处理 + 可复用技能 + 工作日志 + 随时暂停 + 写入前确认**。协议层面先采用内部窄接口；只有真实用户需求出现后，再评估 Playwright MCP/CDP，WebDriver BiDi 作为标准化长期观察项。

### 本章保留的官方来源

- [Perplexity Comet 官方产品页](https://www.perplexity.ai/comet/it) — Comet 的跨标签上下文、自动化、技能/助手能力说明。
- [Dia Getting Started](https://www.diabrowser.com/getting-started) — 上下文、历史和敏感站点默认处理说明。
- [Dia Security Bulletins](https://www.diabrowser.com/security/bulletins) — provenance 与提示注入导致数据外泄的安全案例。
- [Dia 1.0.1](https://www.diabrowser.com/changelog/1-0-1) — Skills 构建和复用说明。
- [Microsoft Edge for Business](https://enablement.microsoft.com/en-us/edge-for-business/) — Edge 中 Microsoft 365 Copilot Chat。
- [Microsoft Copilot Tasks](https://www.microsoft.com/en-us/microsoft-copilot/blog/2026/02/26/copilot-tasks-from-answers-to-actions/) — 后台、一次性和定时 Agent 任务。
- [Google Chrome auto browse](https://blog.google/products-and-platforms/products/chrome/gemini-3-auto-browse/) — Gemini 多标签上下文、Auto Browse、敏感动作确认。
- [Chrome Skills](https://blog.google/products-and-platforms/products/chrome/skills-in-chrome/) — 一键复用技能和确认机制。
- [Google Architecting Security for Agentic Capabilities](https://blog.google/security/architecting-security-for-agentic/) — origin gating、工作日志、接管、确认与提示注入防护。
- [Browser Use Open Source](https://docs.browser-use.com/open-source/introduction) — 本地/自托管 Agent 与浏览器动作分层。
- [Browser Use + Playwright Integration](https://docs.browser-use.com/open-source/examples/templates/playwright-integration) — 通过 CDP 共享浏览器实例。
- [Playwright MCP Interaction](https://playwright.dev/mcp/tools/interaction) — accessibility snapshot 和结构化动作。
- [Playwright MCP Profile & State](https://playwright.dev/mcp/configuration/user-profile) — profile、隔离、持久状态和本地路径。
- [Microsoft Playwright MCP](https://github.com/microsoft/playwright-mcp) — 官方 MCP 项目与配置入口。
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/tot/) — CDP 能力和稳定性边界。
- [W3C WebDriver BiDi](https://www.w3.org/TR/webdriver-bidi/) — 双向事件化 WebDriver 标准。

### 专题缺口

- Comet、Dia、Copilot 和 Gemini 的具体版本、地区、订阅和 Windows 可用性变化很快；本章只采信页面公开文字，不推断 QuickPane 可直接调用其内部服务。
- 尚未测量 WebView2 页面可见文本提取、20/50/100 标签并发下的内存和延迟；MVP 前应做本机基准。
- 尚未选择本地模型运行时；应以用户设备上已有模型/运行时为前提做适配，不因 Agent 研究先引入大型依赖。

## 11. 非 Agent 产品方向补充调研：组织、通知、保存与隐私

> 本章追加于 2026-09-04 产品调研。外部事实只采用产品官网、官方帮助文档、官方 GitHub/源码；QuickPane 的价值、成本和 MVP 判断属于产品建议。Agent 方向仅保留：**暂缓**。

### 11.1 已验证产品事实

#### A. 垂直标签、标签树、标签组与会话快照

1. **Vivaldi** 官方支持将标签栏放在左/右侧，并以 Accordion Tab Stacks 在垂直栏中压缩标签组；Windows and Tabs Panel 可按父子关系显示树状标签、窗口、工作区和标签堆。Tab Button 统一搜索打开标签、重复标签、非活跃标签和最近关闭标签，并可直接切换或恢复。[垂直标签与 Accordion](https://vivaldi.com/blog/tips/tip-833/) · [标签树](https://vivaldi.com/blog/tips/tip-820/) · [Tab Button](https://help.vivaldi.com/desktop/tabs/tab-button/)
2. **Vivaldi** Workspaces 按主题隔离标签；删除工作区会关闭其中标签，官方提示需先移动或从 Closed Tabs 恢复，因此工作区删除属于有破坏性的操作。[Workspaces](https://help.vivaldi.com/desktop/tabs/workspaces/) · [复用/删除工作区](https://vivaldi.com/blog/tips/tip-785/)
3. **Chrome** 官方公开了可选的垂直标签布局，目标是显示完整标题并继续管理标签组；同时提供沉浸式阅读模式，将页面变为更少干扰的文本视图。[Chrome 垂直标签与阅读模式](https://blog.google/products-and-platforms/products/chrome/new-chrome-productivity-features/)
4. **Arc** 官方帮助将 Spaces 定义为独立浏览区域，侧栏承载 Pinned 与 Unpinned 标签；其核心启发是“按上下文恢复”，不是单纯增加标签栏功能。[Arc Spaces](https://resources.arc.net/hc/en-us/articles/19228064149143-Spaces-Distinct-Browsing-Areas)
5. **Wavebox** 的 Dashboard Collections 可把当前标签集保存为可重新打开的集合；其官方说明强调这解决“关闭后下周取回”的场景。Wavebox Snapshots 则是约每 24 小时一次的云端 Profile 备份，可恢复 groups、apps、tabs、bookmarks、settings 等。[层级与 Collections](https://hub.wavebox.io/the-wavebox-hierarchy-what-sits-inside-what/) · [Profile Snapshots](https://hub.wavebox.io/snapshots/)

**判断：** “树状标签”解决结构浏览，“工作区”解决上下文切换，“快照/集合”解决跨时间恢复。三者不是同一功能；QuickPane 首版应先做工作区 + 命名会话快照，不做递归树编辑器。

#### B. 工作区、服务容器和多账号隔离

1. **Wavebox Spaces**（旧称 cookie containers/profiles）为每个 Space 保持独立登录，可同时使用多个同站账号；Wavebox Profiles 是更强边界，各自拥有 spaces、settings、extensions、passwords 等，互不共享。[Wavebox Discover](https://hub.wavebox.io/discover/) · [Spaces 与 Profiles](https://hub.wavebox.io/spaces-vs-profiles-which-one-should-you-choose/)
2. **Rambox** 官方产品页提供 Workspaces、Profiles、多账号同时登录、按 app/workspace 控制通知，以及暂停非活跃 app 以节省内存和 CPU。[Rambox Features](https://rambox.app/features/) 官方支持文档还提供 Focus Mode 徽章控制和应用消耗监视器设置。[General Settings](https://support.rambox.app/support/solutions/articles/42000027666-general-settings)
3. **Ferdium** 官方 GitHub 仓库可确认其以 services/workspaces 聚合网页服务；但官方 issue 记录显示 Windows/桌面通知、跨 workspace QuickSwitch 等行为仍有兼容或体验问题。因此它适合作为“聚合复杂度和可靠性风险”的反例，不应把 issue 中的需求当成已交付能力。[Ferdium 源码仓库](https://github.com/ferdium/ferdium-app) · [通知问题](https://github.com/ferdium/ferdium-app/issues/2377) · [跨 workspace QuickSwitch 问题](https://github.com/ferdium/ferdium-app/issues/2368)

**判断：** 多账号容器的关键不是 UI 分组，而是 cookie/storage/profile 的隔离边界。WebView2 可实现独立 user data folders，但这会增加磁盘、登录、扩展、锁和恢复测试成本；QuickPane 不应把通用“服务容器”当近期方向。

#### C. 通知聚合、免打扰和专注

1. **Wavebox** 的 Wavebox Mini 将所有未读项目和通知汇总到一个列表；Focus Mode 会关闭通知、隐藏未读徽章并静音。[Wavebox Discover](https://hub.wavebox.io/discover/)
2. **Rambox** 支持按 app 或 workspace 控制提醒，并提供 Focus Mode 暂停通知、隐藏徽章；其官方材料还描述了按工作时段控制通知和 Focus Session 定时结束。[Rambox Features](https://rambox.app/features/) · [专注与工作时段](https://rambox.app/blog/context-switching/)
3. **Chrome** 官方宣布桌面端会对低互动、高通知量网站自动撤销通知权限，用户可在 Safety Check 中查看并重新授权，也可关闭该自动撤销功能。[Chrome 通知权限治理](https://blog.google/chromium/automatic-notification-permission/)

**判断：** QuickPane 的低成本版本不是重新实现消息中心，而是“隐藏窗口时静音 + 一键专注模式 + 站点通知总开关/按站点例外”。WebView2 页面通知与 Windows toast 的聚合、点击回跳需先实测，不应承诺所有网站一致。

#### D. 稍后阅读、离线保存与网页剪藏

1. Chrome 官方阅读模式提供即时的低干扰文本阅读，但这不是离线存档或网页剪藏。[Chrome 阅读模式](https://blog.google/products-and-platforms/products/chrome/new-chrome-productivity-features/)
2. Wavebox 官方的 **Saved Items** 保存链接而不保持页面加载；其 Dashboard Collections 可保存一组链接/标签作为未来打开的集合。[Wavebox 层级说明](https://hub.wavebox.io/the-wavebox-hierarchy-what-sits-inside-what/)
3. 本轮未找到 Ferdium、Rambox、Arc 的第一方文档，能明确证明其提供通用离线网页保存/网页剪藏的完整行为；因此不将第三方扩展、博客或源码外部项目的描述写成产品事实。

**判断：** “稍后打开”只需 URL + 标题 + favicon/来源时间，成本低；“离线保存”涉及 HTML 清洗、资源抓取、版权/隐私、动态页面和磁盘膨胀，成本显著更高。QuickPane 先做本地阅读清单，不做离线网页副本。

#### E. 隐私控制、站点权限和资源休眠

1. Microsoft Edge 官方策略文档列出了按 URL 控制 cookies、通知、地理位置、弹窗、JavaScript、文件系统读写、WebUSB/WebHID 等权限；也列出了 Vertical Tabs 与 Sleeping Tabs 配置，包括启用、超时和 URL 排除。[Edge 策略文档](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies)
2. Edge 的官方策略表明 Sleeping Tabs 可按不活跃超时释放资源，并支持对特定 URL 禁用；这验证了“休眠必须有可见的例外列表”，不能静默处理所有页面。[Sleeping Tabs 策略](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies#sleeping-tabs-settings)
3. Rambox 官方功能页也声明可暂停非活跃 app 以节省内存/CPU；Wavebox 官方资料说明未使用内容可休眠、需要时唤醒。[Rambox Features](https://rambox.app/features/) · [Wavebox 层级说明](https://hub.wavebox.io/the-wavebox-hierarchy-what-sits-inside-what/)

**判断：** WebView2 适合先暴露“按站点权限查看/撤销”和“标签休眠白名单”。不要复制 Edge 企业策略系统；用户产品只需少量高价值权限和清晰恢复操作。

### 11.2 QuickPane 功能机会：按价值/成本排序

成本为相对判断（1 最低，5 最高），价值为用户收益（1 最低，5 最高）。

| 优先级 | 机会 | 价值 | 成本 | 建议最小范围 |
| ---: | --- | ---: | ---: | --- |
| 1 | **统一快速切换 + 最近关闭/命名快照** | 5 | 2 | 在现有地址栏/`Ctrl+K` 中搜索标签、书签、历史、最近关闭和用户保存的会话；一键恢复整组 URL，显示数量和上次保存时间。 |
| 2 | **轻量工作区** | 5 | 3 | 3–8 个命名工作区；保存标签 URL、标题、激活标签和顺序；切换不强制关闭其他工作区；删除前选择“仅移除工作区”或“关闭标签”。 |
| 3 | **专注模式** | 4 | 2 | 一键关闭 WebView 通知提示、静音媒体、隐藏未读/活动标记；退出后不丢失通知状态。可选 25/50 分钟定时，不做后台调度。 |
| 4 | **标签休眠/唤醒** | 4 | 3 | 对未激活且无音频/下载/表单输入的标签冻结；按站点或标签提供“不休眠”；恢复失败时保留 URL 并显示重载。复用现有 `browser.rs` 冻结边界。 |
| 5 | **稍后阅读清单** | 4 | 2 | 保存 URL、标题、favicon、加入时间、标签/工作区；本地搜索、标记已读、批量打开/删除；不抓取正文。 |
| 6 | **站点权限面板** | 4 | 3 | 当前站点的通知、音频、位置、摄像头/麦克风、弹窗状态一览；允许用户撤销或恢复，敏感权限不做自动授权。 |
| 7 | **垂直标签视图** | 3 | 3 | 作为可选布局显示标签标题、工作区和折叠的一级分组；不做无限树、拖拽跨窗和任意布局。 |
| 8 | **多账号容器/Profile** | 3 | 5 | 只有真实用户提出跨账号需求后，才以独立 WebView2 user data folder 做实验；先不与工作区混为一体。 |
| 9 | **离线网页剪藏** | 3 | 5 | 暂不实现；若稍后阅读数据证明需求强，再做“用户明确点击保存、可搜索纯文本”的窄版本。 |

### 11.3 适合 Windows + WebView2 的 MVP

**推荐一个最小、可验收的组合：统一切换、轻量工作区、专注模式、稍后阅读。** 不改 WebView2 核心导航模型，不新增账号或云服务。

1. `Ctrl+K` 打开本地切换面板：标签、书签、历史、最近关闭、已保存会话和稍后阅读统一搜索；Enter 聚焦/恢复，Esc 关闭。
2. 工作区只保存 QuickPane 自己的元数据：名称、颜色、标签 ID/URL、顺序、激活标签；最多允许用户创建 8 个，避免无限组织层级。
3. “保存会话”保存 URL/标题/标签关系，不复制页面内容；恢复按需创建 WebView，失败时提供重试和移除失效项。
4. 专注模式沿用已有隐藏时静音逻辑，额外暂停页面通知提示并隐藏界面徽章；状态持久化，退出后恢复原设置。
5. 稍后阅读只保存 URL/标题/favicon/时间/已读状态；数据留在本地应用目录并受应用锁保护。

**MVP 验收目标：** 20 个标签下切换面板从快捷键到显示本地结果 ≤300 ms；保存/恢复 30 个 URL 不丢顺序；专注模式不产生页面通知且退出后可恢复；稍后阅读重启后可见；应用锁开启时不能通过切换面板绕过锁读取受保护元数据。

### 11.4 明确不建议做的方向

1. **完整标签树/无限嵌套和 Vivaldi 级布局编辑器**：结构维护成本高，不能直接证明比搜索/工作区更快。
2. **Wavebox/Ferdium/Rambox 式服务聚合与多账号容器**：它要求独立 cookie/storage、通知适配、登录恢复和大量站点兼容；会把轻量浏览器变成 Electron 式工作台。
3. **云端会话快照、账号同步和团队工作区**：需要加密、冲突合并、账号体系和服务器；QuickPane 的核心价值是本地即时恢复。
4. **通用离线网页镜像/剪藏**：动态网页、图片/字体资源、脚本、版权和磁盘配额会形成长期维护面；先保存链接。
5. **自动管理通知权限或自动休眠所有标签**：站点、音频、下载、表单和 WebSocket 行为差异大；只提供可解释的用户开关、白名单和撤销。

### 11.5 需要用户实测的数据

1. **标签规模与恢复行为**：实际用户常态标签数（P50/P90/P99）、工作区数量、一次会话恢复标签数，以及恢复后仍需要的页面比例。
2. **性能/内存**：WebView2 在 20/50/100 标签下的窗口显示延迟、切换延迟、冷启动时间、工作集内存；冻结后恢复耗时和失败率。
3. **专注模式需求**：用户希望暂停的是网页通知、声音、QuickPane 徽章还是 Windows toast；典型专注时长和退出后是否需要通知汇总。
4. **稍后阅读真实需求**：每周保存链接数量、打开率、重复 URL 比例、是否需要正文搜索；若打开率低于保存量，应停止扩展为离线剪藏。
5. **权限与隐私边界**：哪些站点被用户加入“不休眠/不通知/始终允许”白名单；应用锁开启时用户是否接受仅保存 URL，还是要求标题/favicon 也加密。

### 11.6 本章来源与取舍

**保留：**

- [QuickPane README](https://github.com/zxbdzh/QuickPane/blob/master/README.md) — 当前 Windows、WebView2、快捷呼出、会话、历史、书签、锁与静音基线。
- [Vivaldi Tab Button](https://help.vivaldi.com/desktop/tabs/tab-button/) — 官方标签统一搜索、切换和恢复事实。
- [Vivaldi vertical/Accordion tabs](https://vivaldi.com/blog/tips/tip-833/) — 官方垂直标签与标签堆事实。
- [Vivaldi tab tree](https://vivaldi.com/blog/tips/tip-820/) — 官方树状关系视图事实。
- [Vivaldi Workspaces](https://help.vivaldi.com/desktop/tabs/workspaces/) — 官方工作区行为。
- [Arc Spaces](https://resources.arc.net/hc/en-us/articles/19228064149143-Spaces-Distinct-Browsing-Areas) — 官方 Space 信息架构。
- [Chrome vertical tabs and reading mode](https://blog.google/products-and-platforms/products/chrome/new-chrome-productivity-features/) — 官方垂直标签与阅读模式事实。
- [Wavebox Discover](https://hub.wavebox.io/discover/) — 官方 Spaces、通知汇总、Focus Mode、Saved Items 等能力。
- [Wavebox Spaces vs Profiles](https://hub.wavebox.io/spaces-vs-profiles-which-one-should-you-choose/) — 官方 cookie/profile 隔离边界。
- [Wavebox hierarchy](https://hub.wavebox.io/the-wavebox-hierarchy-what-sits-inside-what/) — 官方 Saved Items 与 Collections 行为。
- [Wavebox Snapshots](https://hub.wavebox.io/snapshots/) — 官方云端 Profile 快照事实。
- [Rambox Features](https://rambox.app/features/) — 官方 Workspaces、Profiles、通知、Focus Mode、休眠事实。
- [Rambox General Settings](https://support.rambox.app/support/solutions/articles/42000027666-general-settings) — 官方徽章和应用设置。
- [Ferdium GitHub](https://github.com/ferdium/ferdium-app) — 官方源码仓库；仅用于确认项目范围，不把 issue 需求当作已交付能力。
- [Edge policies](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies) — 官方按站点权限、Vertical Tabs、Sleeping Tabs 配置边界。
- [Chrome notification permission](https://blog.google/chromium/automatic-notification-permission/) — 官方通知权限自动治理事实。

**舍弃：** 第三方 Arc 扩展、第三方会话保存扩展、论坛/社区帖子、搜索摘要和无官方文档支撑的营销推断；它们不满足本次第一方来源限制。

### 11.7 缺口

- 本轮未验证 Arc 在 Windows 的完整功能可用性，也未把其营销页能力外推到当前版本。
- Wavebox/Rambox 的“云同步、通知聚合、休眠”具体订阅/版本限制需在 Windows 实机和对应计划下核对。
- WebView2 对网页 Notification、service worker、音频、下载和表单输入的冻结/恢复行为没有本机数据；MVP 前必须做上述 20/50/100 标签矩阵。
- 尚未做用户访谈和匿名埋点；以上排序仍是基于 QuickPane 定位的产品假设。
