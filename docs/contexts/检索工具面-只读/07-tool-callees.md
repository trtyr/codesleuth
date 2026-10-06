# callees 被调查询（检索工具面（只读））

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`07-tool-callees-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# callees 被调查询（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

callees 是 codesleuth 只读检索工具面中的结图层查询工具：给定符号名，返回该符号调用了谁（codegraph 结构图正向边），帮助 Agent 理解代码流向。入口由宏 cg_symbol_tool! 生成 CalleesTool，execute 校验 symbol 后转发 CodegraphEngine::call，再经 McpClient 以 MCP tools/call 调外部 codegraph server 的 codegraph_callees 工具；仅当 codegraph 索引存在且符号非空时才注册。

## 证据列表

1. 功能：callees 查询「这个符号调用了谁」（结构图正向边），参数 {symbol}，是五图工具（explore/callers/callees/impact/files）之一
    - src/tools/graph.rs:174-178（审计 #2）
2. 入口：CalleesTool 由宏 cg_symbol_tool! 生成，实现统一 Tool trait（name/description/parameters/execute）
    - src/tools/graph.rs:130-167（审计 #2）
3. 校验：execute 经 symbol_args 要求 symbol 必填（非空），透传可选 limit/depth
    - src/tools/graph.rs:113-128（审计 #2）
4. 调用链：CalleesTool.execute → CodegraphEngine.call("callees") 自动附加 projectPath → McpClient.call_tool("codegraph_callees")（MCP over stdio JSON-RPC）调外部 codegraph server
    - src/tools/graph.rs:104-110（审计 #2）
    - docs/检索工具面-只读/09-mcp-codegraph.md:6（审计 #16）
5. 注册与门控：cli.rs 仅在 codegraph 存在且 .codegraph/codegraph.db 符号索引非空时注册 CalleesTool 等五工具，否则注入地形提示改用 find_files/grep/read
    - src/cli.rs:275-294（审计 #2）
6. 交互：上游依赖 src/mcp.rs 的 McpClient 与 CodegraphEngine（init/serve 生命周期）；与同族 callers/impact/explore 共用同一宏与结图层；vector 召回层（chunk.rs relations_for_symbol）也查询 callees 关系作为关系上下文
    - src/vector/chunk.rs:83-88（审计 #14）
    - docs/检索工具面-只读/09-mcp-codegraph.md:12（审计 #16）

## 死胡同

无

## 置信度

high

## 统计

turns=5 · tool_calls=6 · duration=34438ms · tokens=31722

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
callees 是 codesleuth 结图层的被调查询工具：Agent 给定 symbol，返回该符号调用了谁（结构图正向边）。链路：CalleesTool.execute（宏 cg_symbol_tool! 生成，symbol_args 校验 symbol 非空、透传可选 limit/depth）→ CodegraphEngine.call("callees")（自动附 projectPath、加 codegraph_ 前缀）→ McpClient.call_tool 经 stdio JSON-RPC tools/call 调外部 codegraph serve --mcp 子进程 → extract_tool_text 拼接返回（带 [codegraph callees] 头）。装配门控：engine 启动成功且 .codegraph/codegraph.db 符号索引非空才注册，否则地形提示降级 find_files/grep。配置仅 graph.bin（默认 "codegraph"）+ --fresh-index，env 层已移除。vector 层 relations_for_symbol 以只读 SQL 旁路直查同一 db。勘误：初稿"仅当索引存在才注册"实为符号索引非空判定且缺失会先 init；引用的 09-mcp-codegraph.md 路径/行号有误，实际为 docs/contexts/检索工具面-只读/01-mcp-codegraph.md。

## 证据列表
1. 工具定义：CalleesTool 由宏 cg_symbol_tool! 生成，实现统一 Tool trait，参数 schema required:[symbol] + 可选 limit/depth
   - src/tools/graph.rs:130-167（审计 #2）
   - src/tools/graph.rs:174-178（审计 #2）
2. execute：symbol_args 校验 symbol 非空（否则 USER_INPUT），engine.call 后包装输出 [codegraph callees] {label}\n{out}
   - src/tools/graph.rs:159-164（审计 #2）
   - src/tools/graph.rs:113-128（审计 #2）
3. CodegraphEngine::call 自动附加 projectPath 并加 codegraph_ 前缀调 codegraph_callees
   - src/tools/graph.rs:104-110（审计 #2）
4. McpClient.call_tool 发 newline-delimited JSON-RPC tools/call；request 循环匹配 id，90s 超时（DEFAULT_TIMEOUT），非法行跳过、通知忽略
   - src/mcp.rs:128-133（审计 #4）
   - src/mcp.rs:152-190（审计 #4）
   - src/mcp.rs:16（审计 #4）
5. 响应解析：isError=true 或空 text 均报 INDEX_BUILD_FAILED（空 text 提示符号不存在或索引未就绪）
   - src/mcp.rs:40-51（审计 #4）
6. 子进程生命周期：McpClient.spawn 启动 codegraph serve --mcp（注入 CODEGRAPH_TELEMETRY=0/DO_NOT_TRACK=1，stderr 丢弃），Drop 回收；引擎先经 bootlock 做 init/index --force 再握手
   - src/mcp.rs:68-102（审计 #4）
   - src/tools/graph.rs:75-101（审计 #2）
7. 装配：CodegraphEngine::start(&repo_abs, &cfg.graph.bin, self.fresh_index)；INDEX_LOCKED 判负退出，其他错误非致命降级并审计 degraded
   - src/cli.rs:250-272（审计 #9）
8. 诚实工具面门控：repo_map_inputs(cg_db) 符号非空才注册五图工具，否则注入地形提示改用 find_files/grep/read
   - src/cli.rs:275-295（审计 #9）
9. 配置：唯一结图配置键 graph.bin 默认 "codegraph"，经 默认←全局←项目←CLI 配置链合并；环境变量层已整体移除（CODEGRAPH_BIN 已删）
   - src/config.rs:1-3（审计 #25）
   - src/config.rs:72-76（审计 #25）
   - src/config.rs:128-130（审计 #25）
10. vector 召回层旁路：relations_for_symbol 以 rusqlite 只读直查同一 codegraph.db 拿 callees（e.source=?1，LIMIT 8 去重）作为嵌入关系上下文
   - src/vector/chunk.rs:118-133（审计 #14）
   - src/vector/chunk.rs:83-88（审计 #14）
11. 坑：宏 schema 注释称 depth 为 impact 用，但 symbol_args 对所有图工具盲透传 limit/depth，callees 侧语义未在仓内验证（推断：外部忽略）
   - src/tools/graph.rs:148-158（审计 #2）
   - src/tools/graph.rs:120-126（审计 #2）
12. 坑：codegraph 锁竞争 stderr 为空时有专用 hint（codegraph unlock）；引导锁竞争败者判负退出不降级
   - src/tools/graph.rs:58-63（审计 #2）
   - src/tools/graph.rs:76-88（审计 #2）

## 死胡同
- CODEGRAPH_NO_DAEMON / CODEGRAPH_BIN：grep 无命中，确认 env 已移除，仅源码注释残留（graph.rs:1-5）
- grep 字面量 graph.bin 无命中（serde 展开），改读 src/config.rs 落实
- codegraph.db 表 schema 属外部工具，仓内仅有 chunk.rs 只读 SQL 反推，无法读到 DDL

## 置信度
high

## 统计
turns=9 · tool_calls=14 · duration=129543ms · tokens=153849


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

callees 是 codesleuth 只读检索工具面中的结图层查询工具：给定符号名，返回该符号调用了谁（codegraph 结构图正向边），帮助 Agent 理解代码流向。入口由宏 cg_symbol_tool! 生成 CalleesTool，execute 校验 symbol 后转发 CodegraphEngine::call，再经 McpClient 以 MCP tools/call 调外部 codegraph server 的 codegraph_callees 工具；仅当 codegraph 索引存在且符号非空时才注册。

## 证据列表

1. 功能：callees 查询「这个符号调用了谁」（结构图正向边），参数 {symbol}，是五图工具（explore/callers/callees/impact/files）之一
    - src/tools/graph.rs:174-178（审计 #2）
2. 入口：CalleesTool 由宏 cg_symbol_tool! 生成，实现统一 Tool trait（name/description/parameters/execute）
    - src/tools/graph.rs:130-167（审计 #2）
3. 校验：execute 经 symbol_args 要求 symbol 必填（非空），透传可选 limit/depth
    - src/tools/graph.rs:113-128（审计 #2）
4. 调用链：CalleesTool.execute → CodegraphEngine.call("callees") 自动附加 projectPath → McpClient.call_tool("codegraph_callees")（MCP over stdio JSON-RPC）调外部 codegraph server
    - src/tools/graph.rs:104-110（审计 #2）
    - docs/检索工具面-只读/09-mcp-codegraph.md:6（审计 #16）
5. 注册与门控：cli.rs 仅在 codegraph 存在且 .codegraph/codegraph.db 符号索引非空时注册 CalleesTool 等五工具，否则注入地形提示改用 find_files/grep/read
    - src/cli.rs:275-294（审计 #2）
6. 交互：上游依赖 src/mcp.rs 的 McpClient 与 CodegraphEngine（init/serve 生命周期）；与同族 callers/impact/explore 共用同一宏与结图层；vector 召回层（chunk.rs relations_for_symbol）也查询 callees 关系作为关系上下文
    - src/vector/chunk.rs:83-88（审计 #14）
    - docs/检索工具面-只读/09-mcp-codegraph.md:12（审计 #16）

## 死胡同

无

## 置信度

high

## 统计

turns=5 · tool_calls=6 · duration=34438ms · tokens=31722
