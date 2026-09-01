# QuickPane 中文地址建议：拼音依赖调研

> 核查时间：2026-09-01。范围为 React 19 + Vite + Tauri 2/WebView2、离线运行、本地隐私和当前同步纯函数建议链路。npm 的版本与体积数据来自锁定版本的 registry 查询和 `npm pack --dry-run`；包 README 中的体积数据不等同于 QuickPane 最终 bundle。

## 结论

推荐顺序：**`pinyin-pro` > `cnchar` > `pinyin`**。已完成小型 spike：锁定 `pinyin-pro@3.29.3` 后，Vite 产物增量和本地输入延迟均在当前可接受范围内，因此按最小纯函数适配接入。

`pinyin-pro` 最贴合本需求：有现成的 `match()`，同时支持全拼、首字母和混合匹配，提供 ESM 和 TypeScript 声明，MIT 许可证，无需远程服务。已用发布包实测：

```text
match("便宜", "py")      -> [0, 1]
match("便宜", "bianyi") -> [0, 1]
```

它不是完整的地址栏排序器，仍需由 QuickPane 维护现有的原生中文/英文/URL 匹配、来源优先级、去重和最多 8 条规则。

## 当前代码适配

- `getAddressSuggestions()` 是同步纯函数，数据来自快捷站点、书签和历史记录；当前没有拼音依赖或网络请求。
- 最小接入点是候选匹配层：保留现有 URL 解析、标题/主机名/URL 排名和规范化 URL 去重，在标题上增加拼音匹配分值。
- 拼音命中应低于原生标题、主机名和 URL 命中，避免中文直输行为改变。展示内容仍使用原始标题和 URL，不把拼音写入持久化数据。
- 只接受 ASCII 字母拼音查询时才启用拼音路径；查询侧统一小写，并明确处理空格。输入、历史、书签和标题全部留在本地 WebView，不调用远程建议服务。

## 候选对比

| 包 | 当前版本 | 官方 API / 能力 | 浏览器与类型 | 发布包线索 | 判断 |
| --- | --- | --- | --- | --- | --- |
| `pinyin-pro` | 3.29.3 | `match()` 支持全拼、首字母、混合匹配；`pinyin()` 支持无声调和逐字数组 | ESM、内置 typings；README 提供浏览器 script 用法 | npm pack 320.7 KB，解包 944.4 KB；README 的 `match` 独立 ESM 线索为 185.67 KB（gzip 80.90 KB） | 首选，先实测最终 bundle |
| `cnchar` | 3.2.6 | `spell(text, 'first', 'low')` 首字母；`spell(..., 'poly')` 多音候选；可配 `cnchar-poly` | 浏览器/webpack 可用；单体 `cnchar.min.js`，有 typings，无 ESM 字段 | npm pack 72.8 KB，解包 152.1 KB；MIT；最后发布 2024-03-23 | 体积小，维护和 tree-shaking 较弱；可作备选 |
| `pinyin`（hotoo） | 4.0.0 | `style: 'normal'`、`'first_letter'`、`heteronym`、`segment` | Node/Web；ESM、CJS、类型声明 | npm pack 约 13.3 MB，解包约 61.9 MB；依赖 `commander`；MIT | API 可用，但不符合轻量包体 |
| `tiny-pinyin` | 1.3.2 | `convertToPinyin()`；无现成首字母匹配 API | 有 typings 和浏览器构建 | npm pack 13.4 KB，解包 42.2 KB；最后发布 2021-01-10；README 明确不支持多音字 | 只适合极端体积优先，不推荐本项目 |
| `parse-pinyin` | 1.3.5 | `toPinyin()`、`toHanzi()`、`parse()`；README 未给出首字母 API | ESM/MJS、types，声明支持浏览器/Vite | npm pack 200.0 KB，解包 708.6 KB；MIT；2025-10-13 发布 | 可观察的新包，但需要自己补首字母/排序适配 |
| `pinyin-match-hanzi` | 0.1.1 | 主要是拼音转汉字候选和分词 | ESM/CJS/types；依赖 `dawg-lookup`、`lru-cache` | 解包约 6.8 MB；MIT | 方向是“拼音到汉字”，不是“汉字标题被检索” |

## 事实与限制

### `pinyin-pro`

官方 README 明确给出：

- `match("中文拼音", "zwp")` 首字母匹配。
- `match("中文拼音", "zhongwenpin")` 全拼匹配。
- `pinyin("汉语拼音", { toneType: "none" })` 返回无声调全拼，`type: "array"` 返回逐字结果。
- README 声明自动识别多音字，但孤立字、短标题、品牌名和专名仍不能保证语境正确。
- ESM API 有独立体积表，静态导入适合交给 Vite 做 tree-shaking；最终大小必须以 QuickPane 的 `npm run build` 为准。

`match()` 未命中时返回 `null`，命中时返回字符位置数组；因此适配代码不能假设返回空数组，也不应把全拼和首字母粗暴拼成一个字段，否则可能产生跨字段误命中。

### `cnchar`

官方 README 给出 `spell('测试', 'first', 'low')` 返回 `cs`，并提供 `poly` 参数和可选的 `cnchar-poly` 多音词库。它的发布包很小，但 `main` 指向单体压缩脚本，没有 `module` 字段；实际 Vite 产物和 CommonJS/UMD 兼容性需要单独验证。npm registry 显示最近发布早于 `pinyin-pro`，应把维护风险纳入取舍。

### `pinyin`（hotoo）

官方 README 支持 `first_letter` 和多音字/分词选项，但同时说明 Web 版只有常用字库、没有分词，Node 版字库和多音字结果更完整。其 4.0.0 发布包包含约 7.4 MB 的 ESM/CJS 主文件和大量 source map，和 QuickPane 的轻量地址栏目标不相称。

### `tiny-pinyin`

它并非不存在：npm registry 可核验 1.3.2、MIT、浏览器构建和 typings。问题是官方 README 明确“不支持多音字”，还记载 IE/Edge 测试未通过；最后一次发布在 2021 年。它不能直接满足首字母检索和中文语境准确性要求。

## 推荐接入策略

1. 先只做 `pinyin-pro` spike，静态导入 `match`，不引入网络、CDN、React 组件或 Rust 命令。
2. 为每个候选标题在一次建议计算中生成或缓存拼音索引；不要在每次比较时重复转换同一标题。
3. 将拼音命中排在原生标题/主机名/URL 命中之后，再沿用现有来源优先级、历史时间排序、URL 去重和 8 条上限。
4. 查询规范化至少覆盖 `py`、`bianyi`、`PY`、带空格的 `bian yi`、中文直输、英文/数字/URL 混合输入；不要改变展示文本。
5. 在 100、1,000、10,000 条候选上测冷启动、连续 20 次输入、p50/p95 响应时间和 Vite gzip/未压缩增量。

## 必测中文边界

- 多音字：`重庆`、`银行`、`长大`、`音乐`、`行长`、`重阳`，以及孤立的 `行`、`长`、`重`。
- 混合标题：`GitHub中文`、`中文GitHub`、`Type-C 中文`、数字、emoji 和标点。
- 拼音输入：`py`、`bianyi`、`by`、大小写、空格和全角空格。
- 现有契约：同 URL 的快捷站点/书签/历史仍只保留一条；坏 URL 和无效时间不影响结果；输入过程中无 XHR/fetch、剪贴板或远程请求。
- WebView2：Windows 10/11 发布构建确认 ESM 解析、无 Node 内置模块依赖、无动态网络加载。

## 最终决策

**已接入 `pinyin-pro@3.29.3`。** spike 结果满足当前门槛：10,000 条本地候选建议计算 p50 26.91ms、p95 33.08ms，生产 JS 相对接入前增加约 186 KB minified、约 85 KB gzip。实现保持单一静态依赖和纯本地匹配；后续若候选规模或启动性能目标改变，再评估索引预计算或按需加载。

## 来源

- [pinyin-pro npm](https://www.npmjs.com/package/pinyin-pro) — 版本、MIT、README 和包入口。
- [pinyin-pro GitHub](https://github.com/zh-lx/pinyin-pro) — 官方源码、`match`/`pinyin` API 和构建体积表。
- [pinyin-pro documentation](https://pinyin-pro.cn/use/pinyin.html) — 官方 API 文档。
- [pinyin-pro registry](https://registry.npmjs.org/pinyin-pro) — 发布版本与时间元数据。
- [pinyin npm](https://www.npmjs.com/package/pinyin) / [hotoo/pinyin README](https://github.com/hotoo/pinyin/blob/master/README.md) — Web/Node 差异、样式和许可证。
- [pinyin registry](https://registry.npmjs.org/pinyin) — 版本、入口、依赖和发布元数据。
- [tiny-pinyin npm](https://www.npmjs.com/package/tiny-pinyin) / [creeperyang/pinyin](https://github.com/creeperyang/pinyin) — 浏览器支持、无多音字限制和许可证。
- [tiny-pinyin registry](https://registry.npmjs.org/tiny-pinyin) — 版本、体积和发布时间。
- [cnchar npm](https://www.npmjs.com/package/cnchar) / [theajack/cnchar](https://github.com/theajack/cnchar) — `spell`、首字母、多音词和浏览器用法。
- [parse-pinyin npm](https://www.npmjs.com/package/parse-pinyin) — ESM、浏览器/Vite、`toPinyin`/`toHanzi` API。
- [pinyin-match-hanzi npm](https://www.npmjs.com/package/pinyin-match-hanzi) — 反向候选方向和依赖信息。

## 实施记录

- 2026-09-01：锁定 `pinyin-pro@3.29.3`，在 `src/lib/address-suggestions.ts` 中通过 `match()` 增加本地全拼、首字母和混合标题匹配；原生标题/主机名/URL 匹配保持更高优先级。
- 纯函数测试覆盖 `py`、`bianyi`、大小写与全角空格、原生匹配优先、多音词、混合标题和 URL 去重，共 13 项通过。
- mock 地址栏实际验证拼音候选、方向键/Enter 导航、无 XHR/fetch 和 32px/54px 布局稳定性。
- 产物相对未接入前增加约 186 KB minified、约 85 KB gzip；10,000 条本地候选的建议计算 p50 26.91ms、p95 33.08ms。
- 依赖仍是静态本地导入，无远程服务、剪贴板访问或新的 Tauri 命令；`pnpm-lock.yaml` 已同步，未改动既有过时的 `package-lock.json`。
