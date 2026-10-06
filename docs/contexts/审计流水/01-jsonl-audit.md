# 全量 JSONL 审计（审计流水）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`01-jsonl-audit-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 全量 JSONL 审计（审计流水）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
全量 JSONL 审计：按 seq 单调递增把会话事件追加写入 {session_id}.jsonl，作为全量原文恢复源；harness/writeguard/cli 各环节调用 Audit::record 写入，压缩 handoff 用 last_seq 记录范围，recall 经 read_range 按区间读回。

## 证据列表
1. 审计 JSONL 是全量原文恢复源，支持按 seq 区间读回（read_range 升序不重不漏）
   - src/audit.rs:1-3（审计 #2）
   - src/audit.rs:114-133（审计 #2）
2. Audit::create 建每会话文件并做 64MB 单代轮转；record 在锁内取 seq+写盘保证单调
   - src/audit.rs:63-103（审计 #2）
   - src/audit.rs:135-163（审计 #2）
3. harness::run 压缩前用 last_seq 定区间并写 compaction 事件；writeguard::write_check 用 append_line 追加宿主级事件
   - src/harness.rs:99-124（审计 #16）
   - src/audit.rs:20-51（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=6 · tool_calls=6 · duration=25619ms · tokens=33571


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
全量 JSONL 审计是 codesleuth 的全量原文恢复源：入口 main 生成 session_id（纳秒+pid hex），CLI run_task_inner 在 ~/.codesleuth/audit/ 下创建 {session_id}.jsonl（Audit::create，64MB 单代轮转 .old）；Agent 主循环 Harness::run 的每类事件（llm/llm_error/tool_call/tool_result/tool_rejected/prose_steer/answer/report/compaction_begin/compaction/recall/degraded/fuse）经 Audit::record 在锁内取号（AtomicU64+Mutex File）追加写入并即时 flush，保证 seq 单调；任务结束后 writeguard 零写入自证经 audit::append_line（读全文件取 max_seq+1）追加 write_check 行；压缩 handoff 用 last_seq 定区间 [1, last_seq]，LLM 通过内置 recall 工具经 read_range 按 seq 区间升序读回被驱逐原文；Evidence.audit_seq 构成报告↔审计双向互查锚点（评估硬门 G2 强制校验）。初稿结论方向正确，细节升级见 findings。

## 证据列表
1. 调用链起点：main.rs 生成 session_id（audit::new_session_id，纳秒时间+pid hex）→ init_tracing → Cli::run(session_id)；审计与日志共用同一会话身份
   - src/main.rs:5-11（审计 #37）
   - src/audit.rs:53-60（审计 #2）
   - src/cli.rs:226-229（审计 #7）
2. Audit::create：文件落点 {state_dir}/audit/{session_id}.jsonl，state_dir=global_state_dir()=~/.codesleuth（HOME 缺失时报 CONFIG_MISSING 错误）；create_dir_all + append 打开；超 64MB（AUDIT_MAX_BYTES）单代轮转 .old，轮转失败仅 warn 继续追加；轮转后旧代原文 recall 不可达（已知限制）；结构 Audit{path, Mutex<File>, AtomicU64 seq}
   - src/cli.rs:225-229（审计 #7）
   - src/config.rs:147-150（审计 #28）
   - src/audit.rs:69-103（审计 #2）
3. Audit::record：payload 键先并入 BTreeMap，再顶层写 seq/ts_ms/kind（顶层胜出防键冲突）；seq 取号必须在 Mutex 锁内（锁外取号会破「按 seq 单调」不变量）；flush 失败打 warn 不静默；返回单调 seq 供账本/证据引用。事件种类：llm(:153)、llm_error(:140)、compaction_begin/compaction(:112,:117)、prose_steer(:189)、answer(:201)、report(:243)、tool_call(:350)、tool_result(:365)、recall(:284)、tool_rejected(:604)、fuse(:619)
   - src/audit.rs:135-163（审计 #2）
   - src/harness.rs:350-355（审计 #4）
4. writeguard 宿主级事件不走 Audit 实例：任务结束 run_task_inner 考后 diff 后调用 audit::append_line(outcome.audit_path, 'write_check', ...)——每次全量读文件取 max_seq+1 保持单调，与 record 同规合并 payload；留痕自身失败也有 ERROR/warn 兜底
   - src/cli.rs:369-421（审计 #7）
   - src/audit.rs:20-51（审计 #2）
5. 压缩三段式与审计的契约：主循环触发压缩时 audit_to=last_seq()，build_handoff(task, 1, audit_to)——起点硬编码 1（P005 R4.1：seq 从 1 起号，硬编码 2 会让首条记录永久失联）；有驱逐才写 compaction_begin/compaction 留痕
   - src/harness.rs:99-124（审计 #4）
6. recall 钻取：RECALL_TOOL='recall' 为 harness 内置工具（对 LLM 可见可调，builtin_schemas），主循环 0.5 步拦截（不走去重），解析 from/to（缺省 from=0/to=from）→ Audit::read_range 全文件扫描、闭区间 [from,to] 过滤、sort_by_key 升序返回（不重不漏；解析失败行 continue、seq 缺失按 0）→ context::recall_message 包装为带 #seq 标注 User 消息回填，recall 调用本身也写审计 kind='recall'
   - src/harness.rs:273-298（审计 #4）
   - src/harness.rs:445-456（审计 #4）
   - src/audit.rs:114-133（审计 #2）
7. 报告↔审计双向回溯：工具执行后 EvidenceStore.observe/observe_exact 记录 路径→首次观察 seq（evidence.rs:26-55）；submit_report 组装时 build_report 逐条 evidence 用 cite_seq 匹配，命中回填 Evidence.audit_seq（report.rs:10-16，互查锚点），未命中列入 uncited 打回
   - src/harness.rs:504-516（审计 #4）
   - src/evidence.rs:26-55（审计 #18）
   - src/report.rs:9-16（审计 #40）
8. 熔断 fuse_if_hit：no_progress≥MAX_NO_PROGRESS_STREAK(5) 时 audit.record('fuse', {no_progress_streak}) + tracing::error + 返回 LLM_FUSE(CS2099) 会话级错误——审计流水是熔断的落点之一
   - src/harness.rs:616-629（审计 #4）
   - src/harness.rs:18-21（审计 #4）
9. 配置与开关：审计路径由 global_state_dir()（~/.codesleuth，HOME 环境变量间接决定）决定；64MB 轮转阈值 AUDIT_MAX_BYTES 编译期常量不可配；压缩时机由 context.model_context_tokens（默认 1_000_000）与 context.compact_at_percent（默认 60）经四层配置链（CLI > 项目 .codesleuth/config.toml > 全局 ~/.codesleuth/config.toml > 默认，无环境变量层）影响；熔断/转向阈值(5/2)硬编码不可调
   - src/audit.rs:77-79（审计 #2）
   - src/config.rs:115-118（审计 #28）
   - src/config.rs:173-201（审计 #28）
   - src/harness.rs:18-21（审计 #4）
10. 下游消费：RunOutcome.audit_path 供 cli.rs 打印『# 审计:』与 write_check 追加；评估系统 scripts/eval/hardgate.py load_audit_rows 从 stderr 抽取审计路径建 {seq:row}，硬门 G2 强制每条 evidence.audit_seq 可回溯到 tool_call 行；单测 audit_records_monotonic_lines 固化 seq 单调语义
   - src/harness.rs:26-33（审计 #4）
   - src/cli.rs:448-453（审计 #7）
   - src/audit.rs:174-193（审计 #2）

## 死胡同
- grep 'audit|append_line|read_range' 初次 plain 模式 0 命中（字段名差异），换 regex 后命中
- 未逐行读 harness.rs 460-1084 全部（读至 669 行 + grep 补证）；fuse 附近测试段 harness.rs:670-1084 未展开，置信依赖已有行号证据
- report.rs 31-189、context.rs 原文未读，recall_message/handoff 细节引用自 docs 交叉验证（已在正文中以 findings 限定在已读行号内）

## 置信度
high

## 统计
turns=12 · tool_calls=16 · duration=50734ms · tokens=339298


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
全量 JSONL 审计：按 seq 单调递增把会话事件追加写入 {session_id}.jsonl，作为全量原文恢复源；harness/writeguard/cli 各环节调用 Audit::record 写入，压缩 handoff 用 last_seq 记录范围，recall 经 read_range 按区间读回。

## 证据列表
1. 审计 JSONL 是全量原文恢复源，支持按 seq 区间读回（read_range 升序不重不漏）
   - src/audit.rs:1-3（审计 #2）
   - src/audit.rs:114-133（审计 #2）
2. Audit::create 建每会话文件并做 64MB 单代轮转；record 在锁内取 seq+写盘保证单调
   - src/audit.rs:63-103（审计 #2）
   - src/audit.rs:135-163（审计 #2）
3. harness::run 压缩前用 last_seq 定区间并写 compaction 事件；writeguard::write_check 用 append_line 追加宿主级事件
   - src/harness.rs:99-124（审计 #16）
   - src/audit.rs:20-51（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=6 · tool_calls=6 · duration=25619ms · tokens=33571
