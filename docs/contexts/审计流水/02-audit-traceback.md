# 报告↔审计双向回溯（审计流水）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`02-audit-traceback-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 报告↔审计双向回溯（审计流水）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

报告↔审计双向回溯：每次 read/工具调用经 Audit.record 写入审计 JSONL 时返回全局单调 seq，EvidenceStore 记录「路径→首次观察 seq」，submit_report 组装 Evidence 时回填 audit_seq，使每条报告证据都能用 seq 在审计文件中定位到那次工具调用原文；反向也可由审计行回查报告。价值：报告证据可审计、可验证、不凭空捏造。

## 证据列表

1. Evidence 结构含 audit_seq 字段，注释明确为「报告↔审计互查锚点」
    - src/report.rs:9-16（审计 #4）
2. EvidenceStore 以路径→首次观察 seq 记账（observe_exact/cite_seq），submit_report 组装时回填 Evidence.audit_seq
    - src/evidence.rs:26-37（审计 #20）
    - src/harness.rs:504-514（审计 #12）
3. Audit.record 写审计行返回单调 seq；read_range 按 seq 范围读回原文不重不漏
    - src/audit.rs:114-163（审计 #2）
4. 测试验证 evidence.audit_seq 在审计文件中命中 kind=tool_call 行，构成正向回溯
    - src/harness.rs:903-913（审计 #12）
5. 评估硬门 G2 强制每条 evidence 带 audit_seq 且能回溯到工具调用行，缺失或无法回溯即判失败
    - scripts/eval/hardgate.py:43-52（审计 #25）
6. handoff/recall 机制复用同一审计 seq 体系（seq 从 1 起号、按范围 recall），是 seq 的另一消费方
    - src/context.rs:125-128（审计 #27）

## 死胡同

无

## 置信度

high

## 统计

turns=8 · tool_calls=10 · duration=26271ms · tokens=69603

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
报告↔审计双向回溯功能链完整核实：Audit::record 在锁内取单调 seq 写 JSONL 返回该 seq（src/audit.rs:136-163）；harness 工具执行后 EvidenceStore.observe/observe_exact 记录 路径→首次观察 seq（src/harness.rs:350-378, src/evidence.rs:26-55）；submit_report → build_report 用 cite_seq 回填 Evidence.audit_seq，未命中列入 uncited 拒绝整份报告（src/harness.rs:460-596）；下游消费：recall 经 Audit::read_range 按 seq 区间升序读回（src/harness.rs:274-297, src/audit.rs:115-133）、eval 硬门 G2 强制每条 evidence.audit_seq 命中 kind∈{tool_call,tool_result} 审计行（scripts/eval/hardgate.py:12,43-52）、render_human 渲染「审计 #N」（src/report.rs:89-94）。审计落盘 ~/.codesleuth/audit/{session_id}.jsonl（Audit::create，64MB 单代轮转，src/audit.rs:70-103；装配 src/cli.rs:229,353-365），报告持久化至 reports/{session_id}.json 含 audit_seq（src/cli.rs:428-436）。初稿结论全部成立；补充：G2 合法 kind 集合含 tool_result，且 audit_seq 语义是「首次观察」seq（or_insert）。

## 证据列表
1. Evidence 结构含可选 audit_seq 字段，注释明确为报告↔审计互查锚点；degraded 时为 None
   - src/report.rs:9-16（审计 #6）
   - src/report.rs:46-57（审计 #6）
2. Audit.record 在 Mutex 锁内用 AtomicU64 fetch_add 取号并写盘，返回单调 seq；flush 失败仅 warn（审计行可能丢）
   - src/audit.rs:136-163（审计 #2）
3. Audit.create 落盘 ~/.codesleuth/audit/{session_id}.jsonl，64MB 单代轮转 .old，轮转后旧代 recall 不可达（已知限制）
   - src/audit.rs:70-103（审计 #2）
   - src/cli.rs:227-229（审计 #38）
4. 工具执行前 audit.record('tool_call') 取 seq；成功后 record('tool_result', {tool_call_seq})；evidence.observe 记账 + read 走 observe_exact 精确路径
   - src/harness.rs:350-378（审计 #9）
5. EvidenceStore 以 路径→首次观察 seq 记账（or_insert 保首次），cite_seq 提供查询
   - src/evidence.rs:26-55（审计 #4）
6. build_report 逐条 evidence 用 cite_seq 回填 audit_seq；未命中列入 uncited 整报告拒绝；另有零 findings 打回与零证据 finding 打回
   - src/harness.rs:504-546（审计 #9）
   - src/harness.rs:547-578（审计 #9）
7. recall 内置工具：主循环 0.5 步拦截 → Audit.read_range 全文件扫描、闭区间 [from,to]、seq 升序返回 → context.recall_message 带 #seq 标注回填；recall 自身也写审计
   - src/harness.rs:274-297（审计 #9）
   - src/audit.rs:115-133（审计 #2）
   - src/context.rs:85-94（审计 #11）
8. 压缩 handoff 以 last_seq 定范围、起点硬编码 1（seq 从 1 起号），是同一 seq 体系的另一消费方
   - src/harness.rs:100-124（审计 #9）
   - src/context.rs:47-54（审计 #11）
9. eval 硬门 G2 强制每条 evidence.audit_seq 命中 kind∈{tool_call,tool_result} 的审计行，缺失即 FAIL；G4 判 degraded 不收敛
   - scripts/eval/hardgate.py:12（审计 #22）
   - scripts/eval/hardgate.py:43-52（审计 #22）
   - scripts/eval/hardgate.py:62-64（审计 #22）
   - scripts/eval/hardgate.py:69-84（审计 #22）
10. 单测验证正向回溯：evidence.audit_seq 对应审计行恰为那次 tool_call（kind/name 断言）
   - src/harness.rs:895-914（审计 #9）
11. render_human 渲染「file:lines（审计 #N）」；报告持久化到 ~/.codesleuth/reports/{session_id}.{md,json} 含 audit_seq
   - src/report.rs:89-94（审计 #6）
   - src/cli.rs:428-447（审计 #38）
12. submit_report schema 对 LLM 只要求 file+lines（audit_seq 由 harness 回填，模型不可伪造）
   - src/harness.rs:423-458（审计 #9）
13. 会话结束后 write_check 经 audit::append_line（全文件 max_seq+1 续号）追加到同一审计文件，保持单调
   - src/audit.rs:21-51（审计 #2）
   - src/cli.rs:379-387（审计 #38）
14. 配置面：压缩阈值=model_context_tokens(默认1M)×compact_at_percent(默认60, 封顶100)，四层配置链无环境变量层，间接决定 recall 可用性；64MB 轮转与熔断/转向阈值为编译期常量
   - src/context.rs:31-33（审计 #11）
   - src/cli.rs:353-360（审计 #38）
   - src/audit.rs:79（审计 #2）

## 死胡同
- grep 'Audit::create|audit_path' plain 模式 0 命中（转义问题），换 regex 后命中；未影响结论

## 置信度
high

## 统计
turns=12 · tool_calls=18 · duration=143360ms · tokens=336465


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

报告↔审计双向回溯：每次 read/工具调用经 Audit.record 写入审计 JSONL 时返回全局单调 seq，EvidenceStore 记录「路径→首次观察 seq」，submit_report 组装 Evidence 时回填 audit_seq，使每条报告证据都能用 seq 在审计文件中定位到那次工具调用原文；反向也可由审计行回查报告。价值：报告证据可审计、可验证、不凭空捏造。

## 证据列表

1. Evidence 结构含 audit_seq 字段，注释明确为「报告↔审计互查锚点」
    - src/report.rs:9-16（审计 #4）
2. EvidenceStore 以路径→首次观察 seq 记账（observe_exact/cite_seq），submit_report 组装时回填 Evidence.audit_seq
    - src/evidence.rs:26-37（审计 #20）
    - src/harness.rs:504-514（审计 #12）
3. Audit.record 写审计行返回单调 seq；read_range 按 seq 范围读回原文不重不漏
    - src/audit.rs:114-163（审计 #2）
4. 测试验证 evidence.audit_seq 在审计文件中命中 kind=tool_call 行，构成正向回溯
    - src/harness.rs:903-913（审计 #12）
5. 评估硬门 G2 强制每条 evidence 带 audit_seq 且能回溯到工具调用行，缺失或无法回溯即判失败
    - scripts/eval/hardgate.py:43-52（审计 #25）
6. handoff/recall 机制复用同一审计 seq 体系（seq 从 1 起号、按范围 recall），是 seq 的另一消费方
    - src/context.rs:125-128（审计 #27）

## 死胡同

无

## 置信度

high

## 统计

turns=8 · tool_calls=10 · duration=26271ms · tokens=69603
