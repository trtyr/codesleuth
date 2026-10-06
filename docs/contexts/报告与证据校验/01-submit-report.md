# submit_report 结构化报告（报告与证据校验）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`01-submit-report-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# submit_report 结构化报告（报告与证据校验）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
submit_report 是 harness 内置的收敛工具：LLM 调查结束时提交版本化结构化报告（answer/findings/dead_ends/confidence/stats），harness 逐条校验 evidence 引用的文件必须是本会话真实观察过的（EvidenceStore），未读文件则拒绝并列出；不通过时回传拒因允许修正重提，通过后渲染为人类可读报告返回。若模型不走结构化提交，则以 degraded_prose 降级构造并诚实标注 degraded=true。

## 证据列表
1. submit_report 是 harness 内置工具（SUBMIT_TOOL），schema 要求 answer/findings/confidence 必填、confidence 限 high|medium|low
   - src/harness.rs:421-443（审计 #4）
2. 报告结构为 Report{report_schema_version=1, answer, findings, dead_ends, confidence, degraded, stats}，并有 validate() 校验置信度与空 answer
   - src/report.rs:7, 32-43（审计 #2）
   - src/report.rs:59-74（审计 #2）
3. 调用链：主循环识别 SUBMIT_TOOL → build_report 逐条用 EvidenceStore.cite_seq 校验证据 file 是否会话内真实读过，未读文件列入拒绝理由；被拒后 reject_call 允许修正重提，成功则 audit.record('report') 并 render_human 返回 RunOutcome
   - src/harness.rs:223-270（审计 #4）
   - src/harness.rs:460-544（审计 #4）
   - src/evidence.rs:26-33（审计 #25）
4. 降级路径：模型无工具调用时先引导提交，仍失败则 Report::degraded_prose 构造 degraded=true 的诚实降级报告
   - src/report.rs:1-2, 46-57（审计 #2）
   - src/harness.rs:176-200（审计 #4）
5. 与其他模块交互：上游依赖 read/审计流水线（EvidenceStore 记录路径→审计 seq，Evidence.audit_seq 作报告↔审计互查锚点）；旧证据账本 Ledger 已裁决砍除，findings 由报告本体承载
   - src/evidence.rs:26-30（审计 #25）
   - src/report.rs:13-15（审计 #2）
   - src/audit.rs:166-168（审计 #30）

## 死胡同
无

## 置信度
high

## 统计
turns=11 · tool_calls=10 · duration=38134ms · tokens=74226


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
submit_report 是 harness 内置唯一结构化收敛工具：CLI run_task_inner 构造 Harness 并 run（cli.rs:353-365），主循环识别 SUBMIT_TOOL 后经 parse_args → build_report 做 5 道校验（空 answer / 未读文件引用 / 零 findings 打回 / 裸 finding 打回 / validate() 置信度枚举），证据校验基于 EvidenceStore 的「路径→首见审计 seq」精确匹配；被拒经 reject_call 回传拒因允许修正重提（不走去重，但计 no_progress，5 步熔断）；通过则 audit.record("report") + render_human 返回 RunOutcome，cli.rs 持久化 md/json 到 <state_dir>/reports/ 并输出。模型无工具调用时先 prose 引导一轮，连续第二轮走 degraded_prose（degraded=true, confidence=low）。本功能无专属配置；context.model_context_tokens/compact_at_percent 只影响压缩时机。初稿勘误：Report 实际还有 task 字段（report.rs:35），初稿遗漏。

## 证据列表
1. 入口：run_task_inner 构造 Harness（传 model_context_tokens/compact_at_percent）并 block_on run
   - src/cli.rs:353-365（审计 #25）
2. submit_report schema 为 harness builtin_schemas，required: answer/findings/confidence，confidence enum high|medium|low；主循环识别 SUBMIT_TOOL 不走去重
   - src/harness.rs:423-444（审计 #4）
   - src/harness.rs:222-224（审计 #4）
3. build_report 五道校验：空answer拒(538)、未读文件拒(541)、零findings打回(553)、裸finding打回(564)、validate()(594)
   - src/harness.rs:460-596（审计 #4）
   - src/harness.rs:538-578（审计 #4）
4. 证据校验基于 EvidenceStore 路径→首见审计seq 精确匹配；read 走 observe_exact（FINDING-010），其他工具走切词 observe
   - src/evidence.rs:26-56（审计 #6）
   - src/harness.rs:356-378（审计 #4）
5. 拒绝经 reject_call 回传拒因并记审计 tool_rejected；拒绝计 no_progress，连续5步熔断(CS2099)
   - src/harness.rs:242-269（审计 #4）
   - src/harness.rs:598-613（审计 #4）
   - src/harness.rs:617-629（审计 #4）
6. 降级路径：无工具调用先 prose_steer 引导，连续第二轮 degraded_prose(degraded=true, confidence=low)
   - src/harness.rs:176-212（审计 #4）
   - src/report.rs:46-57（审计 #2）
7. Report 结构含 task 字段——初稿说仅 report_schema_version/answer/findings/dead_ends/confidence/degraded/stats，实际还有 task
   - src/report.rs:32-43（审计 #2）
8. 成功收敛：audit.record('report') → render_human → RunOutcome；cli.rs 持久化 reports/{session}.md/.json 并按 --json/--out 输出
   - src/cli.rs:423-454（审计 #25）
   - src/harness.rs:242-259（审计 #4）
9. 配置：本功能无专属配置项；model_context_tokens=1M / compact_at_percent=60 仅影响压缩时机；MAX_NO_PROGRESS_STREAK=5 等为编译期常量；confidence 缺省 medium
   - src/config.rs:115-118（审计 #42）
   - src/harness.rs:18-23（审计 #4）
   - src/harness.rs:474-478（审计 #4）
10. 互查锚点：Evidence.audit_seq 指向审计行可回查（测试 harness.rs:901-913）；旧 Ledger 已砍除，findings 由报告本体承载
   - src/audit.rs:166-168（审计 #28）
   - src/report.rs:13-15（审计 #2）

## 死胡同
- grep 'outcome.report|render_human' plain 零命中，换 'outcome' 后定位到 cli.rs 落点

## 置信度
high

## 统计
turns=12 · tool_calls=16 · duration=139516ms · tokens=259005


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
submit_report 是 harness 内置的收敛工具：LLM 调查结束时提交版本化结构化报告（answer/findings/dead_ends/confidence/stats），harness 逐条校验 evidence 引用的文件必须是本会话真实观察过的（EvidenceStore），未读文件则拒绝并列出；不通过时回传拒因允许修正重提，通过后渲染为人类可读报告返回。若模型不走结构化提交，则以 degraded_prose 降级构造并诚实标注 degraded=true。

## 证据列表
1. submit_report 是 harness 内置工具（SUBMIT_TOOL），schema 要求 answer/findings/confidence 必填、confidence 限 high|medium|low
   - src/harness.rs:421-443（审计 #4）
2. 报告结构为 Report{report_schema_version=1, answer, findings, dead_ends, confidence, degraded, stats}，并有 validate() 校验置信度与空 answer
   - src/report.rs:7, 32-43（审计 #2）
   - src/report.rs:59-74（审计 #2）
3. 调用链：主循环识别 SUBMIT_TOOL → build_report 逐条用 EvidenceStore.cite_seq 校验证据 file 是否会话内真实读过，未读文件列入拒绝理由；被拒后 reject_call 允许修正重提，成功则 audit.record('report') 并 render_human 返回 RunOutcome
   - src/harness.rs:223-270（审计 #4）
   - src/harness.rs:460-544（审计 #4）
   - src/evidence.rs:26-33（审计 #25）
4. 降级路径：模型无工具调用时先引导提交，仍失败则 Report::degraded_prose 构造 degraded=true 的诚实降级报告
   - src/report.rs:1-2, 46-57（审计 #2）
   - src/harness.rs:176-200（审计 #4）
5. 与其他模块交互：上游依赖 read/审计流水线（EvidenceStore 记录路径→审计 seq，Evidence.audit_seq 作报告↔审计互查锚点）；旧证据账本 Ledger 已裁决砍除，findings 由报告本体承载
   - src/evidence.rs:26-30（审计 #25）
   - src/report.rs:13-15（审计 #2）
   - src/audit.rs:166-168（审计 #30）

## 死胡同
无

## 置信度
high

## 统计
turns=11 · tool_calls=10 · duration=38134ms · tokens=74226
