# 证据校验（报告与证据校验）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`02-evidence-validation-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 证据校验（报告与证据校验）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
证据校验功能保证侦察报告中的每条 evidence(file:lines) 都来自本会话真实读取过的文件：会话期间 EvidenceStore 记录所有观察到的路径及其审计 seq；模型 submit_report 时逐条用 cite_seq 匹配，未读过的引用直接打回并要求先 read 或删除；同时打回零 findings / 无证据 finding 的报告。通过的 evidence 会被回填 audit_seq，作为报告与审计日志的互查锚点。

## 证据列表
1. EvidenceStore 是会话内证据库：记录真实观察过的路径 → 首次观察的审计 seq，供 submit_report 校验与互查；read 工具走 observe_exact 精确入库
   - src/evidence.rs:26-37（审计 #4）
2. 工具输出处理时，harness 对每次工具调用结果调用 evidence.observe(output, seq)，read 调用额外 observe_exact 记录精确路径
   - src/harness.rs:370-378（审计 #6）
3. submit_report 时逐条 evidence 用 cite_seq(&file) 匹配；命中则回填 audit_seq，未命中进入 uncited 拒收列表
   - src/harness.rs:504-515（审计 #6）
4. 存在未读文件引用时返回错误「evidence 引用了本会话未读过的文件」，即拒收机制本体
   - src/harness.rs:541-545（审计 #6）
5. 附加打回：零 findings 但会话读过文件且无 dead_ends 时打回提炼；findings 无 evidence 时也打回（FINDING-012）
   - src/harness.rs:547-577（审计 #6）
6. 测试验证互查能力：evidence.audit_seq 可在审计日志中定位到对应的 tool_call 记录
   - src/harness.rs:903-913（审计 #6）
7. 数据结构：Report/Finding/Evidence 定义于 report.rs，Evidence 含 file/lines/audit_seq（审计互查锚点）
   - src/report.rs:9-16（审计 #2）
   - src/report.rs:18-22（审计 #2）

## 死胡同
- grep plain 模式检索 compound pattern 无命中，改用 regex 后命中

## 置信度
high

## 统计
turns=7 · tool_calls=8 · duration=33569ms · tokens=41520


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
证据校验由 EvidenceStore（harness 持有的 Mutex，evidence.rs:26-56）+ build_report 校验层（harness.rs:460-596）实现：工具输出经 observe/observe_exact 入库记录路径→首次审计 seq；submit_report 时逐条 evidence 用 cite_seq 逐字匹配，未读文件引用打回回流；另有零 findings/裸 finding/空 answer 打回；通过后回填 audit_seq 作报告↔审计互查锚点。功能无配置开关恒开启，仅受 context.model_context_tokens(默认1M)/compact_at_percent(默认60) 间接影响压缩时机；prose 降级路径完全绕过校验。初稿无实质错误。

## 证据列表
1. 入口链：cli.rs:353-365 构造 Harness 并 run；主循环 harness.rs:74 分发 tool_calls
   - src/cli.rs:353-365（审计 #27）
   - src/harness.rs:74-222（审计 #6）
2. 入库：工具输出 observe(output,seq)（harness.rs:369-378），read 额外 observe_exact（harness.rs:357-361）；observe 切词+path_like，observe_exact 逐字（evidence.rs:22-55）
   - src/harness.rs:357-378（审计 #6）
   - src/evidence.rs:22-55（审计 #2）
3. 校验层：cite_seq 匹配回填 audit_seq/uncited（harness.rs:504-516）；空 answer:538-540；uncited 拒收:541-546；零 findings 打回:547-563；裸 finding 打回:564-578；validate: report.rs:59-74
   - src/harness.rs:504-578（审计 #6）
   - src/report.rs:59-74（审计 #4）
4. 打回回流：reject_call 记 tool_rejected 并回流修正（harness.rs:261-269,598-613），被拒计熔断（阈值5，harness.rs:19,617-625）
   - src/harness.rs:261-269（审计 #6）
   - src/harness.rs:598-613（审计 #6）
5. 互查锚点：Evidence.audit_seq（report.rs:9-16）渲染「审计 #seq」（report.rs:89-94）；测试验证 seq 对应 tool_call（harness.rs:903-915）；拒后恢复测试 harness.rs:917-940
   - src/report.rs:9-16（审计 #4）
   - src/harness.rs:903-915（审计 #6）
6. 配置：证据校验无开关；ContextConfig{model_context_tokens,compact_at_percent} 默认 1_000_000/60（config.rs:114-118），文件覆盖点 config.rs:237-242；压缩只动 messages 不撤证据资格（推断，harness.rs:100-124）
   - src/config.rs:114-118（审计 #32）
   - src/config.rs:237-242（审计 #32）
   - src/harness.rs:100-124（审计 #6）
7. 边界：降级路径 degraded_prose 绕过全部校验（harness.rs:176-212, report.rs:46-57）；含空格/中文路径仅 read 精确存证可引用（测试 harness.rs:1064-1084）；submit 不去重可重提（harness.rs:223）
   - src/harness.rs:176-212（审计 #6）
   - src/report.rs:46-57（审计 #4）
   - src/harness.rs:1064-1084（审计 #6）

## 死胡同
- grep plain 'compact_percent|context_tokens' 零命中（config 键名为 compact_at_percent/model_context_tokens），regex 后命中
- callers Harness::new 只返回测试调用方，生产入口经 grep 定位 cli.rs:353
- 首次提交因引用未读过的 config.rs 被本功能拒收——已补读后重提，反向实证了 uncited 拒收机制

## 置信度
high

## 统计
turns=9 · tool_calls=13 · duration=132166ms · tokens=233317


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
证据校验功能保证侦察报告中的每条 evidence(file:lines) 都来自本会话真实读取过的文件：会话期间 EvidenceStore 记录所有观察到的路径及其审计 seq；模型 submit_report 时逐条用 cite_seq 匹配，未读过的引用直接打回并要求先 read 或删除；同时打回零 findings / 无证据 finding 的报告。通过的 evidence 会被回填 audit_seq，作为报告与审计日志的互查锚点。

## 证据列表
1. EvidenceStore 是会话内证据库：记录真实观察过的路径 → 首次观察的审计 seq，供 submit_report 校验与互查；read 工具走 observe_exact 精确入库
   - src/evidence.rs:26-37（审计 #4）
2. 工具输出处理时，harness 对每次工具调用结果调用 evidence.observe(output, seq)，read 调用额外 observe_exact 记录精确路径
   - src/harness.rs:370-378（审计 #6）
3. submit_report 时逐条 evidence 用 cite_seq(&file) 匹配；命中则回填 audit_seq，未命中进入 uncited 拒收列表
   - src/harness.rs:504-515（审计 #6）
4. 存在未读文件引用时返回错误「evidence 引用了本会话未读过的文件」，即拒收机制本体
   - src/harness.rs:541-545（审计 #6）
5. 附加打回：零 findings 但会话读过文件且无 dead_ends 时打回提炼；findings 无 evidence 时也打回（FINDING-012）
   - src/harness.rs:547-577（审计 #6）
6. 测试验证互查能力：evidence.audit_seq 可在审计日志中定位到对应的 tool_call 记录
   - src/harness.rs:903-913（审计 #6）
7. 数据结构：Report/Finding/Evidence 定义于 report.rs，Evidence 含 file/lines/audit_seq（审计互查锚点）
   - src/report.rs:9-16（审计 #2）
   - src/report.rs:18-22（审计 #2）

## 死胡同
- grep plain 模式检索 compound pattern 无命中，改用 regex 后命中

## 置信度
high

## 统计
turns=7 · tool_calls=8 · duration=33569ms · tokens=41520
