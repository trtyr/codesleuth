# prose 降级路径（报告与证据校验）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`03-prose-degradation-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# prose 降级路径（报告与证据校验）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
prose 降级路径是报告与证据校验体系的兜底机制：当模型连续两轮不调用 submit_report 结构化提交、只回纯文本时，harness 将其文本包装为 degraded=true、confidence=low 的降级 Report 返回，保证调用方始终拿到带统计的报告，且能明确区分"真收敛"与"散文兜底"（eval 硬门 G4 将 degraded 视为不收敛）。

## 证据列表
1. 功能：模型未走 submit_report 时，用其纯文本 prose 构造诚实标注 degraded=true 的降级报告，避免结构化提交失败导致无输出。
   - src/harness.rs:176-204（审计 #6）
   - src/report.rs:1-2（审计 #2）
2. 入口与关键文件：构造入口 Report::degraded_prose（src/report.rs:46-57），触发点在 harness 主循环（src/harness.rs:193-206），人类渲染附加降级说明（src/report.rs:108-115），测试覆盖 degraded_report_marks_itself 与 prose_falls_back_to_degraded_report（src/report.rs:173-179；src/harness.rs:870-881）。
   - src/report.rs:46-57（审计 #2）
   - src/harness.rs:193-206（审计 #6）
   - src/report.rs:108-115（审计 #2）
   - src/report.rs:173-179（审计 #2）
3. 运作：run 主循环检测 tool_calls 为空时计数 prose_streak——首轮将 prose 入史并重新引导 submit_report，连续第二轮则调用 Report::degraded_prose 构造降级报告并向审计记录 answer(degraded=true) 后返回 RunOutcome。
   - src/harness.rs:177-203（审计 #6）
   - src/harness.rs:870-881（审计 #6）
4. 交互：上游依赖 LLM 轮次与审计模块（self.audit.record 记录 prose_steer/answer 事件，src/harness.rs:189-203）；下游被 eval 硬门消费——G4 将 degraded=true 判为不收敛（scripts/eval/hardgate.py:62-64）。
   - src/harness.rs:189-203（审计 #6）
   - scripts/eval/hardgate.py:62-64（审计 #22）

## 死胡同
无

## 置信度
high

## 统计
turns=7 · tool_calls=9 · duration=27759ms · tokens=36725


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
prose 降级路径是 Harness::run（src/harness.rs:74）主循环的兜底收敛通道：模型一轮 tool_calls 为空时 prose_streak 递增（harness.rs:177-178），首轮将 prose 入史并注入 submit_report 引导语、审计记 prose_steer（harness.rs:179-191）；连续第二轮（无工具轮归零，:218-220）则调用 Report::degraded_prose(task, answer, stats) 构造 degraded=true、confidence=low、findings 空（report.rs:46-57）的报告，审计记 answer(degraded=true)（harness.rs:201-204），返回 RunOutcome；CLI 落盘 reports/{session_id}.md/.json（cli.rs:428-436），eval 硬门 G4 判 degraded=true 为不收敛（hardgate.py:62-64）。无专用配置开关，阈值与引导语硬编码；prose 分支不熔断、降级报告不过 validate()。初稿无事实错误。

## 证据列表
1. 调用链：main → Cli::run → run_task_inner 装配 Harness 并 block_on run(task) → 主循环 provider.chat → tool_calls 为空分支（引导/降级）→ RunOutcome → 报告落盘与 stdout
   - src/main.rs:5-10（审计 #42）
   - src/harness.rs:74,177-212,218-220（审计 #7）
   - src/cli.rs:96,353-365（审计 #24）
2. 降级构造：Report::degraded_prose 固定 degraded=true、confidence=low、findings/dead_ends 空、schema_version=1；render_human 在证据列表与置信度行附加降级说明
   - src/report.rs:7,46-57,108-115,173-179（审计 #4）
   - src/harness.rs:193-211,870-881（审计 #7）
3. prose 分支不涉及 no_progress 熔断（fuse 只出现在 submit_report 非法参数路径）；中间有工具轮则 prose_streak 归零重新给引导机会
   - src/harness.rs:176-212（审计 #7）
   - src/harness.rs:223-249（审计 #7）
4. 降级报告不调用 validate()（空 answer 照样返回），validate 仅在 submit_report 路径生效（推断，验证：grep harness.rs 中 validate 调用点）
   - src/harness.rs:67（审计 #7）
   - src/report.rs:67-72（审计 #4）
5. 数据流落点：审计 JSONL answer(degraded=true) 事件行、{state_dir}/reports/{session_id}.md/.json、--out、stdout；下游 eval 硬门 G4（配合 G2 空 findings 无 audit_seq）将降级判为不收敛
   - src/harness.rs:189-204（审计 #7）
   - src/cli.rs:428-447（审计 #24）
   - scripts/eval/hardgate.py:62-64,43-52（审计 #9）
6. 无专用配置项/环境变量：prose_streak 阈值（≤1 引导）与引导文案硬编码；注意 cli.rs 中另有同名审计事件 kind 'degraded'（repo_map 失败留痕），与 prose 降级报告无关
   - src/harness.rs:179,186-188（审计 #7）
   - src/cli.rs:345-348（审计 #24）
7. 首轮 prose 必须先入史再引导：丢上下文且部分网关拒收连续 User 消息（P005 R6.1）；工具轮归零设计防误降级（P004 T2.2）
   - src/harness.rs:180-188,218-220（审计 #7）

## 死胡同
无

## 置信度
high

## 统计
turns=11 · tool_calls=16 · duration=171338ms · tokens=191381


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
prose 降级路径是报告与证据校验体系的兜底机制：当模型连续两轮不调用 submit_report 结构化提交、只回纯文本时，harness 将其文本包装为 degraded=true、confidence=low 的降级 Report 返回，保证调用方始终拿到带统计的报告，且能明确区分"真收敛"与"散文兜底"（eval 硬门 G4 将 degraded 视为不收敛）。

## 证据列表
1. 功能：模型未走 submit_report 时，用其纯文本 prose 构造诚实标注 degraded=true 的降级报告，避免结构化提交失败导致无输出。
   - src/harness.rs:176-204（审计 #6）
   - src/report.rs:1-2（审计 #2）
2. 入口与关键文件：构造入口 Report::degraded_prose（src/report.rs:46-57），触发点在 harness 主循环（src/harness.rs:193-206），人类渲染附加降级说明（src/report.rs:108-115），测试覆盖 degraded_report_marks_itself 与 prose_falls_back_to_degraded_report（src/report.rs:173-179；src/harness.rs:870-881）。
   - src/report.rs:46-57（审计 #2）
   - src/harness.rs:193-206（审计 #6）
   - src/report.rs:108-115（审计 #2）
   - src/report.rs:173-179（审计 #2）
3. 运作：run 主循环检测 tool_calls 为空时计数 prose_streak——首轮将 prose 入史并重新引导 submit_report，连续第二轮则调用 Report::degraded_prose 构造降级报告并向审计记录 answer(degraded=true) 后返回 RunOutcome。
   - src/harness.rs:177-203（审计 #6）
   - src/harness.rs:870-881（审计 #6）
4. 交互：上游依赖 LLM 轮次与审计模块（self.audit.record 记录 prose_steer/answer 事件，src/harness.rs:189-203）；下游被 eval 硬门消费——G4 将 degraded=true 判为不收敛（scripts/eval/hardgate.py:62-64）。
   - src/harness.rs:189-203（审计 #6）
   - scripts/eval/hardgate.py:62-64（审计 #22）

## 死胡同
无

## 置信度
high

## 统计
turns=7 · tool_calls=9 · duration=27759ms · tokens=36725
