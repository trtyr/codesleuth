# 上下文三段式压缩（上下文与长任务管理）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`01-context-compression-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 上下文三段式压缩（上下文与长任务管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
上下文三段式压缩：当会话消息的估算 token 达到「模型窗口 × 百分比」阈值时，Agent 循环自动执行「持久化（审计账本落盘）→ 生成确定性 handoff 衔接通知 → 压缩窗口」，全程零 LLM 调用，使长任务不因上下文装满而中断，被驱逐的原文可经 recall 从审计原文不重不漏地续读。入口在 src/context.rs，由 src/harness.rs 主循环触发。

## 证据列表
1. 功能定义：窗口阈值触发三段式压缩（持久化→handoff→压缩），handoff 内含 recall 指引，零 LLM
   - src/context.rs:1-4（审计 #2）
   - src/context.rs:44-53（审计 #2）
2. 入口与核心函数：compact_threshold_tokens/should_compact 触发判断；compact 为确定性纯函数，保留头部 system+task 与最近 KEEP_RECENT=6 条，中间替换为 handoff
   - src/context.rs:30-37（审计 #2）
   - src/context.rs:56-82（审计 #2）
3. 调用链：harness 主循环每轮算阈值并 should_compact 判断，触发后依次调用 build_handoff 与 compact，再通过 self.audit.record 写 compaction_begin/compaction 审计留痕
   - src/harness.rs:100-124（审计 #10）
4. 交互：上游依赖审计账本（last_seq/record）与 ContextConfig（compact_at_percent 默认 60）；压缩带 tool 调用对感知（不拆散 assistant/tool pair），被驱逐原文供 recall 钻取
   - src/config.rs:92-97,115-118（审计 #15）
   - src/context.rs:63-71（审计 #2）

## 死胡同
- grep 误召回 src/vector/recall.rs 的向量索引 compact，与本功能无关

## 置信度
high

## 统计
turns=6 · tool_calls=5 · duration=25309ms · tokens=43192


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
上下文三段式压缩由 src/harness.rs 主循环每轮触发：阈值=context_tokens×compact_percent（percent>100 封顶 100，context.rs:31-33），should_compact 用 chars/4 估算（context.rs:25-37）判断；触发后取 audit.last_seq()、build_handoff(task,1,seq)（起点硬编码 1，harness.rs:105-107）、compact 保留头 2 条+最近 KEEP_RECENT=6 条并替换中间为 handoff User 消息（context.rs:57-82），有驱逐才写 compaction_begin/compaction 审计留痕（harness.rs:110-120）。全程零 LLM。被驱逐原文的真相源是 audit/{session_id}.jsonl（audit.rs:70-103，seq 锁内取号保证单调 audit.rs:145-147），LLM 可经 harness 内置 recall 工具→read_range→recall_message 回填（harness.rs:274-298, context.rs:85-94）。配置：ContextConfig.model_context_tokens 默认 1M、compact_at_percent 默认 60（config.rs:115-118），KEEP_RECENT=6 为常量不可配。校正初稿两点：①handoff recall 范围固定从 seq 1 起（P005 R4.1）；②「持久化」非压缩流程内独立步骤——审计实时落盘，压缩只取水位+补留痕。边界：无驱逐 no-op 静默（context.rs:73-75）、tool pair 防拆散（context.rs:66-71）、64MB 审计轮转后旧代原文 recall 不可达（audit.rs:77-91）、recall 参数容错 from=0/to=from（harness.rs:280-281）。

## 证据列表
1. 触发链：主循环每轮算阈值（窗口×percent，>100 封顶）并 should_compact（chars/4 估算）判断
   - src/context.rs:31-37（审计 #2）
   - src/harness.rs:100-124（审计 #7）
2. 三段式核心：last_seq 取水位 → build_handoff(task,1,seq_to)（起点硬编码 1，P005 R4.1）→ compact 保留头 2+尾 6，中间替换为 handoff User 消息；零 LLM
   - src/harness.rs:105-120（审计 #7）
   - src/context.rs:47-54（审计 #2）
   - src/context.rs:57-82（审计 #2）
3. compact 的 tool pair 感知：裁剪边界落在 Tool 消息时回退到 assistant 搭档前，防下游 400；evicted_count=0 时静默 no-op
   - src/context.rs:66-71（审计 #2）
   - src/context.rs:73-75（审计 #2）
   - src/harness.rs:121-123（审计 #7）
4. recall 续读：harness 拦截 recall 调用 → Audit::read_range（全文件扫描+seq 升序）→ recall_message 包装 #seq 标注回填 messages，且计入空转重置
   - src/harness.rs:273-298（审计 #7）
   - src/audit.rs:115-133（审计 #22）
   - src/context.rs:85-94（审计 #2）
5. recall 内容命中 seen_keys 时重置 no_progress 与 zero_gain_streak（与空转熔断的交互契约）
   - src/harness.rs:289-295（审计 #7）
   - src/audit.rs:136-158（审计 #22）
6. 配置：ContextConfig.model_context_tokens 默认 1M、compact_at_percent 默认 60，可经配置文件覆盖，无环境变量层；KEEP_RECENT=6 编译期常量
   - src/config.rs:115-118（审计 #27）
   - src/config.rs:237-241（审计 #27）
   - src/cli.rs:353-360（审计 #19）
   - src/context.rs:9（审计 #2）
7. 审计存储契约：audit/{session_id}.jsonl 追加写，seq 锁内取号保证单调；>64MB 轮转 .old 后旧代原文 recall 不可达（已知限制）
   - src/audit.rs:70-103（审计 #22）
   - src/audit.rs:145-147（审计 #22）
8. recall 参数容错：from 缺省 0、to 缺省=from；非法 JSON 走 reject_call
   - src/harness.rs:280-281（审计 #7）
   - src/harness.rs:275-278（审计 #7）

## 死胡同
无

## 置信度
high

## 统计
turns=7 · tool_calls=11 · duration=116272ms · tokens=105808


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
上下文三段式压缩：当会话消息的估算 token 达到「模型窗口 × 百分比」阈值时，Agent 循环自动执行「持久化（审计账本落盘）→ 生成确定性 handoff 衔接通知 → 压缩窗口」，全程零 LLM 调用，使长任务不因上下文装满而中断，被驱逐的原文可经 recall 从审计原文不重不漏地续读。入口在 src/context.rs，由 src/harness.rs 主循环触发。

## 证据列表
1. 功能定义：窗口阈值触发三段式压缩（持久化→handoff→压缩），handoff 内含 recall 指引，零 LLM
   - src/context.rs:1-4（审计 #2）
   - src/context.rs:44-53（审计 #2）
2. 入口与核心函数：compact_threshold_tokens/should_compact 触发判断；compact 为确定性纯函数，保留头部 system+task 与最近 KEEP_RECENT=6 条，中间替换为 handoff
   - src/context.rs:30-37（审计 #2）
   - src/context.rs:56-82（审计 #2）
3. 调用链：harness 主循环每轮算阈值并 should_compact 判断，触发后依次调用 build_handoff 与 compact，再通过 self.audit.record 写 compaction_begin/compaction 审计留痕
   - src/harness.rs:100-124（审计 #10）
4. 交互：上游依赖审计账本（last_seq/record）与 ContextConfig（compact_at_percent 默认 60）；压缩带 tool 调用对感知（不拆散 assistant/tool pair），被驱逐原文供 recall 钻取
   - src/config.rs:92-97,115-118（审计 #15）
   - src/context.rs:63-71（审计 #2）

## 死胡同
- grep 误召回 src/vector/recall.rs 的向量索引 compact，与本功能无关

## 置信度
high

## 统计
turns=6 · tool_calls=5 · duration=25309ms · tokens=43192
