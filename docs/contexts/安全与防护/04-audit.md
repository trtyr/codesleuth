# 审计日志（安全与防护）

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
# 审计日志（安全与防护）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
审计日志是 codesleuth 的事实账本：每会话一个 JSONL 文件按 `seq` 单调记录 LLM/工具调用/压缩/写面自证等宿主级事件，供报告 finding 引用原文回溯，并通过 `audit_seq` 区间让压缩后被驱逐的早期消息可被 recall 续读。

## 证据列表
1. 审计模块的宿主级事件续号入口 `append_line` 在 src/audit.rs:21-51，取现有最大 seq + 1 写入 JSONL，保持单调不重不漏。
   - src/audit.rs:20-51（审计 #2）
2. 会话 ID 生成 `new_session_id` 用纳秒时间 + pid 的 hex 拼接。
   - src/audit.rs:53-60（审计 #2）
3. `Audit` 结构 + `create`/`record`/`read_range` 实现按 seq 单调的 JSONL 账本，单代轮转 64MB 上限。
   - src/audit.rs:62-163（审计 #2）
4. 进程入口在 main 生成 session_id 并注入日志与审计，保证日志串线与审计同一身份。
   - src/main.rs:5-12（审计 #25）
   - src/lib.rs:24-66（审计 #8）
5. CLI 用同一 session_id 创建审计日志，任务结束后调 `audit::append_line` 写 `write_check` 宿主级行（含失败留痕）。
   - src/cli.rs:231-235（审计 #22）
   - src/cli.rs:375-427（审计 #22）
6. Harness 在压缩/失败时通过 `self.audit.record` 留痕 `compaction_begin`/`compaction`/`llm_error`，并取 `last_seq` 喂给 handoff。
   - src/harness.rs:100-148（审计 #35）
7. 下游消费：`context::build_handoff` 引用 `audit_from..audit_to` 续读被驱逐原文；评估脚本从 stderr 解析审计路径并加载审计行做硬关；CLI stderr 摘要指向审计 `write_check` 行。
   - src/context.rs:47-54（审计 #42）
   - scripts/eval/run_eval.py:30-77（审计 #44）
   - src/cli.rs:411-411（审计 #22）
8. 单元测试断言 `append_line` 在已有 seq 之后续号为 max+1（写面自证 3 条 → seq 1,2,3 单调）。
   - src/writeguard.rs:226-249（审计 #4）
   - src/audit.rs:174-193（审计 #2）

## 死胡同
- grep "audit_path|Audit::create|\.record\(" 0 命中（正则需转义）
- grep "audit" 扫到 53 文件后未续扫剩余命中（已足以取证）

## 置信度
high

## 统计
turns=10 · tool_calls=17 · duration=52941ms · tokens=150525


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
审计日志是 codesleuth 唯一的事实账本：每会话一个 JSONL 文件按 seq 单调追加 LLM/工具/压缩/写面自证等宿主级事件，供报告 finding 的 evidence.audit_seq 反向回溯，并由 recall 工具按 audit_seq 区间续读被压缩驱逐的早期消息。两条写入路径：(a) Audit::record 在 Mutex<File> 锁内 AtomicU64::fetch_add 取号+写入原子成对（src/audit.rs:144-147），承担 Harness 主循环的 11+ kind 事件；(b) audit::append_line 走「读全文取 max+1」，专供 CLI 任务结束写 write_check（src/audit.rs:21-51）。读取路径：AUDIT_LINE 正则从 stderr 抓审计路径 → hardgate.load_audit_rows 装入 {seq:row} → G2 校验每条 finding.evidence.audit_seq 必对 kind ∈ {tool_call, tool_result} 行（scripts/eval/hardgate.py:30, 44-52, 69-84）。初稿说 append_line「保持单调不重不漏」是事实，但更精确的「锁内取号+写入原子」不变量是 Audit::record 的事（src/audit.rs:144-147 注释 P004 T2.4），append_line 与 record 不互锁，依赖 CLI 在 drop(agent) 之后单次调用的纪律（src/cli.rs:372-373）规避。

## 证据列表
1. 审计模块导出四个公共符号：append_line（src/audit.rs:21-51）、new_session_id（src/audit.rs:54-60）、Audit 结构（src/audit.rs:62-67）、create/record/last_seq/read_range 方法（src/audit.rs:70-163）
   - src/audit.rs:21-163（审计 #2）
2. Audit::record 锁内 fetch_add 取号+写入原子成对，BTreeMap 先 payload 后控制字段（seq/ts_ms/kind 顶层胜出），flush 失败仅 warn
   - src/audit.rs:136-163（审计 #2）
3. append_line 用「读全文取 max(seq)+1」策略，与 Audit::record 不互锁；当前唯一调用在 CLI 收尾 drop(agent) 之后，规避并发
   - src/audit.rs:21-51（审计 #2）
   - src/cli.rs:372-393（审计 #2）
4. Audit::create 写 ~/.codesleuth/audit/&lt;session_id&gt;.jsonl；超 64MB 单代轮转到 .jsonl.old；new_session_id 用纳秒时间+pid 的 hex
   - src/audit.rs:70-103（审计 #2）
   - src/audit.rs:54-60（审计 #2）
5. Audit::read_range 每次重读全文、按 seq 过滤升序返回，JSON 解析失败静默跳过；用于 Harness recall 工具路径
   - src/audit.rs:115-133（审计 #2）
   - src/harness.rs:273-298（审计 #2）
6. main 进程入口生成 session_id 并注入 init_tracing + Cli::run，保证日志串线与审计同一身份
   - src/main.rs:5-12（审计 #9）
   - src/lib.rs:27-66（审计 #11）
   - src/cli.rs:99-101（审计 #2）
7. CLI run_task_inner 调 Audit::create(state_dir, session_id)，把 Audit 实例注入 Harness::new；零写入自证 writeguard.snapshot 在任务前/后比对
   - src/cli.rs:233-366（审计 #2）
8. Harness 主循环 11+ kind 写入点：llm / llm_error / tool_call / tool_result / recall / compaction_begin / compaction / prose_steer / answer / report / fuse；失败也必留痕
   - src/harness.rs:106-211（审计 #2）
   - src/harness.rs:243-298（审计 #2）
   - src/harness.rs:350-368（审计 #2）
   - src/harness.rs:654-666（审计 #2）
9. 压缩三段式：last_seq() 喂给 build_handoff(task, 1, audit_to)；evicted_count>0 才留痕 compaction_begin/compaction，否则静默
   - src/harness.rs:99-124（审计 #2）
   - src/context.rs:47-54（审计 #23）
   - src/context.rs:85-94（审计 #23）
10. CLI 装配期非致命降级一律 audit.record("degraded", {component, error})，吞错不阻断主流程（codegraph / vector_layer / repo_map）
   - src/cli.rs:272-275（审计 #2）
   - src/cli.rs:328-331（审计 #2）
   - src/cli.rs:351-354（审计 #2）
11. CLI 任务结束统一 audit::append_line(&outcome.audit_path, "write_check", {files_snapshotted, unattributed_changes, samples})，含不可用分支 (available:false) 强制留痕
   - src/cli.rs:375-427（审计 #2）
   - src/harness.rs:25-33（审计 #2）
12. stderr 元数据 # 审计: &lt;path&gt; 是 eval 脚本抓审计路径的唯一渠道
   - src/cli.rs:454-460（审计 #2）
   - scripts/eval/run_eval.py:30（审计 #28）
13. 报告侧 Evidence.audit_seq 字段是报告↔审计互查锚点，degraded 时为 None；Report 字段集不含 audit_path，路径仅 Rust 端传递
   - src/report.rs:9-43（审计 #56）
14. Eval 硬门 G2：finding.evidence.audit_seq 必对 kind ∈ {tool_call, tool_result} 行；G1/G3/G4 校验报告文本、schema、非降级
   - scripts/eval/hardgate.py:12-13（审计 #28）
   - scripts/eval/hardgate.py:26-66（审计 #28）
   - scripts/eval/hardgate.py:69-84（审计 #28）
15. run_eval.py 从子进程 stdout 解析 --json 报告、从 stderr 抓审计路径；load_audit_rows 容忍行级 JSON 解析失败
   - scripts/eval/run_eval.py:34-77（审计 #28）
16. 单元测试三处：audit_records_monotonic_lines 验 record 单调（src/audit.rs:174-193）、write_check_appends_with_monotonic_seq 验 append_line 续号 max+1 且 payload 键冲突顶层胜出（src/writeguard.rs:226-256）、recall_reads_audit_range_without_loss 验 read_range 不重不漏（src/harness.rs:1007-1029）
   - src/audit.rs:174-193（审计 #2）
   - src/writeguard.rs:226-256（审计 #2）
   - src/harness.rs:1007-1029（审计 #2）
17. writeguard.snapshot 跳过 .git/.codegraph/.codesleuth/target/node_modules/dist/__pycache__/.venv；.codegraph 是 D011 豁免，codegraph 索引增长不入 manifest
   - src/writeguard.rs:11-20（审计 #2）
   - src/writeguard.rs:212-224（审计 #2）
18. handoff_recall_starts_at_first_audit_seq 测试断言审计 seq 从 1 起号、handoff 指引 recall 必须从 1 开始（避免首条记录永久失联）
   - src/context.rs:125-129（审计 #23）

## 死胡同
- grep "audit_path|Audit::create|\.record\(" 正则未转义 0 命中（与初稿一致）
- RunOutcome.audit_path 是否进入报告 JSON：grep "pub struct Report" 仅显示 src/report.rs:33-43 字段集，无 audit_path 字段，证实 audit_path 仅在 Rust 端传递、报告本体通过 Evidence.audit_seq 锚点反向引用审计行
- audit::create 内部 64MB 阈值处的轮转失败分支处理仅 read 注释，未逐行读轮转 .old 的打开/读取路径——recall 是否能读 .old 待核

## 置信度
high

## 统计
turns=16 · tool_calls=32 · duration=169863ms · tokens=712260


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
审计日志是 codesleuth 的事实账本：每会话一个 JSONL 文件按 `seq` 单调记录 LLM/工具调用/压缩/写面自证等宿主级事件，供报告 finding 引用原文回溯，并通过 `audit_seq` 区间让压缩后被驱逐的早期消息可被 recall 续读。

## 证据列表
1. 审计模块的宿主级事件续号入口 `append_line` 在 src/audit.rs:21-51，取现有最大 seq + 1 写入 JSONL，保持单调不重不漏。
   - src/audit.rs:20-51（审计 #2）
2. 会话 ID 生成 `new_session_id` 用纳秒时间 + pid 的 hex 拼接。
   - src/audit.rs:53-60（审计 #2）
3. `Audit` 结构 + `create`/`record`/`read_range` 实现按 seq 单调的 JSONL 账本，单代轮转 64MB 上限。
   - src/audit.rs:62-163（审计 #2）
4. 进程入口在 main 生成 session_id 并注入日志与审计，保证日志串线与审计同一身份。
   - src/main.rs:5-12（审计 #25）
   - src/lib.rs:24-66（审计 #8）
5. CLI 用同一 session_id 创建审计日志，任务结束后调 `audit::append_line` 写 `write_check` 宿主级行（含失败留痕）。
   - src/cli.rs:231-235（审计 #22）
   - src/cli.rs:375-427（审计 #22）
6. Harness 在压缩/失败时通过 `self.audit.record` 留痕 `compaction_begin`/`compaction`/`llm_error`，并取 `last_seq` 喂给 handoff。
   - src/harness.rs:100-148（审计 #35）
7. 下游消费：`context::build_handoff` 引用 `audit_from..audit_to` 续读被驱逐原文；评估脚本从 stderr 解析审计路径并加载审计行做硬关；CLI stderr 摘要指向审计 `write_check` 行。
   - src/context.rs:47-54（审计 #42）
   - scripts/eval/run_eval.py:30-77（审计 #44）
   - src/cli.rs:411-411（审计 #22）
8. 单元测试断言 `append_line` 在已有 seq 之后续号为 max+1（写面自证 3 条 → seq 1,2,3 单调）。
   - src/writeguard.rs:226-249（审计 #4）
   - src/audit.rs:174-193（审计 #2）

## 死胡同
- grep "audit_path|Audit::create|\.record\(" 0 命中（正则需转义）
- grep "audit" 扫到 53 文件后未续扫剩余命中（已足以取证）

## 置信度
high

## 统计
turns=10 · tool_calls=17 · duration=52941ms · tokens=150525
