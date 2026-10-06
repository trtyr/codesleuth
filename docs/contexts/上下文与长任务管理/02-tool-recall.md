# recall 续读工具（上下文与长任务管理）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`02-tool-recall-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# recall 续读工具（上下文与长任务管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
recall 是 harness 暴露给 LLM 的审计续读工具：按 [from, to] seq 范围从 JSONL 审计日志读回被压缩驱逐的原文行（升序、不重不漏），格式化为带 #seq 标注的用户消息回填上下文，支撑长任务压缩后的历史钻取。调用链：LLM 发起 RECALL_TOOL 调用 → harness 主循环拦截 → Audit::read_range 过滤排序 → context::recall_message 包装 → push 回 messages。上游依赖审计流水（record/last_seq）与上下文压缩 handoff（提供 recall 范围指引），下游消费是主循环上下文与空转/零增量统计。

## 证据列表
1. recall 工具名为 RECALL_TOOL="recall"，harness 主循环在 0.5 步拦截该调用，解析 from/to 后调 audit.read_range，并把结果作为用户消息 push 回上下文（幂等元操作不走去重）
   - src/harness.rs:273-297（审计 #2）
   - src/harness.rs:23（审计 #2）
2. Audit::read_range 读回 seq ∈ [from,to] 的审计原文行，按 seq 升序排序，保证不重不漏；last_seq 供压缩时标记审计范围
   - src/audit.rs:114-133（审计 #6）
   - src/audit.rs:109-112（审计 #6）
3. context::recall_message 将读回行格式化为「[recall] 审计原文（按 seq 升序，不重不漏）」逐行 #seq 标注，空范围给出提示，包装成 ChatMessage::User
   - src/context.rs:84-94（审计 #4）
4. 压缩 handoff 指引含 recall 可用范围 seq 1..=N（P005 R4.1：起点必须是 1），是 recall 的上游触发来源
   - src/context.rs:124-129（审计 #4）
   - src/harness.rs:105（审计 #2）
5. recall 结果内容参与空转/零增量统计（info_keys 计入 seen_keys 时重置 no_progress 与 zero_gain_streak），且 recall 调用本身也写入审计（record "recall"）
   - src/harness.rs:284-293（审计 #2）
6. recall 与 submit_report 同为 harness 内置处理但对 LLM 可见可调的工具；测试 recall_reads_audit_range_without_loss 验证 read_range 区间语义与消息格式
   - src/harness.rs:422-446（审计 #2）
   - src/harness.rs:942-964（审计 #2）

## 死胡同
- src/vector/recall.rs 的 RecallEngine 与本功能无关，仅命名相似（docs/上下文与长任务管理/01-context-compression.md:22 亦明确提示）

## 置信度
high

## 统计
turns=4 · tool_calls=5 · duration=29990ms · tokens=23009

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
recall 是 harness 内置、对 LLM 可见可调的审计续读工具：LLM 发起 {from,to} 调用，主循环在 0.5 步拦截（不走普通工具去重），Audit::read_range 全文件扫描过滤 seq∈[from,to] 并按 seq 升序返回，context::recall_message 包装为带 #seq 标注的 User 消息 push 回 messages；同时 recall 调用本身写入审计（kind="recall"），其返回内容经 info_keys 计入 seen_keys 可重置 no_progress/zero_gain_streak。上游触发来自压缩 handoff（范围固定 seq 1..=last_seq，P005 R4.1）。初稿各条结论均经原文复核成立，未发现初稿错误；补充发现：seq 缺失行按 0 处理、from 缺省 0/to 缺省 from 的宽容解析、审计 64MB 轮转后旧代原文不可 recall、压缩阈值受 context.compact_at_percent（默认 60）配置影响。

## 证据列表
1. 工具面：RECALL_TOOL="recall" 为 harness 内置常量，不进 ToolRegistry；builtin_schemas() 向 LLM 输出 schema（required [from,to]，integer），与 submit_report 并列
   - src/harness.rs:22-23（审计 #2）
   - src/harness.rs:423-458（审计 #2）
2. 主循环拦截：0.5 步在去重/工具注册表检查之前处理 recall，parse_args 失败则 reject_call；from 缺省 0、to 缺省 from（as_u64 宽容解析）；无 canonical 去重（幂等元操作），执行后 continue 跳过普通工具通道
   - src/harness.rs:273-298（审计 #2）
3. 读取实现：Audit::read_range 每次 std::fs::read_to_string 全量读审计 JSONL，逐行 serde_json 解析，seq 缺失/解析失败行静默跳过（解析失败 continue；seq 缺失按 0 计），闭区间 [from,to] 过滤后 sort_by_key 升序返回 Vec<(u64,Value)>
   - src/audit.rs:114-133（审计 #4）
4. 格式化落点：context::recall_message 生成「[recall] 审计原文（按 seq 升序，不重不漏）：」+ 逐行 "#{seq} {json}"，空范围补「（该范围无审计行）」，包装为 ChatMessage::User push 回 messages
   - src/context.rs:84-94（审计 #6）
5. 审计自记录：recall 执行后 audit.record("recall", {from,to,lines:len}) 留痕，即 recall 结果本身也进审计流水可被再次 recall
   - src/harness.rs:283-287（审计 #2）
6. 空转统计联动：recall 消息内容经 info_keys 提取 key，新 key 计入 seen_keys 时 no_progress 与 zero_gain_streak 归零——recall 读回新原文可视为「有进展」
   - src/harness.rs:288-295（审计 #2）
   - src/harness.rs:89-92（审计 #2）
7. 上游触发：主循环每轮 should_compact 超阈值（窗口×percent）时 build_handoff(task, 1, last_seq) 生成含 recall 指引的交接通知（P005 R4.1 硬编码起点 1），compact 保留头部 2 条+最近 KEEP_RECENT=6 条，中间驱逐
   - src/harness.rs:99-124（审计 #2）
   - src/context.rs:44-54（审计 #6）
   - src/context.rs:56-82（审计 #6）
8. last_seq 依赖锁内原子取号：Audit.seq 为 AtomicU64，record 在 Mutex 锁内 fetch_add 保证 JSONL 按 seq 单调（P004 T2.4），这是 read_range「不重不漏」的前提
   - src/audit.rs:109-112（审计 #4）
   - src/audit.rs:144-147（审计 #4）
9. 配置：ContextConfig{model_context_tokens, compact_at_percent}，默认 1_000_000 / 60；compact_threshold_tokens 将 percent>100 封顶 100；配置决定压缩何时触发从而决定 recall 何时可用
   - src/config.rs:94-97（审计 #29）
   - src/config.rs:115-118（审计 #29）
   - src/context.rs:30-33（审计 #6）
10. 边界坑：审计文件超 64MB 单代轮转为 .jsonl.old，轮转后 recall 无法读旧代原文（代码注释明示此代价）；read_range 只读当前 path
   - src/audit.rs:77-92（审计 #4）
11. 测试契约：recall_reads_audit_range_without_loss 验证 read_range(2,4)==[2,3,4] 与消息含 #2/#4 不含 #1
   - src/harness.rs:942-964（审计 #2）

## 死胡同
- grep 'compact_percent|context_tokens'（plain）无命中：Harness 字段名与 config 键名不同（compact_percent vs compact_at_percent），换 pattern 后命中
- src/vector/recall.rs RecallEngine 仅命名相似，与本功能无关（初稿已判，本次维持）

## 置信度
high

## 统计
turns=7 · tool_calls=12 · duration=37904ms · tokens=126018


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
recall 是 harness 暴露给 LLM 的审计续读工具：按 [from, to] seq 范围从 JSONL 审计日志读回被压缩驱逐的原文行（升序、不重不漏），格式化为带 #seq 标注的用户消息回填上下文，支撑长任务压缩后的历史钻取。调用链：LLM 发起 RECALL_TOOL 调用 → harness 主循环拦截 → Audit::read_range 过滤排序 → context::recall_message 包装 → push 回 messages。上游依赖审计流水（record/last_seq）与上下文压缩 handoff（提供 recall 范围指引），下游消费是主循环上下文与空转/零增量统计。

## 证据列表
1. recall 工具名为 RECALL_TOOL="recall"，harness 主循环在 0.5 步拦截该调用，解析 from/to 后调 audit.read_range，并把结果作为用户消息 push 回上下文（幂等元操作不走去重）
   - src/harness.rs:273-297（审计 #2）
   - src/harness.rs:23（审计 #2）
2. Audit::read_range 读回 seq ∈ [from,to] 的审计原文行，按 seq 升序排序，保证不重不漏；last_seq 供压缩时标记审计范围
   - src/audit.rs:114-133（审计 #6）
   - src/audit.rs:109-112（审计 #6）
3. context::recall_message 将读回行格式化为「[recall] 审计原文（按 seq 升序，不重不漏）」逐行 #seq 标注，空范围给出提示，包装成 ChatMessage::User
   - src/context.rs:84-94（审计 #4）
4. 压缩 handoff 指引含 recall 可用范围 seq 1..=N（P005 R4.1：起点必须是 1），是 recall 的上游触发来源
   - src/context.rs:124-129（审计 #4）
   - src/harness.rs:105（审计 #2）
5. recall 结果内容参与空转/零增量统计（info_keys 计入 seen_keys 时重置 no_progress 与 zero_gain_streak），且 recall 调用本身也写入审计（record "recall"）
   - src/harness.rs:284-293（审计 #2）
6. recall 与 submit_report 同为 harness 内置处理但对 LLM 可见可调的工具；测试 recall_reads_audit_range_without_loss 验证 read_range 区间语义与消息格式
   - src/harness.rs:422-446（审计 #2）
   - src/harness.rs:942-964（审计 #2）

## 死胡同
- src/vector/recall.rs 的 RecallEngine 与本功能无关，仅命名相似（docs/上下文与长任务管理/01-context-compression.md:22 亦明确提示）

## 置信度
high

## 统计
turns=4 · tool_calls=5 · duration=29990ms · tokens=23009
