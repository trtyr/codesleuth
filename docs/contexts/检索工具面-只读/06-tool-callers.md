# callers 调用方查询（检索工具面（只读））

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`06-tool-callers-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# callers 调用方查询（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
callers 是 codesleuth 只读检索工具面中的调用方查询工具：给定符号名，返回「谁调用了它」（codegraph 结构图反向边），帮助 Agent 快速定位符号的上游引用与影响入口。定义于 src/tools/graph.rs:169-173（宏 cg_symbol_tool! 生成 CallersTool，参数 {symbol, limit?}），执行时先经 symbol_args 校验参数（graph.rs:113-128），再由 CodegraphEngine::call 附加 projectPath 并转发给外部 codegraph MCP 服务（graph.rs:104-110，实际查询由 codegraph 完成）。注册在 cli.rs:284-287，仅当 codegraph 存在且符号索引非空时才注册，否则输出地形提示（cli.rs:290-293）。上游依赖：CodegraphEngine 生命周期（start 中 codegraph init/serve，graph.rs:71-102）与 McpClient；下游：与同族 callees/impact/explore 共用同一宏与结图层，作为 Tool 注册进 registry 供 Agent 调用。

# callers 调用方查询（检索工具面 · 只读）

## 1. 是干什么的
给定符号名，返回「谁调用了它」——结构图反向边，供 Agent 快速定位符号的上游引用（src/tools/graph.rs:172 工具描述）。

## 2. 入口与关键文件
- src/tools/graph.rs:169-173 —— CallersTool 定义（宏生成，args: {symbol, limit?}）
- src/tools/graph.rs:159-164 —— execute：校验参数并转发查询
- src/tools/graph.rs:104-110 —— CodegraphEngine::call：附 projectPath 调 MCP
- src/cli.rs:284-287 —— 工具注册（索引非空才注册）

## 3. 怎么运作
CallersTool::execute → symbol_args 校验 symbol → engine.call("callers", args) → McpClient::call_tool("codegraph_callers")，实际反查由外部 codegraph 服务完成，结果格式化为 `[codegraph callers] {symbol}` 返回（graph.rs:162-163）。

## 4. 交互
上游依赖 CodegraphEngine（负责 codegraph init/serve 生命周期，graph.rs:71-102）与 McpClient；与同族 callees/impact/explore 共用宏与结图层（graph.rs:174-183），注册进 cli.rs 的 Tool registry 供 Agent 会话使用。

## 证据列表
1. callers 工具由宏 cg_symbol_tool! 定义为 CallersTool，描述为「谁调用了这个符号（结构图反向边）」，参数 {symbol, limit?}
   - src/tools/graph.rs:169-173（审计 #2）
2. 执行链：execute 经 symbol_args 校验 symbol/limit 后调用 engine.call(tool, a) 转发给 codegraph
   - src/tools/graph.rs:159-164（审计 #2）
   - src/tools/graph.rs:113-128（审计 #2）
3. CodegraphEngine::call 自动附 projectPath，调用 McpClient::call_tool("codegraph_callers")，实际查询由外部 codegraph MCP 服务完成
   - src/tools/graph.rs:104-110（审计 #2）
   - src/tools/graph.rs:89-90（审计 #2）
4. 注册：仅当 codegraph 存在且符号索引非空时注册 CallersTool 等图工具，否则输出地形提示
   - src/cli.rs:282-293（审计 #10）
5. 同族 callees/impact/explore 共用同一宏与结图层；CodegraphEngine 负责 codegraph init/serve 生命周期
   - src/tools/graph.rs:174-183（审计 #2）
   - src/tools/graph.rs:71-102（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=4 · tool_calls=4 · duration=16241ms · tokens=21709

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
callers 是 codesleuth 只读结图层反向查询工具：宏 cg_symbol_tool! 生成的 CallersTool（graph.rs:169-173）校验 symbol 后经 CodegraphEngine::call（注入 projectPath、加 codegraph_ 前缀，graph.rs:104-110）→ McpClient::call_tool 以 newline JSON-RPC tools/call 调外部 codegraph serve --mcp（mcp.rs:128-133），实际反查落在外部进程的 .codegraph/codegraph.db；仅当引擎就绪且符号索引非空才注册（cli.rs:275-294）。初稿三处需修正：①MCP 调用在 mcp.rs:128-133 而非 graph.rs:89-90（后者是 spawn+initialize）；②CODEGRAPH_NO_DAEMON=1 仅存在于注释，实际独占由每会话 spawn+Drop start_kill 保证（mcp.rs:72-73,198-205）；③depth 虽文档写 impact 专属，symbol_args 对 callers 也透传（graph.rs:124-126）。

## 证据列表
1. CallersTool 由宏 cg_symbol_tool! 生成，描述为结构图反向边，参数 schema required=[symbol]，limit/depth 可选
   - src/tools/graph.rs:169-173（审计 #2）
   - src/tools/graph.rs:148-158（审计 #2）
2. execute 链：symbol_args 校验 symbol 非空（USER_INPUT 错误）、透传 limit/depth → engine.call(callers) → 格式化 [codegraph callers] {symbol} 返回
   - src/tools/graph.rs:159-164（审计 #2）
   - src/tools/graph.rs:113-128（审计 #2）
3. CodegraphEngine::call 注入 projectPath 并加 codegraph_ 前缀；McpClient::call_tool 发 tools/call，request 循环自增 id 匹配、90s 超时、通知/乱序/非法行忽略
   - src/tools/graph.rs:104-110（审计 #2）
   - src/mcp.rs:128-133（审计 #25）
   - src/mcp.rs:152-190（审计 #25）
4. 生命周期：CodegraphEngine::start 经 bootlock 引导锁跑 init/index --force，放锁后 spawn serve --mcp + initialize；cli.rs 装配并双重门控注册（引擎就绪+符号非空），否则地形提示降级；INDEX_LOCKED 判负退出，其他错误非致命降级并审计留痕
   - src/tools/graph.rs:71-102（审计 #2）
   - src/cli.rs:250-253（审计 #4）
   - src/cli.rs:275-294（审计 #4）
   - src/cli.rs:259-271（审计 #4）
5. 配置面：唯一配置键 graph.bin 默认 codegraph（配置链 CLI>项目>全局>默认）；CODEGRAPH_BIN env 已移除；spawn 只设 CODEGRAPH_TELEMETRY=0/DO_NOT_TRACK=1，graph.rs:5 注释的 CODEGRAPH_NO_DAEMON=1 无实际设置代码（初稿勘误）；MCP 单请求超时 90s 硬编码，init/index 超时 300s
   - src/mcp.rs:72-73（审计 #25）
   - src/mcp.rs:198-205（审计 #25）
   - src/tools/graph.rs:5（审计 #2）
   - src/config.rs:128-130（审计 #20）
   - src/config.rs:225-226（审计 #20）
   - src/mcp.rs:16（审计 #25）
6. 运行期分发：Harness tools.get 线性查找 ToolRegistry（未知工具拒绝并计入无进展熔断），执行前 audit.record(tool_call) 取 seq；stderr 为 Stdio::null 导致 codegraph 侧错误日志不可见
   - src/harness.rs:331-347（审计 #33）
   - src/harness.rs:350-353（审计 #33）
   - src/tools/mod.rs:38-43（审计 #38）
   - src/mcp.rs:76（审计 #25）
7. 错误边界：isError 响应→INDEX_BUILD_FAILED；二进制缺失→INDEX_NOT_AVAILABLE+安装 hint；锁竞争 stderr 空→unlock hint；单测覆盖 symbol_args 校验与缺失二进制
   - src/mcp.rs:37-45（审计 #25）
   - src/tools/graph.rs:58-63（审计 #2）
   - src/tools/graph.rs:271-292（审计 #2）
8. depth 参数虽 schema 注明 impact 用，symbol_args 对 callers 也透传（初稿勘误③）
   - src/tools/graph.rs:124-126（审计 #2）
   - src/tools/graph.rs:284-291（审计 #2）

## 死胡同
- CODEGRAPH_NO_DAEMON 全库检索无实际设置代码（仅 graph.rs:5 注释），转为勘误
- harness.rs:365-388 evidence.observe/info_keys 段未直接读取，标推断

## 置信度
high

## 统计
turns=10 · tool_calls=15 · duration=131599ms · tokens=182409


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
callers 是 codesleuth 只读检索工具面中的调用方查询工具：给定符号名，返回「谁调用了它」（codegraph 结构图反向边），帮助 Agent 快速定位符号的上游引用与影响入口。定义于 src/tools/graph.rs:169-173（宏 cg_symbol_tool! 生成 CallersTool，参数 {symbol, limit?}），执行时先经 symbol_args 校验参数（graph.rs:113-128），再由 CodegraphEngine::call 附加 projectPath 并转发给外部 codegraph MCP 服务（graph.rs:104-110，实际查询由 codegraph 完成）。注册在 cli.rs:284-287，仅当 codegraph 存在且符号索引非空时才注册，否则输出地形提示（cli.rs:290-293）。上游依赖：CodegraphEngine 生命周期（start 中 codegraph init/serve，graph.rs:71-102）与 McpClient；下游：与同族 callees/impact/explore 共用同一宏与结图层，作为 Tool 注册进 registry 供 Agent 调用。

# callers 调用方查询（检索工具面 · 只读）

## 1. 是干什么的
给定符号名，返回「谁调用了它」——结构图反向边，供 Agent 快速定位符号的上游引用（src/tools/graph.rs:172 工具描述）。

## 2. 入口与关键文件
- src/tools/graph.rs:169-173 —— CallersTool 定义（宏生成，args: {symbol, limit?}）
- src/tools/graph.rs:159-164 —— execute：校验参数并转发查询
- src/tools/graph.rs:104-110 —— CodegraphEngine::call：附 projectPath 调 MCP
- src/cli.rs:284-287 —— 工具注册（索引非空才注册）

## 3. 怎么运作
CallersTool::execute → symbol_args 校验 symbol → engine.call("callers", args) → McpClient::call_tool("codegraph_callers")，实际反查由外部 codegraph 服务完成，结果格式化为 `[codegraph callers] {symbol}` 返回（graph.rs:162-163）。

## 4. 交互
上游依赖 CodegraphEngine（负责 codegraph init/serve 生命周期，graph.rs:71-102）与 McpClient；与同族 callees/impact/explore 共用宏与结图层（graph.rs:174-183），注册进 cli.rs 的 Tool registry 供 Agent 会话使用。

## 证据列表
1. callers 工具由宏 cg_symbol_tool! 定义为 CallersTool，描述为「谁调用了这个符号（结构图反向边）」，参数 {symbol, limit?}
   - src/tools/graph.rs:169-173（审计 #2）
2. 执行链：execute 经 symbol_args 校验 symbol/limit 后调用 engine.call(tool, a) 转发给 codegraph
   - src/tools/graph.rs:159-164（审计 #2）
   - src/tools/graph.rs:113-128（审计 #2）
3. CodegraphEngine::call 自动附 projectPath，调用 McpClient::call_tool("codegraph_callers")，实际查询由外部 codegraph MCP 服务完成
   - src/tools/graph.rs:104-110（审计 #2）
   - src/tools/graph.rs:89-90（审计 #2）
4. 注册：仅当 codegraph 存在且符号索引非空时注册 CallersTool 等图工具，否则输出地形提示
   - src/cli.rs:282-293（审计 #10）
5. 同族 callees/impact/explore 共用同一宏与结图层；CodegraphEngine 负责 codegraph init/serve 生命周期
   - src/tools/graph.rs:174-183（审计 #2）
   - src/tools/graph.rs:71-102（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=4 · tool_calls=4 · duration=16241ms · tokens=21709
