# repo map 导航图（向量语义检索）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`03-repo-map-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# repo map 导航图（向量语义检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

repo map 导航图：--repo-map 开启时，从 codegraph SQLite 读符号与边表度数，按种子→邻居→度数降序在默认 24000 字符预算内贪心生成结构图，经 wrap_repo_section 包成 [repo map] 段注入首条用户消息。有召回命中时生成任务导航图（build_task_map），无召回则全局图（build_repo_map），失败均非致命降级并留审计痕。

## 证据列表

1. 功能定位：codegraph 符号+边表度数生成预算化（默认 24000 字符）仓库导航图，注入 system prompt 尾部 [repo map] 段
    - src/vector/repomap.rs:1-12（审计 #2）
2. 两个生成器：build_task_map（召回种子优先→一跳邻居→度数降序，◈/◇ 标记）与 build_repo_map（纯度数降序），预算满即截断
    - src/vector/repomap.rs:67-135（审计 #2）
    - src/vector/repomap.rs:137-183（审计 #2）
3. 数据来源：repo_map_inputs 只读打开 codegraph SQLite，查 nodes（排除容器型节点）与 edges 累计度数
    - src/vector/repomap.rs:23-64（审计 #2）
4. 入口链：setup_vector_layer 先 recall 取命中，经 chunk::relations_for_symbol 取邻居，调 build_task_map 并 wrap_repo_section 组后缀
    - src/cli.rs:532-659（审计 #7）
5. 兜底路径：仅 --repo-map 无向量时直接 repo_map_inputs→build_repo_map 生成全局图
    - src/cli.rs:333-351（审计 #7）
6. 下游消费与降级：后缀经 Harness::with_first_user_suffix 注入首条消息；向量层失败非致命降级并记审计 degraded
    - src/cli.rs:361-365（审计 #7）
    - src/cli.rs:315-326（审计 #7）
7. 预算可配：vector.repomap_budget 默认 24_000
    - src/config.rs:119-126（审计 #21）

## 死胡同

无

## 置信度

high

## 统计

turns=8 · tool_calls=7 · duration=36026ms · tokens=71599

【初稿内容结束】

深挖要求（在初稿基础上展开，不是另起炉灶）：
1. 完整调用链：从入口到落点的每一步，谁调用谁，全部带 path:line。
2. 数据流：输入从哪来、经过什么处理、落到哪（表 / 文件 / 下游服务）。
3. 配置与开关：这个功能受哪些配置项 / 环境变量影响，默认行为是什么。
4. 边界与坑：错误处理、特殊分支、已知限制、容易误用的地方。
5. 与其他功能的交互契约：上下游接口 / 事件 / 共享存储。
6. 配图：按文末【画图技能规格】为本功能画一张组件/调用关系架构图，
   完整单文件 HTML 放在单独围栏块中（其他正文不要放进这个块）：

   图内容：本功能的调用链与组件关系，组件名用真实模块/文件名，关系带标签；
   省略规格中「Export Toolbar / 导出工具栏」相关要求（文档内嵌图不需要），
   其余设计系统规则（配色/字体/网格背景/间距/图例位置/自包含）全部遵守。
硬性纪律：每个结论带 path:line 证据；没读懂的显式标（推断）+ 最简验证途径；发现初稿有错的，明确写「初稿说 X，实际是 Y」。凭据只写变量名。

【画图技能规格】
---
name: architecture-diagram
description: Create polished dark-themed architecture diagrams as self-contained HTML+SVG files. Use when the user asks for system, infrastructure, cloud, security, or network topology diagrams.
---

# Architecture Diagram Skill

Create professional technical architecture diagrams as self-contained HTML files with inline SVG graphics and CSS styling.

> **Version 1.1** · MIT License · Authored by [Cocoon AI](mailto:hello@cocoon-ai.com)

## Design System

### Color Palette

Use these semantic colors for component types:

| Component Type | Fill (rgba) | Stroke |
|---------------|-------------|--------|
| Frontend | `rgba(8, 51, 68, 0.4)` | `#22d3ee` (cyan-400) |
| Backend | `rgba(6, 78, 59, 0.4)` | `#34d399` (emerald-400) |
| Database | `rgba(76, 29, 149, 0.4)` | `#a78bfa` (violet-400) |
| AWS/Cloud | `rgba(120, 53, 15, 0.3)` | `#fbbf24` (amber-400) |
| Security | `rgba(136, 19, 55, 0.4)` | `#fb7185` (rose-400) |
| Message Bus | `rgba(251, 146, 60, 0.3)` | `#fb923c` (orange-400) |
| External/Generic | `rgba(30, 41, 59, 0.5)` | `#94a3b8` (slate-400) |

### Typography

Use JetBrains Mono for all text (monospace, technical aesthetic):
```html
<link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&display=swap" rel="stylesheet">
```

Font sizes: 12px for component names, 9px for sublabels, 8px for annotations, 7px for tiny labels.

### Visual Elements

**Background:** `#020617` (slate-950) with subtle grid pattern:
```svg
<pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
  <path d="M 40 0 L 0 0 0 40" fill="none" stroke="#1e293b" stroke-width="0.5"/>
</pattern>
```

**Component boxes:** Rounded rectangles (`rx="6"`) with 1.5px stroke, semi-transparent fills.

**Security groups:** Dashed stroke (`stroke-dasharray="4,4"`), transparent fill, rose color.

**Region boundaries:** Larger dashed stroke (`stroke-dasharray="8,4"`), amber color, `rx="12"`.

**Arrows:** Use SVG marker for arrowheads:
```svg
<marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
  <polygon points="0 0, 10 3.5, 0 7" fill="#64748b" />
</marker>
```

**Arrow z-order:** Draw connection arrows early in the SVG (after the background grid) so they render behind component boxes. SVG elements are painted in document order, so arrows drawn first will appear behind shapes drawn later.

**Masking arrows behind transparent fills:** Since component boxes use semi-transparent fills (`rgba(..., 0.4)`), arrows behind them will show through. To fully mask arrows, draw an opaque background rect (e.g., `fill="#0f172a"`) at the same position before drawing the semi-transparent styled rect on top:
```svg
<!-- Opaque background to mask arrows -->
<rect x="X" y="Y" width="W" height="H" rx="6" fill="#0f172a"/>
<!-- Styled component on top -->
<rect x="X" y="Y" width="W" height="H" rx="6" fill="rgba(76, 29, 149, 0.4)" stroke="#a78bfa" stroke-width="1.5"/>
```

**Auth/security flows:** Dashed lines in rose color (`#fb7185`).

**Message buses / Event buses:** Small connector elements between services. Use orange color (`#fb923c` stroke, `rgba(251, 146, 60, 0.3)` fill):
```svg
<rect x="X" y="Y" width="120" height="20" rx="4" fill="rgba(251, 146, 60, 0.3)" stroke="#fb923c" stroke-width="1"/>
<text x="CENTER_X" y="Y+14" fill="#fb923c" font-size="7" text-anchor="middle">Kafka / RabbitMQ</text>
```

### Spacing Rules

**CRITICAL:** When stacking components vertically, ensure proper spacing to avoid overlaps:

- **Standard component height:** 60px for services, 80-120px for larger components
- **Minimum vertical gap between components:** 40px
- **Inline connectors (message buses):** Place IN the gap between components, not overlapping

**Example vertical layout:**
```
Component A: y=70,  height=60  → ends at y=130
Gap:         y=130 to y=170   → 40px gap, place bus at y=140 (20px tall)
Component B: y=170, height=60  → ends at y=230
```

**Wrong:** Placing a message bus at y=160 when Component B starts at y=170 (causes overlap)
**Right:** Placing a message bus at y=140, centered in the 40px gap (y=130 to y=170)

### Legend Placement

**CRITICAL:** Place legends OUTSIDE all boundary boxes (region boundaries, cluster boundaries, security groups).

- Calculate where all boundaries end (y position + height)
- Place legend at least 20px below the lowest boundary
- Expand SVG viewBox height if needed to accommodate

**Example:**
```
Kubernetes Cluster: y=30, height=460 → ends at y=490
Legend should start at: y=510 or below
SVG viewBox height: at least 560 to fit legend
```

**Wrong:** Legend at y=470 inside a cluster boundary that ends at y=490
**Right:** Legend at y=510, below the cluster boundary, with viewBox height extended

### Layout Structure

1. **Header** - Title with pulsing dot indicator, subtitle, and export toolbar
2. **Main SVG diagram** - Contained in rounded border card
3. **Summary cards** - Grid of 3 cards below diagram with key details
4. **Footer** - Minimal metadata line

### Export Toolbar (built-in)

Every diagram ships with a single unobtrusive `⋯` toggle in the header. Click it to reveal three buttons — 📋 Copy (high-DPI PNG to clipboard, scale: 2), 🖼️ PNG (high-DPI PNG download), 📄 PDF (PNG embedded in a one-page PDF via jsPDF). The toolbar collapses back to the icon by default so it doesn't clutter the diagram. All three formats use the same html2canvas capture (with the toolbar excluded and 32px padding around the content), so PDF preserves the dark theme without going through the browser's print dialog.

When generating a new diagram, keep these intact in the template:
- The two CDN scripts in `<head>` (pinned versions, with Subresource Integrity hashes and `crossorigin="anonymous"`):
  - `https://cdn.jsdelivr.net/npm/html2canvas@1.4.1/dist/html2canvas.min.js` — `integrity="sha384-ZZ1pncU3bQe8y31yfZdMFdSpttDoPmOZg2wguVK9almUodir1PghgT0eY7Mrty8H"`
  - `https://cdn.jsdelivr.net/npm/jspdf@2.5.2/dist/jspdf.umd.min.js` — `integrity="sha384-en/ztfPSRkGfME4KIm05joYXynqzUgbsG5nMrj/xEFAHXkeZfO3yMK8QQ+mP7p1/"`
  - SRI ensures generated diagrams are tamper-resistant against CDN compromise. Do not modify the hashes; if the version is bumped, the new hash must be computed fresh.
- `id="report-container"` on the outermost `.container` div (this is what gets captured)
- `.toolbar` markup with `.toolbar-actions` (collapsed by default) and `.toolbar-toggle` (the `⋯` button)
- `.toolbar` CSS + `@media print { .toolbar { display: none !important; } }`
- `copyAsImage()`, `downloadPNG()`, and `downloadPDF()` script before `</body>`, all using `getBoundingClientRect()` + `html2canvas(document.body, { x, y, width, height, ignoreElements })` to capture a precise rect with breathing room and no toolbar

Caveats: clipboard API needs a user gesture and a secure context (https/file/localhost). SVG `<foreignObject>` renders inconsistently in html2canvas — stick to plain `<svg>` shapes and `<text>`. Bump `scale: 2` to `3` or `4` for higher-res output.

### Component Box Pattern

```svg
<rect x="X" y="Y" width="W" height="H" rx="6" fill="FILL_COLOR" stroke="STROKE_COLOR" stroke-width="1.5"/>
<text x="CENTER_X" y="Y+20" fill="white" font-size="11" font-weight="600" text-anchor="middle">LABEL</text>
<text x="CENTER_X" y="Y+36" fill="#94a3b8" font-size="9" text-anchor="middle">sublabel</text>
```

### Info Card Pattern

```html
<div class="card">
  <div class="card-header">
    <div class="card-dot COLOR"></div>
    <h3>Title</h3>
  </div>
  <ul>
    <li>• Item one</li>
    <li>• Item two</li>
  </ul>
</div>
```

## Template

Copy and customize the template at `resources/template.html`. Key customization points:

1. Update the `<title>` and header text
2. Modify SVG viewBox dimensions if needed (default: `1000 x 680`)
3. Add/remove/reposition component boxes
4. Draw connection arrows between components
5. Update the three summary cards
6. Update footer metadata

## Output

Always produce a single self-contained `.html` file with:
- Embedded CSS (no external stylesheets except Google Fonts)
- Inline SVG (no external images)
- No JavaScript required (pure CSS animations)

The file should render correctly when opened directly in any modern browser. The export toolbar uses two CDN scripts (html2canvas and jsPDF) — no other JavaScript dependencies.


## 结论
repo map 导航图功能：CLI 旗标 --repo-map（可与 --vector 叠加）控制。--vector 时 setup_vector_layer 先 recall(task,10) 取召回命中，对命中符号经 chunk::relations_for_symbol 从 codegraph SQLite 取一跳 callers/callees（每侧 LIMIT 8），再从同一 SQLite 全量读符号（nodes，排除容器 kind）与 edges 双向累计度数，由 build_task_map 按「种子序→邻居→度数降序→名称/路径升序」确定性排序贪心装填 vector.repomap_budget（默认 24000 字符，单行封顶 200 字符）；仅有 --repo-map 无向量（或召回失败）时 build_repo_map 纯度数降序生成全局图。图文本经 wrap_repo_section 包成「[repo map]」段，与召回块拼接为首条用户消息后缀，经 Harness::with_first_user_suffix 注入（初稿证据 1 说「注入 system prompt 尾部」，代码注释 repomap.rs:3 与 wrap_repo_section doc 也这么写，但实际注入点是首条 user 消息后缀——harness.rs:67-68 明确「非 system」，属代码内注释与实现不一致）。所有失败路径非致命降级并记审计 degraded，唯引导锁竞争（INDEX_LOCKED）判负退出。

## 证据列表
1. 入口：run_task_inner 校验 task/repo 后三层配置加载（CLI>项目>全局>默认），--repo-map 为 Cli 结构布尔旗标 self.repo_map
   - src/cli.rs:49（审计 #7）
   - src/cli.rs:190-216（审计 #7）
2. 前置：CodegraphEngine::start（--fresh-index 透传）启动 codegraph MCP；cg_db = repo_abs/.codegraph/codegraph.db；用 repo_map_inputs 非空判定 graph_symbols_nonempty（诚实工具面）——repo_map_inputs 因此在启动期被多调用一次做探针
   - src/cli.rs:252-279（审计 #7）
3. --vector 路径：setup_vector_layer(ctx, cfg, self.repo_map, registry)，返回 Ok(Some(suffix)) 存入 first_suffix；Err 且 INDEX_LOCKED 直接 return Err 判负退出，其余 Err 非致命降级并审计 record("degraded", component=vector_layer)
   - src/cli.rs:298-327（审计 #7）
4. setup_vector_layer 内：resolve_embed_endpoint（[vector] base_url/api_key 缺省跟随 [llm]）→ EmbedClient → bootlock acquire_guard（竞争判负）→ build_vector_index 补建（失败降级复用旧索引）→ VectorStore::open 失败返回 Ok(None)（无召回层继续）→ 注册 vector_search 工具 → recall(task, 10)（失败降级空集+审计 degraded component=recall）
   - src/cli.rs:532-556（审计 #7）
   - src/cli.rs:559-595（审计 #7）
   - src/cli.rs:596-612（审计 #7）
5. 种子/邻居构造：仅当 repo_map=true 且 recall 成功；命中按序遍历，跳过 kind==fallback/leftover 与重复 symbol，逐个调 chunk::relations_for_symbol(cg_db, c.file, c.symbol) 取 (callers, callees) 并入 neighbors，symbol push 进 seeds；build_task_map(symbols, degrees, budget, seeds, neighbors)
   - src/cli.rs:617-647（审计 #7）
6. relations_for_symbol：只读开 SQLite，按 file_path+name 定位 node id（ORDER BY start_line LIMIT 1），callers = edges 反查 JOIN LIMIT 8；节点不存在返回空（不报错），错误由调用方 unwrap_or_default 降级
   - src/vector/chunk.rs:83-109（审计 #20）
7. 数据输入 repo_map_inputs：SQLITE_OPEN_READ_ONLY 打开 codegraph.db；nodes 查 kind IN (function/method/struct/class/interface/impl/trait)（排除 file/import 容器）；edges 逐行 source/target 双向 degrees+1；坏行 warn 跳过；所有错误映射 INDEX_NOT_AVAILABLE
   - src/vector/repomap.rs:23-64（审计 #2）
8. build_task_map：排序键 = 种子序(usize::MAX 兜底) → 是否邻居 → 度数降序 → name 升序 → file_path 升序；行格式「file:line name（kind，度 N）◈/◇」；used+min(len,200)>budget 即 break 截断并追加「…共 N 符号」提示行
   - src/vector/repomap.rs:70-135（审计 #2）
9. build_repo_map：同格式无 ◈/◇ 标记，纯度数降序→name→file_path；out 为空时追加「（仓库无符号索引）」——注意：预算极小导致首行装不下时也会触发此误导性文案
   - src/vector/repomap.rs:137-178（审计 #2）
10. wrap_repo_section：包成「[repo map]\n{map}\n结构详情勿凭此图推断，用 explore / callers / callees 查证。」
   - src/vector/repomap.rs:180-183（审计 #2）
11. 兜底路径：first_suffix 为 None 且 self.repo_map 时直接 repo_map_inputs→build_repo_map→wrap_repo_section；失败 warn + 审计 degraded(component=repo_map)，跳过注入
   - src/cli.rs:333-351（审计 #7）
12. 落点：Harness::with_first_user_suffix 存 first_user_suffix；run() 中拼到首条 User 消息尾部（D005：system 恒定、任务派生数据走 user）——推翻「注入 system prompt 尾部」的初稿表述与代码内过时注释（repomap.rs:3）
   - src/cli.rs:361-365（审计 #7）
   - src/harness.rs:67-72（审计 #22）
   - src/harness.rs:83-88（审计 #22）
13. 配置与开关：vector.repomap_budget 默认 24_000（config.toml 可覆盖，env 层已整体移除）；硬编码 DEFAULT_BUDGET_CHARS=24_000、MAX_LINE_CHARS=200；旗标 --vector/--repo-map/--fresh-index 与 graph.bin 键
   - src/config.rs:119-126（审计 #9）
   - src/vector/repomap.rs:9-12（审计 #2）
14. 坑①：setup_vector_layer 内地图构建 Err 仅 warn，无审计留痕（与兜底路径 342-348 记 degraded 不对称）；坑②：--vector 开但向量库打开失败时 Ok(None)，仅 --repo-map 分支接管生成全局图（初稿「无召回则全局图」成立，但机制是 None 回退而非显式分支）
   - src/cli.rs:649-651（审计 #7）
   - AGENTS.md:37（审计 #33）

## 死胡同
无

## 置信度
high

## 统计
turns=10 · tool_calls=12 · duration=159102ms · tokens=181745


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

repo map 导航图：--repo-map 开启时，从 codegraph SQLite 读符号与边表度数，按种子→邻居→度数降序在默认 24000 字符预算内贪心生成结构图，经 wrap_repo_section 包成 [repo map] 段注入首条用户消息。有召回命中时生成任务导航图（build_task_map），无召回则全局图（build_repo_map），失败均非致命降级并留审计痕。

## 证据列表

1. 功能定位：codegraph 符号+边表度数生成预算化（默认 24000 字符）仓库导航图，注入 system prompt 尾部 [repo map] 段
    - src/vector/repomap.rs:1-12（审计 #2）
2. 两个生成器：build_task_map（召回种子优先→一跳邻居→度数降序，◈/◇ 标记）与 build_repo_map（纯度数降序），预算满即截断
    - src/vector/repomap.rs:67-135（审计 #2）
    - src/vector/repomap.rs:137-183（审计 #2）
3. 数据来源：repo_map_inputs 只读打开 codegraph SQLite，查 nodes（排除容器型节点）与 edges 累计度数
    - src/vector/repomap.rs:23-64（审计 #2）
4. 入口链：setup_vector_layer 先 recall 取命中，经 chunk::relations_for_symbol 取邻居，调 build_task_map 并 wrap_repo_section 组后缀
    - src/cli.rs:532-659（审计 #7）
5. 兜底路径：仅 --repo-map 无向量时直接 repo_map_inputs→build_repo_map 生成全局图
    - src/cli.rs:333-351（审计 #7）
6. 下游消费与降级：后缀经 Harness::with_first_user_suffix 注入首条消息；向量层失败非致命降级并记审计 degraded
    - src/cli.rs:361-365（审计 #7）
    - src/cli.rs:315-326（审计 #7）
7. 预算可配：vector.repomap_budget 默认 24_000
    - src/config.rs:119-126（审计 #21）

## 死胡同

无

## 置信度

high

## 统计

turns=8 · tool_calls=7 · duration=36026ms · tokens=71599
