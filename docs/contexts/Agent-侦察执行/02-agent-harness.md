# Agent 主循环 Harness（Agent 侦察执行）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`02-agent-harness-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# Agent 主循环 Harness（Agent 侦察执行）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

Harness（src/harness.rs）是 codesleuth 的 Agent 自主侦察主循环：接收用户任务后驱动「LLM 决策 → 工具调度 → 增量记账 → 收敛/熔断」的多轮循环，直到模型调用 submit_report 提交带证据校验的结构化报告（或 prose 降级），为调用方交付带审计留痕的 RunOutcome。入口为 src/cli.rs:353-365（run_task_inner 构造并 block_on agent.run），核心循环在 src/harness.rs:74 的 Harness::run。上游依赖：LLM 接入（src/llm.rs 的 LlmProvider）、工具注册表（src/tools/mod.rs 的 ToolRegistry，schemas() 注入工具面、get() 分发执行，src/harness.rs:130,331）、审计（src/audit.rs）、上下文压缩（src/context.rs，src/harness.rs:100-124）、首条消息导航图后缀（with_first_user_suffix，src/harness.rs:69）。收敛通道为内置 submit_report 工具（src/harness.rs:224-250 经 build_report 出 Report）；空转熔断 fuse_if_hit 在连续 5 步无进展时返回 LLM_FUSE（src/harness.rs:19,227,331-333），连续零增量注入转向指令（harness.rs:781-804），上下文超阈触发确定性压缩+handoff（harness.rs:100-124）。下游消费：RunOutcome 交回 cli.rs 渲染输出，Report/Evidence 由 src/report.rs、src/evidence.rs 承载。

## 证据列表

1. Harness 是 Agent 主循环：LLM 决策→工具调度→增量记账→收敛/熔断，收敛 = submit_report 或 prose 降级，对外产出 RunOutcome（report/answer/turns/tool_calls/audit_path）
    - src/harness.rs:1-3,74（审计 #2）
2. 入口在 CLI：run_task_inner 构造 Harness（provider/registry/audit/model/上下文参数）并 block_on agent.run(&task)
    - src/cli.rs:353-365（审计 #14）
    - src/harness.rs:46-65（审计 #2）
3. 主循环每轮：超阈值先 context 压缩+handoff，再组装 ChatRequest（注册工具 schemas + 内置 submit_report/recall schema）调 provider.chat
    - src/harness.rs:99-136（审计 #2）
4. 工具调度：循环遍历 resp.tool_calls，submit_report 走 build_report 收敛，普通工具经 self.tools.get() 分发执行
    - src/harness.rs:222-250（审计 #2）
    - src/harness.rs:331-333（审计 #2）
5. 熔断与转向：连续 5 步无进展（MAX_NO_PROGRESS_STREAK）经 fuse_if_hit 返回 LLM_FUSE；连续零增量注入「无新信息」转向指令
    - src/harness.rs:18-21（审计 #2）
    - src/harness.rs:227,331-333（审计 #2）
    - src/harness.rs:781-804（审计 #2）
6. 上游/下游交互：依赖 LlmProvider（src/llm.rs）、ToolRegistry（src/tools/mod.rs）、Audit（src/audit.rs）、context 压缩（src/context.rs）；RunOutcome 交回 cli.rs 渲染，模块全景见 docs/00-overview.md
    - src/cli.rs:353-360（审计 #14）
    - docs/00-overview.md:12（审计 #29）

## 死胡同

无

## 置信度

high

## 统计

turns=8 · tool_calls=11 · duration=36230ms · tokens=70038

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
Harness（src/harness.rs:74-419）是 codesleuth 的 Agent 自主侦察主循环：CLI run_task_inner（src/cli.rs:190）装配 config/provider/工具/审计后构造 Harness（cli.rs:353-364）并 block_on agent.run(task)（cli.rs:365）；主循环每轮执行「压缩检查（context.rs:31-82）→ 组装 ChatRequest（registry.schemas + 内置 submit_report/recall，harness.rs:126-136,423-458）→ provider.chat（harness.rs:136）→ tool_calls 分发（submit_report 收敛 / recall 钻取 / 普通工具 get+execute，harness.rs:222-417）→ 增量记账（EvidenceStore + Audit JSONL）」，收敛于 build_report 证据校验（harness.rs:460-596）或 prose 降级（harness.rs:177-212），空转经 fuse_if_hit 连续 5 步无进展熔断 LLM_FUSE=CS2099（harness.rs:616-629；errors.rs:40），RunOutcome 交回 cli.rs 渲染落盘 reports/{sid}.md|.json 并做 write_check 零写入自证（cli.rs:369-454）。配置链为 CLI>项目>全局>默认，环境变量层已移除（config.rs:1-3,173-199）；压缩阈值=窗口×%（默认 1M×60%，config.rs:115-118）。初稿结论全部核实无误。

## 证据列表
1. 完整调用链：Cli::run→run_task→run_task_inner 装配并 block_on Harness::run；run 内每轮循环为压缩检查→ChatRequest→provider.chat→tool_calls 分发
   - src/harness.rs:74-134（审计 #2）
   - src/harness.rs:222-417（审计 #2）
2. 入口装配：task/repo 校验、config::load、Audit::create、OpenAiProvider(重试 2 次)、read/fuzzy 必装、codegraph 按符号注册图工具否则注入地形提示、--vector 装配召回层与首条消息后缀
   - src/cli.rs:190-365（审计 #4）
3. 收敛通道 submit_report 与钻取工具 recall 为 harness 内置（builtin_schemas，harness.rs:423-458）；recall 经 Audit::read_range 按 seq 升序不重不漏回填
   - src/harness.rs:421-458（审计 #2）
   - src/context.rs:85-94（审计 #4）
   - src/tools/mod.rs:24-58（审计 #70）
4. build_report 证据校验四道闸：未读文件引用拒绝(harness.rs:541-546)、零 findings 打回提炼 FINDING-012(547-563)、零证据 finding 打回(564-578)、confidence/answer 校验(report.rs:59-74)；被拒允许修正重提且计入熔断
   - src/harness.rs:460-596（审计 #2）
5. 空转防御：MAX_NO_PROGRESS_STREAK=5 熔断 LLM_FUSE(CS2099，exit 3)，重复调用/非法 JSON/未知工具/工具错误/零增量均计入；ZERO_GAIN_STEER_THRESHOLD=2 注入转向指令
   - src/harness.rs:18-23（审计 #2）
   - src/harness.rs:616-629（审计 #2）
   - src/errors.rs:35-40（审计 #4）
   - src/harness.rs:394-402（审计 #2）
6. 上下文压缩：阈值=窗口×百分比(>100 封顶)，token 估算 chars/4 启发式；三段式 handoff(从 seq 1 起)→确定性 compact(KEEP_RECENT=6、消息对感知防孤儿 tool 消息)
   - src/harness.rs:99-124（审计 #2）
   - src/context.rs:25-82（审计 #4）
   - src/audit.rs:109-133（审计 #38）
7. 数据流落点：报告写 ~/.codesleuth/reports/{sid}.md|.json、审计 ~/.codesleuth/audit/{sid}.jsonl(64MB 单代轮转)、Evidence.audit_seq 与审计行互查；write_check 零写入自证
   - src/cli.rs:423-454（审计 #4）
   - src/cli.rs:369-421（审计 #4）
   - src/report.rs:32-76（审计 #4）
   - src/audit.rs:62-103（审计 #38）
8. 配置：四层链 CLI>项目(.codesleuth/config.toml)>全局(~/.codesleuth/config.toml)>默认，环境变量层已移除；默认 model=gpt-4o-mini、context_tokens=1M、compact_at_percent=60、thinking_disabled=true
   - src/config.rs:1-3（审计 #4）
   - src/config.rs:92-133（审计 #4）
   - src/cli.rs:353-360（审计 #4）

## 死胡同
- src/main.rs 未读（main 入口为推断，源自 cli.rs:96-98 注释）
- src/llm.rs 未全文阅读，LlmProvider/OpenAiProvider 内部实现未核实（接口经 harness.rs:136 与 cli.rs:231-236 侧面确认）

## 置信度
high

## 统计
turns=19 · tool_calls=28 · duration=150050ms · tokens=640030


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

Harness（src/harness.rs）是 codesleuth 的 Agent 自主侦察主循环：接收用户任务后驱动「LLM 决策 → 工具调度 → 增量记账 → 收敛/熔断」的多轮循环，直到模型调用 submit_report 提交带证据校验的结构化报告（或 prose 降级），为调用方交付带审计留痕的 RunOutcome。入口为 src/cli.rs:353-365（run_task_inner 构造并 block_on agent.run），核心循环在 src/harness.rs:74 的 Harness::run。上游依赖：LLM 接入（src/llm.rs 的 LlmProvider）、工具注册表（src/tools/mod.rs 的 ToolRegistry，schemas() 注入工具面、get() 分发执行，src/harness.rs:130,331）、审计（src/audit.rs）、上下文压缩（src/context.rs，src/harness.rs:100-124）、首条消息导航图后缀（with_first_user_suffix，src/harness.rs:69）。收敛通道为内置 submit_report 工具（src/harness.rs:224-250 经 build_report 出 Report）；空转熔断 fuse_if_hit 在连续 5 步无进展时返回 LLM_FUSE（src/harness.rs:19,227,331-333），连续零增量注入转向指令（harness.rs:781-804），上下文超阈触发确定性压缩+handoff（harness.rs:100-124）。下游消费：RunOutcome 交回 cli.rs 渲染输出，Report/Evidence 由 src/report.rs、src/evidence.rs 承载。

## 证据列表

1. Harness 是 Agent 主循环：LLM 决策→工具调度→增量记账→收敛/熔断，收敛 = submit_report 或 prose 降级，对外产出 RunOutcome（report/answer/turns/tool_calls/audit_path）
    - src/harness.rs:1-3,74（审计 #2）
2. 入口在 CLI：run_task_inner 构造 Harness（provider/registry/audit/model/上下文参数）并 block_on agent.run(&task)
    - src/cli.rs:353-365（审计 #14）
    - src/harness.rs:46-65（审计 #2）
3. 主循环每轮：超阈值先 context 压缩+handoff，再组装 ChatRequest（注册工具 schemas + 内置 submit_report/recall schema）调 provider.chat
    - src/harness.rs:99-136（审计 #2）
4. 工具调度：循环遍历 resp.tool_calls，submit_report 走 build_report 收敛，普通工具经 self.tools.get() 分发执行
    - src/harness.rs:222-250（审计 #2）
    - src/harness.rs:331-333（审计 #2）
5. 熔断与转向：连续 5 步无进展（MAX_NO_PROGRESS_STREAK）经 fuse_if_hit 返回 LLM_FUSE；连续零增量注入「无新信息」转向指令
    - src/harness.rs:18-21（审计 #2）
    - src/harness.rs:227,331-333（审计 #2）
    - src/harness.rs:781-804（审计 #2）
6. 上游/下游交互：依赖 LlmProvider（src/llm.rs）、ToolRegistry（src/tools/mod.rs）、Audit（src/audit.rs）、context 压缩（src/context.rs）；RunOutcome 交回 cli.rs 渲染，模块全景见 docs/00-overview.md
    - src/cli.rs:353-360（审计 #14）
    - docs/00-overview.md:12（审计 #29）

## 死胡同

无

## 置信度

high

## 统计

turns=8 · tool_calls=11 · duration=36230ms · tokens=70038
