# MCP codegraph 客户端（检索工具面（只读））

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`01-mcp-codegraph-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# MCP codegraph 客户端（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
MCP codegraph 客户端是 codesleuth 只读检索工具面中的结图层数据源：它 spawn 外部 codegraph 服务（MCP over stdio、newline-delimited JSON-RPC），为 agent 的 explore/callers/callees/impact/files 五个结构检索工具提供符号调用图查询能力，会话结束由 Drop 回收子进程。

## 证据列表
1. 价值：通过 spawn 外部 codegraph MCP 服务（stdio JSON-RPC）获得结构图查询能力，供 explore/callers/callees/impact/files 五个图工具使用
   - src/mcp.rs:1-4（审计 #2）
   - src/tools/graph.rs:1-5（审计 #4）
2. 入口与关键文件：src/mcp.rs 定义 McpClient（spawn/initialize/call_tool/Drop 回收）；src/tools/graph.rs 定义 CodegraphEngine 及图工具（ExploreTool/CallersTool 等）；src/cli.rs 负责启动与注册
   - src/mcp.rs:18-25,68,104-114,128-133,198-205（审计 #2）
   - src/tools/graph.rs:18-22,71-102,104-110,169-183,186-188（审计 #4）
3. 运作链：cli.rs 调 CodegraphEngine::start（索引缺失跑 codegraph init、force 则重建，经 bootlock 串行化，然后 spawn `codegraph serve --mcp` 并 initialize 握手）→ agent 调用图工具 execute → engine.call（自动附 projectPath，工具名加 codegraph_ 前缀）→ McpClient.call_tool 发 JSON-RPC
   - src/cli.rs:252-257（审计 #10）
   - src/tools/graph.rs:75-101,105-110（审计 #4）
   - src/mcp.rs:128-133（审计 #2）
4. 模块交互：上游依赖 bootlock 引导锁（D014）与外部 codegraph CLI 二进制（cfg.graph.bin 传入）；下游注册进 ToolRegistry 供 agent 检索（无符号索引时不注册并提示降级用 find_files/grep/read；启动失败非致命降级并审计留痕）
   - src/cli.rs:273-296（审计 #10）
   - src/tools/graph.rs:78-88（审计 #4）

## 死胡同
- grep "CodegraphEngine::start"（plain，含双冒号）零命中，改用 "CodegraphEngine" 命中 src/cli.rs

## 置信度
high

## 统计
turns=6 · tool_calls=6 · duration=17816ms · tokens=51917

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
MCP codegraph 客户端是 codesleuth 只读检索工具面的结图层数据源：装配期由 cli.rs 在 tokio runtime 上调 CodegraphEngine::start（bootlock 引导锁内按需跑 codegraph init / index --force，放锁后 spawn `codegraph serve --mcp` 并完成 initialize 握手），会话内五个图工具（explore/callers/callees/impact/files）经 CodegraphEngine::call（注入 projectPath、加 codegraph_ 前缀）→ McpClient::call_tool 以 newline-delimited JSON-RPC 调用外部 codegraph 服务；单请求 90s 超时，会话结束由 Drop kill 子进程。配置面极窄：graph.bin（默认 "codegraph"）与 --fresh-index，env 层已移除；图工具仅在「engine 启动成功且 .codegraph/codegraph.db 有符号」双重条件下注册，否则降级提示。初稿结论基本正确，一处勘误：初稿引用的 graph.rs:5 称 `CODEGRAPH_NO_DAEMON=1` 保证独占，实际全库该 env 只出现在注释、spawn 代码未设置——独占性由每会话 spawn+Drop kill 实现。

## 证据列表
1. 完整调用链（装配）：run_task_inner 建 tokio runtime → CodegraphEngine::start(&repo_abs, &cfg.graph.bin, self.fresh_index)（src/cli.rs:250-253）→ start_with_bin 先 bootlock acquire_guard（graph.rs:78-82），force_reindex 跑 run_cli(index --force) 或 .codegraph 缺失跑 run_cli(init)（graph.rs:83-87），drop(guard) 放锁后 McpClient::spawn(bin, ["serve","--mcp"], root) + client.initialize() 握手（graph.rs:88-90）
   - src/cli.rs:250-257（审计 #15）
   - src/tools/graph.rs:71-72（审计 #4）
2. 完整调用链（运行期）：agent 调图工具 execute（宏生成：CallersTool/CalleesTool/ImpactTool graph.rs:169-183；ExploreTool execute graph.rs:214-226；FilesTool execute graph.rs:257-263）→ CodegraphEngine::call 注入 args["projectPath"] 并加 codegraph_ 前缀（graph.rs:105-109）→ McpClient::call_tool 发 tools/call（mcp.rs:128-133）→ request 循环：自增 id、写 stdin、带 deadline 读 stdout、id 匹配返回、通知/乱序行忽略（mcp.rs:152-190）
   - src/tools/graph.rs:159-163, 214-226, 257-263（审计 #4）
   - src/tools/graph.rs:104-110（审计 #4）
   - src/mcp.rs:128-133（审计 #2）
   - src/mcp.rs:152-190（审计 #2）
3. 数据流：输入 = agent 的 JSON 参数（symbol/query/filter/limit/depth）→ symbol_args 提取校验 symbol 非空、透传 limit/depth（graph.rs:113-128）→ MCP 工具响应 text content 拼接为 String（mcp.rs:37-53,55-65）→ 工具 execute 加「[codegraph {tool}] {label}」前缀（graph.rs:163,225,263）返回 agent；实际索引数据落在外部 .codegraph/codegraph.db（cli.rs:275，由外部 codegraph 维护，codesleuth 不读写）
   - src/mcp.rs:37-53（审计 #2）
   - src/mcp.rs:55-65（审计 #2）
   - src/tools/graph.rs:161-163（审计 #4）
   - src/cli.rs:275（审计 #15）
4. 配置与开关：唯一配置键 graph.bin（GraphConfig，默认 "codegraph"，src/config.rs:72-76,129），经配置链 CLI>项目>全局>默认合并（config.rs:225-226，键表 389-392）；CLI 旗标 --fresh_index 透传为 force_reindex（cli.rs:43,253）；CODEGRAPH_BIN env 已移除；无其他开关
   - src/config.rs:72-76（审计 #27）
   - src/config.rs:129（审计 #27）
   - src/cli.rs:43,253（审计 #15）
   - src/config.rs:225-226（审计 #27）
   - src/config.rs:389-392（审计 #27）
5. 边界与坑：单请求超时 DEFAULT_TIMEOUT=90s 硬编码（mcp.rs:16,163-168）；CLI init/index 超时 300s（graph.rs:26-42）；spawn 失败 NotFound 给安装 hint（mcp.rs:78-85）；isError=true 或空文本视为工具错误 INDEX_BUILD_FAILED（mcp.rs:40-51）；CLI stderr 为空的锁竞争给 unlock hint（graph.rs:58-63）；server stderr 直接丢弃（Stdio::null，mcp.rs:76）；Drop 用 try_lock，拿不到锁（请求在途）则不 kill（mcp.rs:201-203，极端并发窗口可能泄漏进程，推断）
   - src/mcp.rs:16（审计 #2）
   - src/mcp.rs:159-168（审计 #2）
   - src/tools/graph.rs:26-42（审计 #4）
   - src/mcp.rs:78-85（审计 #2）
   - src/mcp.rs:40-51（审计 #2）
   - src/tools/graph.rs:58-63（审计 #4）
   - src/mcp.rs:76（审计 #2）
   - src/mcp.rs:198-204（审计 #2）
6. 错误处理语义：INDEX_LOCKED（引导锁竞争败者）判负整体退出，不降级（cli.rs:259-262）；其他启动错误非致命——图工具不注册、审计记 degraded 事件、主流程继续（cli.rs:263-271）；诚实工具面：graph_symbols_nonempty = engine 就绪 && repo_map_inputs(cg_db) 符号非空，否则注入地形提示引导用 find_files/grep/read（cli.rs:273-295）；单测覆盖二进制缺失→INDEX_NOT_AVAILABLE（graph.rs:271-278）
   - src/cli.rs:259-262（审计 #15）
   - src/cli.rs:263-271（审计 #15）
   - src/cli.rs:273-295（审计 #15）
   - src/tools/graph.rs:271-278（审计 #4）
7. 与其他功能契约：上游依赖 bootlock（D014）串行化索引引导、放锁后才 serve（graph.rs:76-88）；下游与向量层共享同一 cg_db 文件路径（cli.rs:275,305）——vector 层也读 codegraph.db，repo-map 注册探针复用 repo_map_inputs（cli.rs:276-279）；五工具注册进 ToolRegistry，Arc<CodegraphEngine> 共享（cli.rs:284-288）
   - src/tools/graph.rs:76-88（审计 #4）
   - src/cli.rs:275（审计 #15）
   - src/cli.rs:299-307（审计 #15）
   - src/cli.rs:284-288（审计 #15）
8. 初稿勘误：graph.rs:5 注释称 CODEGRAPH_NO_DAEMON=1 保证每会话独占进程，但 McpClient::spawn 只设置 CODEGRAPH_TELEMETRY=0 与 DO_NOT_TRACK=1（mcp.rs:72-73），CODEGRAPH_NO_DAEMON 全库无实际设置代码——独占性实际由每会话 spawn + Drop start_kill 保证
   - src/tools/graph.rs:5（审计 #4）
   - src/mcp.rs:68-76（审计 #2）
   - src/mcp.rs:72-73（审计 #2）
9. 握手细节与验收：initialize 发 protocolVersion "2025-03-26" + clientInfo codesleuth，随后发 notifications/initialized（mcp.rs:105-114）；服务器响应里的 instructions 字段仅 tracing::debug 记录（Phase 4 预留，graph.rs:91-97）；list_tools 已实现但主路径未调用（mcp.rs:116-126，推断：预留探针）；live 验收测试（默认 #[ignore]）走完整 init→spawn→握手→五工具调用→Drop 回收（tests/graph_live.rs:1-5,12-19,26-48）
   - src/mcp.rs:105-114（审计 #2）
   - src/tools/graph.rs:90-97（审计 #4）
   - src/mcp.rs:116-126（审计 #2）
   - tests/graph_live.rs:1-5（审计 #32）
   - tests/graph_live.rs:12-19（审计 #32）
   - tests/graph_live.rs:26-48（审计 #32）

## 死胡同
- grep "CodegraphEngine|graph"（plain 含竖线）0 命中——plain 模式把 | 当字面量，换 regex 后命中
- CODEGRAPH_NO_DAEMON 全库检索仅命中 src/tools/graph.rs:5 注释行，无实际设置代码（作为勘误而非死路）

## 置信度
high

## 统计
turns=11 · tool_calls=11 · duration=125395ms · tokens=198609


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
MCP codegraph 客户端是 codesleuth 只读检索工具面中的结图层数据源：它 spawn 外部 codegraph 服务（MCP over stdio、newline-delimited JSON-RPC），为 agent 的 explore/callers/callees/impact/files 五个结构检索工具提供符号调用图查询能力，会话结束由 Drop 回收子进程。

## 证据列表
1. 价值：通过 spawn 外部 codegraph MCP 服务（stdio JSON-RPC）获得结构图查询能力，供 explore/callers/callees/impact/files 五个图工具使用
   - src/mcp.rs:1-4（审计 #2）
   - src/tools/graph.rs:1-5（审计 #4）
2. 入口与关键文件：src/mcp.rs 定义 McpClient（spawn/initialize/call_tool/Drop 回收）；src/tools/graph.rs 定义 CodegraphEngine 及图工具（ExploreTool/CallersTool 等）；src/cli.rs 负责启动与注册
   - src/mcp.rs:18-25,68,104-114,128-133,198-205（审计 #2）
   - src/tools/graph.rs:18-22,71-102,104-110,169-183,186-188（审计 #4）
3. 运作链：cli.rs 调 CodegraphEngine::start（索引缺失跑 codegraph init、force 则重建，经 bootlock 串行化，然后 spawn `codegraph serve --mcp` 并 initialize 握手）→ agent 调用图工具 execute → engine.call（自动附 projectPath，工具名加 codegraph_ 前缀）→ McpClient.call_tool 发 JSON-RPC
   - src/cli.rs:252-257（审计 #10）
   - src/tools/graph.rs:75-101,105-110（审计 #4）
   - src/mcp.rs:128-133（审计 #2）
4. 模块交互：上游依赖 bootlock 引导锁（D014）与外部 codegraph CLI 二进制（cfg.graph.bin 传入）；下游注册进 ToolRegistry 供 agent 检索（无符号索引时不注册并提示降级用 find_files/grep/read；启动失败非致命降级并审计留痕）
   - src/cli.rs:273-296（审计 #10）
   - src/tools/graph.rs:78-88（审计 #4）

## 死胡同
- grep "CodegraphEngine::start"（plain，含双冒号）零命中，改用 "CodegraphEngine" 命中 src/cli.rs

## 置信度
high

## 统计
turns=6 · tool_calls=6 · duration=17816ms · tokens=51917
