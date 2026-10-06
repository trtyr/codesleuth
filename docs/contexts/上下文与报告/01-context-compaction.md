# 上下文压缩（上下文与报告）

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
# 上下文压缩（上下文与报告）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
上下文压缩（D013 v2）由 Harness::run 主循环驱动：每轮用 模型窗口 × 百分比（默认 1M × 60%）算阈值，由 should_compact 判定；触发后三段式 —— ①调 build_handoff 拼承上启下文本（零 LLM）→ ②调 compact 保留头部 2 条 + handoff + 最近 KEEP_RECENT=6 条、中间驱逐 → ③写 compaction_begin / compaction 审计留痕；被驱逐的原文通过 recall 工具按审计 seq 范围续读，不重不漏。

## 证据列表
1. 压缩触发判定在 Harness::run 主循环内：算阈值 → should_compact → 三段式执行。
   - src/harness.rs:100-109（审计 #16）
   - src/harness.rs:102（审计 #16）
   - src/harness.rs:103（审计 #16）
2. 阈值函数为「模型上下文 × 百分比」，percent>100 按 100 封顶。
   - src/context.rs:30-33（审计 #2）
   - src/context.rs:109-114（审计 #2）
3. 承上启下 handoff 是确定性拼装、零 LLM，先于压缩生成，包含任务重述、recall 范围指引、下一步指引。
   - src/context.rs:44-54（审计 #2）
   - src/harness.rs:104-107（审计 #16）
4. compact 纯函数保留头部 2 条 + handoff User 消息 + 最近 keep_recent 条，并保证 tool/assistant 配对不被裁剪边界拆散。
   - src/context.rs:56-82（审计 #2）
   - src/context.rs:131-152（审计 #2）
   - src/context.rs:162-195（审计 #2）
5. KEEP_RECENT=6 为保留尾部消息条数常量。
   - src/context.rs:8-9（审计 #2）
6. 真正产生驱逐时（evicted_count>0）才写 compaction_begin / compaction 审计；no-op 时不产噪音。
   - src/harness.rs:110-123（审计 #16）
7. recall 工具通过 context::recall_message 把审计原文按 seq 升序包装为 User 消息回灌，承接被驱逐内容的不重不漏续读。
   - src/context.rs:84-94（审计 #2）
   - src/harness.rs:288（审计 #16）
   - src/harness.rs:1020（审计 #16）
8. callers 图确认 compact / should_compact / build_handoff / compact_threshold_tokens 的唯一生产调用方均为 Harness::run（除测试）。
   - src/harness.rs:74（审计 #16）
9. D013 决策文档规定三段式时序与「窗口是物理口径」原则，并要求报告落盘到 reports/<session>.md|.json。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/013-compaction-v2-report-persistence.md:15-18（审计 #4）
10. 状态目录约定：~/.codesleuth/ 统一放 reports/audit/ledger/eval；上下文压缩必须确定性、零 LLM。
   - AGENTS.md:11（审计 #51）
   - AGENTS.md:14（审计 #51）
   - src/config.rs:164（审计 #53）
11. RunOutcome 携带 Report、answer、turns、tool_calls、audit_path，Harness 持有 context_tokens 与 compact_percent 配置。
   - src/harness.rs:25-44（审计 #16）
   - src/harness.rs:60-64（审计 #16）

## 死胡同
- grep "write_markdown|finish_report|report_path|finish_session" 0 命中——报告持久化写盘逻辑未在本次会话被读到（仅由 D013 决策与 AGENTS.md/config.rs 间接确认落点）。
- grep "pub async fn run" 仅 1 命中，RunOutcome 字段确认，但持久化报告的写文件实现在 harness.rs 后续行未逐行读完。

## 置信度
high

## 统计
turns=13 · tool_calls=20 · duration=52159ms · tokens=125652


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
上下文压缩（D013 v2）是 codesleuth Harness::run 主循环每轮 LLM 调用前做的确定性收口：按 `模型上下文窗口 × compact_percent`（默认 1M × 60%，percent>100 封顶）算阈值，由 `should_compact` 判定；触发后三段式——①`build_handoff` 拼承上启下文本（零 LLM，确定性）→ ②`compact` 纯函数保留头 2 条 + handoff User + 最近 KEEP_RECENT=6 条并对 tool/assistant 配对做回退保护 → ③真发生驱逐（evicted_count>0）才写 `compaction_begin` / `compaction` 审计。被驱逐原文通过 `recall` 工具按审计 seq 范围（起点硬编码 1）从 JSONL 钻取后由 `recall_message` 包装为 User 消息回灌，实现不重不漏续读。初稿描述与代码实际行为一致；唯一遗留是「reports/<session>.md|.json 写盘函数」未在 harness 后续行直接读到，仅由 D013 决策与 AGENTS.md 间接确认落点。

## 证据列表
1. 压缩触发判定在 Harness::run 主循环内：算阈值 → should_compact → 三段式执行。
   - src/harness.rs:99-124（审计 #5）
   - src/harness.rs:101-103（审计 #5）
   - src/context.rs:25-37（审计 #2）
2. 阈值函数为「模型上下文 × 百分比」，percent>100 按 100 封顶。
   - src/context.rs:30-33（审计 #2）
   - src/context.rs:109-114（审计 #2）
   - src/harness.rs:41-42（审计 #5）
   - src/harness.rs:60-62（审计 #5）
3. 承上启下 handoff 是确定性拼装、零 LLM，先于压缩生成，包含任务重述、recall 范围指引（起点硬编码 1）、下一步指引。
   - src/context.rs:44-54（审计 #2）
   - src/harness.rs:104-107（审计 #5）
   - src/context.rs:125-129（审计 #2）
4. compact 纯函数保留头部 2 条 + handoff User 消息 + 最近 keep_recent 条，并保证 tool/assistant 配对不被裁剪边界拆散。
   - src/context.rs:56-82（审计 #2）
   - src/context.rs:63-71（审计 #2）
   - src/context.rs:131-152（审计 #2）
   - src/context.rs:162-195（审计 #2）
5. KEEP_RECENT=6 为保留尾部消息条数常量，head=2 为固定保留 system+task。
   - src/context.rs:8-9（审计 #2）
   - src/context.rs:63（审计 #2）
6. 真正产生驱逐时（evicted_count>0）才写 compaction_begin / compaction 审计；no-op 时不产噪音。
   - src/harness.rs:110-123（审计 #5）
   - src/context.rs:72-75（审计 #2）
   - src/context.rs:155-160（审计 #2）
7. recall 工具通过 context::recall_message 把审计原文按 seq 升序包装为 User 消息回灌，承接被驱逐内容的不重不漏续读。
   - src/context.rs:84-94（审计 #2）
   - src/harness.rs:274-297（审计 #5）
   - src/harness.rs:1008-1029（审计 #5）
8. callers 图确认 compact / should_compact / compact_threshold_tokens 的唯一生产调用方均为 Harness::run（除测试）；build_handoff 同样。
   - src/harness.rs:74（审计 #5）
   - src/context.rs:132-195（审计 #2）
9. D013 决策文档规定三段式时序与「窗口是物理口径」原则，并要求报告落盘到 reports/<session>.md|.json。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/013-compaction-v2-report-persistence.md:1-25（审计 #17）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/013-compaction-v2-report-persistence.md:15-18（审计 #17）
10. 状态目录约定：~/.codesleuth/ 统一放 reports/audit/ledger/eval；上下文压缩必须确定性、零 LLM。
   - AGENTS.md:11（审计 #20）
   - AGENTS.md:14（审计 #20）
   - src/config.rs:160-167（审计 #23）
11. RunOutcome 携带 Report、answer、turns、tool_calls、audit_path，Harness 持有 context_tokens 与 compact_percent 配置；reject_call 把 Tool 拒绝原因回写 messages。
   - src/harness.rs:25-33（审计 #5）
   - src/harness.rs:35-44（审计 #5）
   - src/harness.rs:47-65（审计 #5）
   - src/harness.rs:205-211（审计 #5）
   - src/harness.rs:252-259（审计 #5）
   - src/harness.rs:600-615（审计 #5）

## 死胡同
- grep "write_markdown|finish_report|report_path|finish_session" 0 命中：报告持久化写盘的具体函数未在本次会话读到
- harness.rs 1149 行，仅读到 25-339、595-634、980-1079 区间；持久化报告生成函数位于未读区
- Audit::create/record/read_range 内部实现未直接读，仅通过 callers 与 harness 内的 self.audit.* 间接确认存在与签名

## 置信度
high

## 统计
turns=14 · tool_calls=18 · duration=107937ms · tokens=250459


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
上下文压缩（D013 v2）由 Harness::run 主循环驱动：每轮用 模型窗口 × 百分比（默认 1M × 60%）算阈值，由 should_compact 判定；触发后三段式 —— ①调 build_handoff 拼承上启下文本（零 LLM）→ ②调 compact 保留头部 2 条 + handoff + 最近 KEEP_RECENT=6 条、中间驱逐 → ③写 compaction_begin / compaction 审计留痕；被驱逐的原文通过 recall 工具按审计 seq 范围续读，不重不漏。

## 证据列表
1. 压缩触发判定在 Harness::run 主循环内：算阈值 → should_compact → 三段式执行。
   - src/harness.rs:100-109（审计 #16）
   - src/harness.rs:102（审计 #16）
   - src/harness.rs:103（审计 #16）
2. 阈值函数为「模型上下文 × 百分比」，percent>100 按 100 封顶。
   - src/context.rs:30-33（审计 #2）
   - src/context.rs:109-114（审计 #2）
3. 承上启下 handoff 是确定性拼装、零 LLM，先于压缩生成，包含任务重述、recall 范围指引、下一步指引。
   - src/context.rs:44-54（审计 #2）
   - src/harness.rs:104-107（审计 #16）
4. compact 纯函数保留头部 2 条 + handoff User 消息 + 最近 keep_recent 条，并保证 tool/assistant 配对不被裁剪边界拆散。
   - src/context.rs:56-82（审计 #2）
   - src/context.rs:131-152（审计 #2）
   - src/context.rs:162-195（审计 #2）
5. KEEP_RECENT=6 为保留尾部消息条数常量。
   - src/context.rs:8-9（审计 #2）
6. 真正产生驱逐时（evicted_count>0）才写 compaction_begin / compaction 审计；no-op 时不产噪音。
   - src/harness.rs:110-123（审计 #16）
7. recall 工具通过 context::recall_message 把审计原文按 seq 升序包装为 User 消息回灌，承接被驱逐内容的不重不漏续读。
   - src/context.rs:84-94（审计 #2）
   - src/harness.rs:288（审计 #16）
   - src/harness.rs:1020（审计 #16）
8. callers 图确认 compact / should_compact / build_handoff / compact_threshold_tokens 的唯一生产调用方均为 Harness::run（除测试）。
   - src/harness.rs:74（审计 #16）
9. D013 决策文档规定三段式时序与「窗口是物理口径」原则，并要求报告落盘到 reports/<session>.md|.json。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/013-compaction-v2-report-persistence.md:15-18（审计 #4）
10. 状态目录约定：~/.codesleuth/ 统一放 reports/audit/ledger/eval；上下文压缩必须确定性、零 LLM。
   - AGENTS.md:11（审计 #51）
   - AGENTS.md:14（审计 #51）
   - src/config.rs:164（审计 #53）
11. RunOutcome 携带 Report、answer、turns、tool_calls、audit_path，Harness 持有 context_tokens 与 compact_percent 配置。
   - src/harness.rs:25-44（审计 #16）
   - src/harness.rs:60-64（审计 #16）

## 死胡同
- grep "write_markdown|finish_report|report_path|finish_session" 0 命中——报告持久化写盘逻辑未在本次会话被读到（仅由 D013 决策与 AGENTS.md/config.rs 间接确认落点）。
- grep "pub async fn run" 仅 1 命中，RunOutcome 字段确认，但持久化报告的写文件实现在 harness.rs 后续行未逐行读完。

## 置信度
high

## 统计
turns=13 · tool_calls=20 · duration=52159ms · tokens=125652
