# MCP 进程生命周期回收（只读边界与安全）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`03-mcp-lifecycle-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# MCP 进程生命周期回收（只读边界与安全）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

MCP 进程生命周期回收（D011）让 codesleuth 以一次性会话方式启动外部 codegraph MCP 子进程使用其图查询工具，并在会话结束时由 Rust 的 Drop 语义自动 kill 子进程，保证无残留进程。入口是 `McpClient::spawn`（src/mcp.rs:68）与 `Drop for McpClient`（src/mcp.rs:198-205）；调用链为 `cli.rs` → `CodegraphEngine::start`（src/tools/graph.rs:71）→ `McpClient::spawn` + `initialize` 握手，会话期间通过 `CodegraphEngine::call` → `McpClient::call_tool` 做 JSON-RPC 请求，engine 被丢弃时 Drop 触发 `child.start_kill()` 回收。上游依赖 `crate::errors`、`crate::bootlock`（引导锁）与 tokio 进程/IO；下游消费方是 `src/tools/graph.rs` 的 CodegraphEngine 及其暴露给上层的 graph 工具，另有真机验收测试 `tests/graph_live.rs:12`。

## 证据列表

1. 该功能是最小 MCP over stdio 客户端：spawn server → newline-delimited JSON-RPC（initialize/tools_list/tools_call）→ Drop 回收，模块头即声明此定位
    - src/mcp.rs:1-4（审计 #2）
2. 入口与回收点：McpClient::spawn 以 piped stdin/stdout 启动子进程；Drop 实现调用 child.start_kill() 保证会话结束即回收 server 进程
    - src/mcp.rs:68-102（审计 #2）
    - src/mcp.rs:198-205（审计 #2）
3. 调用链：CodegraphEngine::start 经 bootlock 引导锁后 spawn McpClient 并握手；call 方法封装 tools/call 并自动附 projectPath
    - src/tools/graph.rs:71-102（审计 #19）
    - src/tools/graph.rs:105-110（审计 #19）
4. 上游调用方为 CLI 入口，从配置链传入可执行名（cfg.graph.bin）创建 CodegraphEngine 并置于 Arc 共享
    - src/cli.rs:249-257（审计 #24）
5. 真机验收测试覆盖完整生命周期：start → 五工具调用 → drop 回收（对应验收 D011）
    - tests/graph_live.rs:1-5（审计 #4）
    - tests/graph_live.rs:47-48（审计 #4）
6. 交互模块：依赖 crate::errors 错误体系（INDEX_NOT_AVAILABLE 等）与 crate::bootlock 仓库级引导锁（索引 init 串行化）
    - src/mcp.rs:6（审计 #2）
    - src/tools/graph.rs:78-88（审计 #19）

## 死胡同

无

## 置信度

high

## 统计

turns=9 · tool_calls=8 · duration=27903ms · tokens=61491

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
D011 MCP 进程生命周期回收 = codesleuth 自管 codegraph MCP 子进程：cli.rs 以 cfg.graph.bin（默认 "codegraph"）调 CodegraphEngine::start → bootlock 引导锁内 run_cli(init / index --force) → 放锁后 McpClient::spawn("serve --mcp", piped stdio) + initialize 握手 → 会话内五工具（explore/callers/callees/impact/files）经 CodegraphEngine::call 自动附 projectPath、加 codegraph_ 前缀做 newline-delimited JSON-RPC（单请求 90s 超时）→ engine 被 drop 时 Drop::drop 对 child try_lock 成功则 start_kill() 回收。初稿勘误：「CODEGRAPH_NO_DAEMON=1 保证独占进程」只存在于 graph.rs:5 注释，spawn 代码未设置该 env；初稿其余结论与代码吻合。

## 证据列表
1. 装配入口：cli.rs run 经 tokio runtime 调 CodegraphEngine::start(&repo_abs, &cfg.graph.bin, self.fresh_index)；INDEX_LOCKED 判负退出，其他错误非致命降级并审计留痕 degraded
   - src/cli.rs:252-254（审计 #11）
   - src/tools/graph.rs:71-72（审计 #4）
2. 引导段：acquire_guard 仓库级引导锁内，force 走 index --force --quiet，.codegraph 缺失走 init；放锁后才 spawn serve，serve 阶段不持锁
   - src/tools/graph.rs:78-88（审计 #4）
   - src/tools/graph.rs:83-87（审计 #4）
3. spawn + 握手：McpClient::spawn 以 piped stdin/stdout、stderr null 启动子进程并注入 CODEGRAPH_TELEMETRY=0 / DO_NOT_TRACK=1；initialize 发 protocolVersion 2025-03-26 + notifications/initialized
   - src/tools/graph.rs:89-97（审计 #4）
   - src/mcp.rs:68-102（审计 #2）
   - src/mcp.rs:105-114（审计 #2）
4. 会话期调用链：五工具 execute → CodegraphEngine::call（注入 projectPath、方法名前缀 codegraph_）→ call_tool → request（自增 id 匹配响应、通知/乱序忽略、DEFAULT_TIMEOUT=90s）
   - src/tools/graph.rs:105-110（审计 #4）
   - src/mcp.rs:128-133（审计 #2）
   - src/mcp.rs:152-190（审计 #2）
5. 工具面：cg_symbol_tool! 宏生成 Callers/Callees/Impact，另手写 Explore 与 Files；symbol_args 缺 symbol 报 USER_INPUT
   - src/tools/graph.rs:113-128（审计 #4）
   - src/tools/graph.rs:130-183（审计 #4）
   - src/tools/graph.rs:186-265（审计 #4）
6. 回收：Drop::drop 对 child try_lock 成功才 start_kill()——竞态下可能静默跳过 kill；且 start_kill 不 wait
   - src/mcp.rs:198-204（审计 #2）
7. 降级路径：codegraph 无符号索引（cg.db 空）时不注册五工具并注入地形提示；启动失败非致命降级 + degraded 审计
   - src/cli.rs:259-271（审计 #11）
   - src/cli.rs:275-295（审计 #11）
8. 配置：graph.bin 默认 "codegraph"（GraphConfig），经配置链 CLI>项目>全局>默认覆盖；CODEGRAPH_BIN env 已移除
   - src/config.rs:128-130（审计 #19）
   - src/config.rs:225-226（审计 #19）
   - src/tools/graph.rs:70（审计 #4）
9. 初稿勘误：CODEGRAPH_NO_DAEMON=1 只出现在模块注释（graph.rs:5），spawn 代码未设置该 env，实际独占性由每会话 spawn + Drop kill 保证
   - src/tools/graph.rs:5（审计 #4）
   - src/mcp.rs:72-73（审计 #2）
   - src/tools/graph.rs:27（审计 #4）
   - src/tools/graph.rs:42（审计 #4）
10. 错误处理：二进制缺失→INDEX_NOT_AVAILABLE+安装 hint；run_cli stderr 为空时给「codegraph unlock 残留锁」hint（2026-10-05 事故实测）；MCP 层 isError/空内容/JSON-RPC error 均映射 INDEX_BUILD_FAILED
   - src/tools/graph.rs:52-64（审计 #4）
   - src/mcp.rs:42-50（审计 #2）
   - src/mcp.rs:79-82（审计 #2）
11. 验收测试：tests/graph_live.rs #[ignore] 真机测试覆盖 start→explore/callers/files→drop 回收全生命周期
   - tests/graph_live.rs:10-48（审计 #16）

## 死胡同
- CODEGRAPH_NO_DAEMON 全库检索仅命中注释行 src/tools/graph.rs:5，无实际设置代码——作为初稿勘误记录而非死路
- Drop try_lock 竞态与 start_kill 不 wait 的运行时残留概率未做真机验证（静态推断）

## 置信度
high

## 统计
turns=9 · tool_calls=12 · duration=142746ms · tokens=162445


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

MCP 进程生命周期回收（D011）让 codesleuth 以一次性会话方式启动外部 codegraph MCP 子进程使用其图查询工具，并在会话结束时由 Rust 的 Drop 语义自动 kill 子进程，保证无残留进程。入口是 `McpClient::spawn`（src/mcp.rs:68）与 `Drop for McpClient`（src/mcp.rs:198-205）；调用链为 `cli.rs` → `CodegraphEngine::start`（src/tools/graph.rs:71）→ `McpClient::spawn` + `initialize` 握手，会话期间通过 `CodegraphEngine::call` → `McpClient::call_tool` 做 JSON-RPC 请求，engine 被丢弃时 Drop 触发 `child.start_kill()` 回收。上游依赖 `crate::errors`、`crate::bootlock`（引导锁）与 tokio 进程/IO；下游消费方是 `src/tools/graph.rs` 的 CodegraphEngine 及其暴露给上层的 graph 工具，另有真机验收测试 `tests/graph_live.rs:12`。

## 证据列表

1. 该功能是最小 MCP over stdio 客户端：spawn server → newline-delimited JSON-RPC（initialize/tools_list/tools_call）→ Drop 回收，模块头即声明此定位
    - src/mcp.rs:1-4（审计 #2）
2. 入口与回收点：McpClient::spawn 以 piped stdin/stdout 启动子进程；Drop 实现调用 child.start_kill() 保证会话结束即回收 server 进程
    - src/mcp.rs:68-102（审计 #2）
    - src/mcp.rs:198-205（审计 #2）
3. 调用链：CodegraphEngine::start 经 bootlock 引导锁后 spawn McpClient 并握手；call 方法封装 tools/call 并自动附 projectPath
    - src/tools/graph.rs:71-102（审计 #19）
    - src/tools/graph.rs:105-110（审计 #19）
4. 上游调用方为 CLI 入口，从配置链传入可执行名（cfg.graph.bin）创建 CodegraphEngine 并置于 Arc 共享
    - src/cli.rs:249-257（审计 #24）
5. 真机验收测试覆盖完整生命周期：start → 五工具调用 → drop 回收（对应验收 D011）
    - tests/graph_live.rs:1-5（审计 #4）
    - tests/graph_live.rs:47-48（审计 #4）
6. 交互模块：依赖 crate::errors 错误体系（INDEX_NOT_AVAILABLE 等）与 crate::bootlock 仓库级引导锁（索引 init 串行化）
    - src/mcp.rs:6（审计 #2）
    - src/tools/graph.rs:78-88（审计 #19）

## 死胡同

无

## 置信度

high

## 统计

turns=9 · tool_calls=8 · duration=27903ms · tokens=61491
