# explore 结构图探索（检索工具面（只读））

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`05-tool-explore-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# explore 结构图探索（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
explore 是 codesleuth 只读检索工具面中的结构图探索工具：一次调用返回与 query（符号名/文件名/自然语言）相关的符号源码、调用路径与影响面，专为「开局看图」设计。实现上它是 codegraph 外部 MCP 服务的薄封装：ExploreTool::execute 校验 query 后经 CodegraphEngine.call 转发为 MCP 工具 codegraph_explore，引擎在启动时通过 codegraph CLI init/serve 建立连接。CLI 启动时仅在 codegraph 存在且符号索引非空时才注册该工具（诚实工具面策略），否则以地形提示告知降级用 find_files/grep/read。

## 证据列表
1. explore 工具定位：一次性返回相关符号源码 + 调用路径 + 影响面，供开局看图/流程问题使用，参数为 query（符号名/文件/自然语言）
   - src/tools/graph.rs:185-204（审计 #2）
2. 实现：execute 校验非空 query 后经 engine.call("explore", {query}) 转发，输出加 [codegraph explore] 前缀
   - src/tools/graph.rs:214-226（审计 #2）
3. 转发层：CodegraphEngine::call 自动附 projectPath，调用 McpClient.call_tool("codegraph_explore")
   - src/tools/graph.rs:104-110（审计 #2）
4. 上游依赖：CodegraphEngine::start 负责索引引导（init/force，带引导锁）并 spawn codegraph serve --mcp 完成握手，会话内五工具复用（模块头注释 graph.rs:1-5）
   - src/tools/graph.rs:69-102（审计 #2）
5. 注册与准入：CLI 启动时仅在 codegraph 存在且符号索引非空（graph_symbols_nonempty）时注册 ExploreTool 及 callers/callees/impact/files；否则不注册并注入地形提示引导改用 find_files/grep/read
   - src/cli.rs:273-295（审计 #11）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=4 · duration=13264ms · tokens=29786

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
explore 是 codegraph MCP 外部服务的薄封装：agent 发 {query} → ExploreTool::execute 校验非空 → CodegraphEngine::call 注入 projectPath 并加 codegraph_ 前缀 → McpClient::call_tool 走 newline-delimited JSON-RPC（stdio、90s 超时）→ 外部 codegraph serve --mcp 查 .codegraph/codegraph.db → 文本回传加 [codegraph explore] 前缀。会话启动时经 bootlock 引导锁做 init/index --force，双重准入（engine 就绪 ∧ codegraph.db 符号非空）才注册工具，否则注入地形提示降级 find_files/grep/read。配置仅 graph.bin（默认 "codegraph"）+ --fresh_index 旗标；CODEGRAPH_BIN env 已移除。初稿结论均经复核成立，无勘误；唯一存疑：模块注释提到的 CODEGRAPH_NO_DAEMON 在我读到的 McpClient::spawn 代码中未见实际设置点（已标注推断 + 验证途径）。

## 证据列表
1. explore 入口：ExploreTool::execute 校验 query 非空（缺失报 USER_INPUT），调 engine.call("explore", {query})，输出加 [codegraph explore] 前缀
   - src/tools/graph.rs:214-226（审计 #2）
2. CodegraphEngine::call 自动注入 projectPath 并把工具名拼成 codegraph_explore 转发 McpClient::call_tool
   - src/tools/graph.rs:104-110（审计 #2）
3. 启动链：bootlock acquire_guard 引导锁内按 --fresh_index 跑 index --force 或按 .codegraph 缺失跑 init（run_cli 300s 超时，graph.rs:25-67），drop(guard) 后 spawn codegraph serve --mcp 并 initialize 握手
   - src/tools/graph.rs:75-102（审计 #2）
4. McpClient 传输层：newline-delimited JSON-RPC over stdio，单请求 90s 超时（DEFAULT_TIMEOUT），通知/非法行忽略；isError 或空文本报 INDEX_BUILD_FAILED
   - src/mcp.rs:128-133（审计 #22）
   - src/mcp.rs:152-190（审计 #22）
   - src/mcp.rs:16（审计 #22）
   - src/mcp.rs:38-53（审计 #22）
5. 装配与错误分级：INDEX_LOCKED 判负整体退出；其他启动错误非致命降级（cg=None）并审计留痕 degraded
   - src/cli.rs:252-272（审计 #4）
6. 诚实工具面准入：graph_symbols_nonempty（repo_map_inputs 读 .codegraph/codegraph.db 符号）才注册 explore/callers/callees/impact/files 五工具，否则注入地形提示降级
   - src/cli.rs:273-296（审计 #4）
7. 配置：唯一键 graph.bin 默认 "codegraph"（配置链 CLI>项目>全局>默认合并于 config.rs:225-226）；--fresh_index 透传 force_reindex；CODEGRAPH_BIN env 已移除（graph.rs:70 注释）
   - src/config.rs:128-130（审计 #17）
   - src/config.rs:389-392（审计 #17）
   - src/cli.rs:43（审计 #4）
   - src/cli.rs:253（审计 #4）
8. 兄弟工具契约：CallersTool/CalleesTool/ImpactTool 经 cg_symbol_tool! 宏与 FilesTool 共享同一 CodegraphEngine
   - src/tools/graph.rs:130-183（审计 #2）
   - src/tools/graph.rs:229-265（审计 #2）
9. 生命周期回收：engine drop → McpClient Drop 杀 codegraph 子进程
   - src/mcp.rs:198-200（审计 #22）

## 死胡同
- grep 'fn call_tool|fn spawn|fn initialize' 以 plain 模式带管道符搜索 0 命中，改为 find_files 定位 src/mcp.rs 后直接通读
- grep 'CODEGRAPH_NO_DAEMON' 实际设置点：仅在 graph.rs:5 注释出现，正文设置位置未找到（标注推断）

## 置信度
high

## 统计
turns=7 · tool_calls=10 · duration=96476ms · tokens=99318


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
explore 是 codesleuth 只读检索工具面中的结构图探索工具：一次调用返回与 query（符号名/文件名/自然语言）相关的符号源码、调用路径与影响面，专为「开局看图」设计。实现上它是 codegraph 外部 MCP 服务的薄封装：ExploreTool::execute 校验 query 后经 CodegraphEngine.call 转发为 MCP 工具 codegraph_explore，引擎在启动时通过 codegraph CLI init/serve 建立连接。CLI 启动时仅在 codegraph 存在且符号索引非空时才注册该工具（诚实工具面策略），否则以地形提示告知降级用 find_files/grep/read。

## 证据列表
1. explore 工具定位：一次性返回相关符号源码 + 调用路径 + 影响面，供开局看图/流程问题使用，参数为 query（符号名/文件/自然语言）
   - src/tools/graph.rs:185-204（审计 #2）
2. 实现：execute 校验非空 query 后经 engine.call("explore", {query}) 转发，输出加 [codegraph explore] 前缀
   - src/tools/graph.rs:214-226（审计 #2）
3. 转发层：CodegraphEngine::call 自动附 projectPath，调用 McpClient.call_tool("codegraph_explore")
   - src/tools/graph.rs:104-110（审计 #2）
4. 上游依赖：CodegraphEngine::start 负责索引引导（init/force，带引导锁）并 spawn codegraph serve --mcp 完成握手，会话内五工具复用（模块头注释 graph.rs:1-5）
   - src/tools/graph.rs:69-102（审计 #2）
5. 注册与准入：CLI 启动时仅在 codegraph 存在且符号索引非空（graph_symbols_nonempty）时注册 ExploreTool 及 callers/callees/impact/files；否则不注册并注入地形提示引导改用 find_files/grep/read
   - src/cli.rs:273-295（审计 #11）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=4 · duration=13264ms · tokens=29786
