# 结构图工具集（结构图与MCP）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-07

# 侦察报告

任务：【执行纪律——逐条遵守，违反任何一条即为失败任务】
1. 只输出被要求的内容本身；不要解释、不要前言、不要总结你的做法。
2. 不要复述、改写或摘抄本任务书的任何句子（包括本节）。
3. 严格按规定的结构和标题输出；标题、列名逐字照写，不要自创。
4. 事实必须来自你实际读到的代码，每条带 文件路径:行号；没把握的写（推断）。
5. 禁止 TODO、禁止「待补充」、禁止用省略号省略内容、禁止留空节。
6. 被要求的内容全部输出完后立即停止，不要追加任何文字。


【任务】你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 结构图工具集（结构图与MCP）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
结构图工具集通过本地 codegraph CLI 的 MCP 接口（`codegraph serve --mcp`）向上层 agent 暴露五个 Tool：explore / callers / callees / impact / files。CodegraphEngine 在 cli.rs 启动时完成 init（或 force index）→ spawn MCP 子进程 → initialize 握手，并独占进程（`CODEGRAPH_NO_DAEMON=1`，Drop 回收）；五个 Tool 的 execute 把参数归一化后统一走 CodegraphEngine::call（自动附 projectPath）→ McpClient::call_tool，回填 `[codegraph <tool>] <label]\n<out>`。索引为空或非代码仓时按 QA FINDING-009 诚实降级，不注册图工具并给 agent 地形提示。

## 证据列表
1. 模块定位：结图层以 MCP 方式接入 codegraph，生命周期归 codesleuth，每会话独占进程。
   - src/tools/graph.rs:1-5（审计 #2）
2. CodegraphEngine::start 入口负责索引引导（init/force，可执行名由配置链传入）后 spawn `codegraph serve --mcp` 并完成 initialize 握手。
   - src/tools/graph.rs:69-102（审计 #2）
3. CodegraphEngine::call 统一为 MCP 调用，自动追加 projectPath 并把工具名加 `codegraph_` 前缀。
   - src/tools/graph.rs:104-110（审计 #2）
4. cg_symbol_tool! 宏生成 CallersTool / CalleesTool / ImpactTool 三个反向边/正向边/影响面工具，共享参数 schema（symbol 必填，limit/depth 可选）。
   - src/tools/graph.rs:130-183（审计 #2）
5. ExploreTool 接受自然语言 query（结构图一把梭），FilesTool 接受可选 glob filter，分别独立实现 Tool trait。
   - src/tools/graph.rs:185-265（审计 #2）
6. 上游注册点：cli.rs 在索引非空时把 5 个工具装入 registry；非代码仓/索引空时按 QA FINDING-009 不注册并给出地形提示，引导 agent 用 find_files/grep/read。
   - src/cli.rs:255-301（审计 #15）
7. 下游依赖：McpClient::spawn 起子进程与 JSON-RPC 收发，call_tool 执行 tools/call；Drop 回收 server 进程。
   - src/mcp.rs:128-133（审计 #4）
8. 真机验收：tests/graph_live.rs 默认 ignore，演示 init→spawn→5 工具真实调用→Drop 全链路。
   - tests/graph_live.rs:1-48（审计 #4）

## 死胡同
- grep 用复合字面量模式首次 0 命中（被字面 OR 与字符类吞掉），改用正则重试后在 src/cli.rs 命中 5 行注册点。
- src/vector/chunk.rs 出现 codegraph sqlite 适配器（symbols_from_codegraph），与结构图工具集不直接相关，未纳入 findings。

## 置信度
high

## 统计
turns=6 · tool_calls=6 · duration=30019ms · tokens=58176


【初稿内容结束】

严格按以下六节输出，节标题逐字照写：

「## 功能定位」：这个功能是干什么的，2-4 句。
「## 完整调用链」：从入口到落点的每一步，谁调用谁，逐条列出，每条带 path:line。
「## 数据流」：输入从哪来、经过什么处理、落到哪（表 / 文件 / 下游服务）。
「## 配置与开关」：受哪些配置项 / 环境变量影响，默认行为是什么。
「## 边界与坑」：错误处理、特殊分支、已知限制、容易误用的地方。
「## 交互契约」：上下游接口 / 事件 / 共享存储，逐项列出。

配图（第 7 项，必须做）：按文末【画图技能规格】为本功能画一张组件/调用关系架构图，
完整单文件 HTML 放在单独围栏块中（其他正文不要放进这个块）：

块内容硬性要求：以 <!DOCTYPE html> 开头、含内联 <svg>、含多个组件矩形与带标签箭头、
可直接在浏览器打开渲染；禁止省略号、占位文本、说明性文字或对任务的复述。
组件名用真实模块/文件名；省略规格中「Export Toolbar / 导出工具栏」相关要求
（文档内嵌图不需要），其余设计系统规则全部遵守。

【五条红线，违反即失败】
A. 在初稿基础上展开，不是另起炉灶；初稿说错的地方明确写「初稿说 X，实际是 Y」。
B. 每个结论带 path:line；没读懂的显式标（推断）+ 最简验证途径。
C. 凭据只写变量名，禁止出现真实 token/密码/密钥。
D. 禁止 TODO / 待补充 / 空节 / 省略号省略内容。
E. 六节 + 配图全部输出完后立即停止。

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
结构图工具集是 codesleuth 对本地 codegraph CLI 的 MCP over stdio 封装：CLI 启动期在 bootlock 下完成索引引导（init / index --force），spawn `codegraph serve --mcp` 子进程，initialize 握手后把 5 个 Tool（explore / callers / callees / impact / files）注入 ToolRegistry，agent 每轮 LLM 响应按 name 查表 → tool.execute → CodegraphEngine::call（自动附 projectPath 并加 `codegraph_` 前缀）→ McpClient call_tool → newline JSON-RPC → 服务端响应 → 文本回填为 `[codegraph <tool>] <label>\n<out>`。索引空或非代码仓时按 QA FINDING-009 不注册 5 工具并向 first_suffix 推「地形提示」（find_files + grep + read），引导锁竞争败者 INDEX_LOCKED 判负退出（D014），其它失败走 audit "degraded" 降级留痕。会话结束 src/cli.rs:373 显式 drop(cg) 触发 McpClient::drop → start_kill 回收 server 子进程。初稿在 `CODEGRAPH_NO_DAEMON=1` 上有误——实际只存在于 src/tools/graph.rs:1-5, 70 的文档注释，代码侧无任何 .env() 调用，独占进程靠 Arc 所有权 + Drop 实现。

## 证据列表
1. CodegraphEngine 在 start_with_bin（src/tools/graph.rs:75-102）走 bootlock 引导 → init/force 索引（run_cli，src/tools/graph.rs:25-67，300s 超时）→ drop(guard) 放锁 → McpClient::spawn(bin, ["serve","--mcp"], root)（src/mcp.rs:68-102）→ client.initialize()（src/mcp.rs:104-114，protocolVersion=2025-03-26）→ 构造 CodegraphEngine {client, project_path}。
   - src/tools/graph.rs:75-102（审计 #2）
   - src/mcp.rs:68-114（审计 #4）
2. CodegraphEngine::call（src/tools/graph.rs:104-110）统一为 MCP 调用：注入 args["projectPath"]，把工具名加 `codegraph_` 前缀，转交 McpClient::call_tool。
   - src/tools/graph.rs:104-110（审计 #2）
   - src/mcp.rs:128-133（审计 #4）
3. cg_symbol_tool! 宏（src/tools/graph.rs:130-183）生成 CallersTool / CalleesTool / ImpactTool，共享参数 schema（symbol 必填，limit/depth 可选）；execute 走 symbol_args 归一化 → engine.call → 回填 [codegraph <tool>] <symbol>\n<out>。
   - src/tools/graph.rs:113-183（审计 #2）
4. ExploreTool（src/tools/graph.rs:185-227）独立实现 Tool trait，args.query 必填，调用 codegraph_explore；FilesTool（src/tools/graph.rs:229-265）独立实现，args.filter 可选 glob，调用 codegraph_files。
   - src/tools/graph.rs:185-265（审计 #2）
5. CLI 装配在 src/cli.rs:255-301：起 tokio runtime → CodegraphEngine::start（pass-through self.fresh_index + cfg.graph.bin）→ Err(INDEX_LOCKED) 显式 return Err 判负退出 → 其它错误 audit "degraded" 留痕置 None；再以 vector::repomap::repo_map_inputs(&cg_db) 判 graph_symbols_nonempty，非空时注册 5 工具，空时推「地形提示」进 first_suffix 兜底注入首条 user 消息。
   - src/cli.rs:255-301（审计 #12）
   - src/vector/repomap.rs:23-65（审计 #47）
6. McpClient（src/mcp.rs:1-239）实现 newline-delimited JSON-RPC：build_request 构造请求行，request 用 AtomicU64 自增 id + BufReader 读行 + 90s 超时（DEFAULT_TIMEOUT）；extract_tool_text 拼 result.content[*].text，isError=true 视为工具错误，空内容视为「符号不存在或索引未就绪」。
   - src/mcp.rs:16-204（审计 #4）
7. 生命周期回收：src/mcp.rs:198-204 impl Drop for McpClient 调 try_lock + start_kill 杀子进程；src/cli.rs:373 在 run_task 成功后显式 drop(agent); drop(cg) 触发回收（D011）。
   - src/mcp.rs:198-204（审计 #4）
   - src/cli.rs:372-373（审计 #12）
8. 配置入口：graph.bin 默认 "codegraph"（src/config.rs:142-144），通过 FileGraph.bin（src/config.rs:24-29）走全局+项目配置链 + 键表驱动 set/get（src/config.rs:457-464, 488-497），未知键 CS1012 含全 13 键名 hint。CODEGRAPH_BIN env 已移除（仅注释提及，src/tools/graph.rs:70、src/config.rs:24）。
   - src/config.rs:24-29（审计 #27）
   - src/config.rs:142-144（审计 #27）
   - src/config.rs:283-285（审计 #27）
   - src/config.rs:457-497（审计 #27）
   - src/tools/graph.rs:70-72（审计 #2）
9. 真机验收：tests/graph_live.rs:1-49 默认 #[ignore]，演示 init→spawn→5 工具真实调用（含 explore / callers / files）→ drop(engine) 收 server 全链路。单元测试覆盖：src/tools/graph.rs:271-292（missing_binary_is_structured_error + symbol_args_validation）、src/mcp.rs:207-238（request_line_shape + response_id_and_content_extraction）。
   - tests/graph_live.rs:1-49（审计 #14）
   - src/tools/graph.rs:267-293（审计 #2）
   - src/mcp.rs:207-239（审计 #4）
10. 错误码契约：INDEX_NOT_AVAILABLE=4010 / INDEX_BUILD_FAILED=4011 / INDEX_TIMEOUT=4014 / INDEX_LOCKED=4016（均 CS4xxx 段位 exit 5）；USER_INPUT=1001 用于缺参数（src/tools/graph.rs:118, 219）。CS4xxx 错误段位统一映射 exit 5 由 src/errors.rs:13-22 决定。
   - src/errors.rs:13-53（审计 #67）
11. 引导锁契约：bootlock（src/bootlock.rs:1-204）用 flock(.codesleuth/boot.lock) 跨进程互斥，300s 上限；acquire 三态 Won/Lost/超时统一以 CS4016 INDEX_LOCKED 判负退出；graph 引导（src/tools/graph.rs:78）+ vector 构建（src/cli.rs:162）+ 手动 index --vector（src/cli.rs:162）三处共用。
   - src/bootlock.rs:1-204（审计 #42）
   - src/tools/graph.rs:75-88（审计 #2）
   - src/cli.rs:162-166（审计 #12）
12. Tool trait 契约（src/tools/mod.rs:14-22）：Send + Sync，name() -> &'static str, description() -> String, parameters() -> Value（JSON Schema 对象）, async execute(args: Value) -> CsResult<String>。ToolRegistry（src/tools/mod.rs:24-59）提供 register/get/schemas/is_empty，由 Harness 主循环按 name 查表分发。
   - src/tools/mod.rs:14-59（审计 #32）
13. 初稿纠错：初稿说"`CODEGRAPH_NO_DAEMON=1` 保证每会话独占进程"。实际 src/tools/graph.rs:1-5, 70 仅为文档注释声明，代码侧无任何 .env("CODEGRAPH_NO_DAEMON", ...) 调用；独占进程靠 Arc 所有权 + McpClient::drop::start_kill（src/mcp.rs:198-204）+ src/cli.rs:373 显式 drop(cg) 实现。运行期实际只设 CODEGRAPH_TELEMETRY=0（src/tools/graph.rs:32、src/mcp.rs:72）与 DO_NOT_TRACK=1（src/tools/graph.rs:33、src/mcp.rs:73）。
   - src/tools/graph.rs:1-5（审计 #2）
   - src/tools/graph.rs:32-33（审计 #2）
   - src/tools/graph.rs:70-72（审计 #2）
   - src/mcp.rs:72-73（审计 #4）
   - src/mcp.rs:198-204（审计 #4）
   - src/cli.rs:372-373（审计 #12）

## 死胡同
- grep `graph.bin|graph_bin|codegraph_bin|CODEGRAPH_BIN|CODEGRAPH_NO_DAEMON` 全 plain 模式 0 命中，0/112 文件——平铺字符串不在索引里，切换 regex 模式后确认 CODEGRAPH_BIN 仅在 4 处历史归档与 config.rs:24 文档注释出现，CODEGRAPH_NO_DAEMON 仅在 graph.rs:1-5, 70 文档注释出现，无任何 .env() 调用点。
- grep `acquire_guard|bootlock|INDEX_LOCKED` 平铺 0/112 命中，regex 命中 0/9 文件——但通过 grep `bootlock|INDEX_LOCKED`（更宽泛）确认 src/bootlock.rs:24, 78, 116 与 src/cli.rs:265, 321 实际使用点；src/tools/graph.rs:78 的 acquire_guard 来自读到的源码。
- 未读取 harness.rs 内部对 tool.execute 返回 Err 后的具体熔断/喂回分支（推断由其它侦察 session 验证：grep `Err(e)` in src/harness.rs:362+），不影响本工具集结构结论。
- src/vector/chunk.rs 的 codegraph sqlite 适配器（symbols_from_codegraph）与本结构图工具集是同源但不同模块——chunk 走只读 SQLite 拉符号边界用于向量切块，不在 5 工具运行时路径上；已显式排除。

## 置信度
high

## 统计
turns=18 · tool_calls=32 · duration=174613ms · tokens=585166


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
结构图工具集通过本地 codegraph CLI 的 MCP 接口（`codegraph serve --mcp`）向上层 agent 暴露五个 Tool：explore / callers / callees / impact / files。CodegraphEngine 在 cli.rs 启动时完成 init（或 force index）→ spawn MCP 子进程 → initialize 握手，并独占进程（`CODEGRAPH_NO_DAEMON=1`，Drop 回收）；五个 Tool 的 execute 把参数归一化后统一走 CodegraphEngine::call（自动附 projectPath）→ McpClient::call_tool，回填 `[codegraph <tool>] <label]\n<out>`。索引为空或非代码仓时按 QA FINDING-009 诚实降级，不注册图工具并给 agent 地形提示。

## 证据列表
1. 模块定位：结图层以 MCP 方式接入 codegraph，生命周期归 codesleuth，每会话独占进程。
   - src/tools/graph.rs:1-5（审计 #2）
2. CodegraphEngine::start 入口负责索引引导（init/force，可执行名由配置链传入）后 spawn `codegraph serve --mcp` 并完成 initialize 握手。
   - src/tools/graph.rs:69-102（审计 #2）
3. CodegraphEngine::call 统一为 MCP 调用，自动追加 projectPath 并把工具名加 `codegraph_` 前缀。
   - src/tools/graph.rs:104-110（审计 #2）
4. cg_symbol_tool! 宏生成 CallersTool / CalleesTool / ImpactTool 三个反向边/正向边/影响面工具，共享参数 schema（symbol 必填，limit/depth 可选）。
   - src/tools/graph.rs:130-183（审计 #2）
5. ExploreTool 接受自然语言 query（结构图一把梭），FilesTool 接受可选 glob filter，分别独立实现 Tool trait。
   - src/tools/graph.rs:185-265（审计 #2）
6. 上游注册点：cli.rs 在索引非空时把 5 个工具装入 registry；非代码仓/索引空时按 QA FINDING-009 不注册并给出地形提示，引导 agent 用 find_files/grep/read。
   - src/cli.rs:255-301（审计 #15）
7. 下游依赖：McpClient::spawn 起子进程与 JSON-RPC 收发，call_tool 执行 tools/call；Drop 回收 server 进程。
   - src/mcp.rs:128-133（审计 #4）
8. 真机验收：tests/graph_live.rs 默认 ignore，演示 init→spawn→5 工具真实调用→Drop 全链路。
   - tests/graph_live.rs:1-48（审计 #4）

## 死胡同
- grep 用复合字面量模式首次 0 命中（被字面 OR 与字符类吞掉），改用正则重试后在 src/cli.rs 命中 5 行注册点。
- src/vector/chunk.rs 出现 codegraph sqlite 适配器（symbols_from_codegraph），与结构图工具集不直接相关，未纳入 findings。

## 置信度
high

## 统计
turns=6 · tool_calls=6 · duration=30019ms · tokens=58176
