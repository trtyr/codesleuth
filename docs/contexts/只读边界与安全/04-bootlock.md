# bootlock 引导锁（只读边界与安全）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`04-bootlock-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# bootlock 引导锁（只读边界与安全）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
bootlock（D014 索引引导锁）是一把仓库级跨进程文件锁 `.codesleuth/boot.lock`，用于把并发进程的索引引导（codegraph init / index --force 与向量索引构建）串行化，防止两路 check-then-act 竞争撞库；竞争败者以 CS4016 INDEX_LOCKED 判负退出而不降级（重跑秒过），从而保证调用方拿到完整工具面。

## 证据列表
1. bootlock 用 flock 锁文件把两层「check-then-act」索引引导在进程间串行化，背景是 2026-10-05 双进程并发 init 事故（图与向量层共用同一问题）
   - src/bootlock.rs:1-14（审计 #2）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:7-14（审计 #4）
2. 入口在 src/bootlock.rs：acquire 实现 try_lock 首试 + 500ms 轮询等待（Won/Lost 三态，超时 300s），acquire_guard 把 Lost/超时统一收残为 INDEX_LOCKED 结构化错误
   - src/bootlock.rs:50-114（审计 #2）
   - src/bootlock.rs:116-129（审计 #2）
3. 三个调用点：graph.rs start_with_bin 的 init/index --force 引导、cli.rs 的 index --vector 手动构建、cli.rs run 时的向量构建，均通过 `?` 传播判负退出
   - src/tools/graph.rs:75-84（审计 #13）
   - src/cli.rs:157-166（审计 #15）
4. 上游依赖：errors.rs 的 CS4016 INDEX_LOCKED / REPO_NOT_READABLE 错误码；实现依赖 std::fs::File::try_lock（Rust 1.89，无新外部依赖），Guard Drop 时自动 unlock
   - src/bootlock.rs:16（审计 #2）
   - src/bootlock.rs:12-13（审计 #2）
   - src/bootlock.rs:35-48（审计 #2）
5. 下游消费：向量构建（build_vector_index，chunk 集合是 codegraph 图符号表的投影）在锁内执行；锁只罩 init/build，不跨 LLM 调用、不罩 serve
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:14（审计 #4）
   - src/bootlock.rs:116-119（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=6 · duration=16377ms · tokens=28372


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
bootlock（D014）是仓库级跨进程 flock 文件锁 `.codesleuth/boot.lock`，将三层 check-then-act 索引引导（graph init/index --force、run 时向量构建、index --vector 手动构建）在进程间串行化。三个调用点统一走 acquire_guard（P005 R1 收残）：Won 正常引导，Lost/超时收残为 CS4016 INDEX_LOCKED（errors.rs:51），CLI 在 cli.rs:259/315 对该码特判判负退出不降级（exit code 5）；锁仅罩引导段，不跨 LLM/serve/检索；500ms 轮询与 300s 上限为硬编码常量不可配置；进程崩溃由内核放锁无陈旧锁问题。

## 证据列表
1. 锁实现：acquire 首试 try_lock 得 Won，竞争则 500ms 轮询（上限 300s），TryLockError::Error 亦映射 INDEX_LOCKED；acquire_guard 把 Lost/超时统一收残为 INDEX_LOCKED + hint
   - src/bootlock.rs:22-24（审计 #2）
   - src/bootlock.rs:51-114（审计 #2）
   - src/bootlock.rs:116-129（审计 #2）
2. 调用点 A：run_task_inner 调 CodegraphEngine::start(&repo_abs, &cfg.graph.bin, self.fresh_index)，start_with_bin 内 acquire_guard 后执行 init/index --force，drop(guard) 放锁后才 spawn serve
   - src/cli.rs:252-253（审计 #14）
   - src/tools/graph.rs:71-89（审计 #9）
3. 调用点 B/C：index --vector 走 run_index_vector（cli.rs:158-162），run --vector 走 setup_vector_layer（cli.rs:559-563），均先 acquire_guard 再 build_vector_index
   - src/cli.rs:128-178（审计 #14）
   - src/cli.rs:534-584（审计 #14）
4. 判负不降级契约：CLI 对 graph 层与向量层的 INDEX_LOCKED 特判 return Err 判负退出；同函数内构建失败（非锁竞争）弹性降级并审计 degraded——锁竞争致命、构建失败可降级是设计取舍
   - src/cli.rs:259-262（审计 #14）
   - src/cli.rs:315-318（审计 #14）
   - src/cli.rs:575-582（审计 #14）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:24-27（审计 #4）
5. 错误码与退出码：INDEX_LOCKED = CsCode(4016)（errors.rs:51），CS4xxx 段位映射 exit code 5（errors.rs:13-22）；REPO_NOT_READABLE(3002) 用于锁目录/文件打开失败
   - src/errors.rs:13-22（审计 #24）
   - src/errors.rs:43（审计 #24）
   - src/errors.rs:51（审计 #24）
6. 锁文件数据流：create_dir_all(.codesleuth) → OpenOptions create+write+truncate(false) 打开 boot.lock → flock 互斥 → Guard Drop unlock；.codesleuth/ 为 writeguard 豁免目录故锁文件常驻不触发零写入自证告警
   - src/bootlock.rs:12-13（审计 #2）
   - src/bootlock.rs:52-64（审计 #2）
   - src/bootlock.rs:44-48（审计 #2）
7. 被锁保护的业务流：build_vector_index 从 .codegraph/codegraph.db 只读 nodes 表投影 chunk → text_hash 增量复用 → embed 网络调用 → GC+upsert+meta 原子事务落 SQLite（防两进程并发 GC/嵌入/写同一库）
   - src/vector/build.rs:31-36（审计 #29）
   - src/vector/build.rs:61-68（审计 #29）
   - src/vector/build.rs:78-92（审计 #29）
8. 配置面：无专属配置项/环境变量，POLL_INTERVAL=500ms 与 DEFAULT_TIMEOUT=300s 硬编码且调用点固定传 DEFAULT_TIMEOUT；间接相关 cfg.graph.bin（CODEGRAPH_BIN env 已移除，graph.bin 为键表配置键）、--fresh-index、--vector
   - src/bootlock.rs:22-24（审计 #2）
   - src/tools/graph.rs:78-82（审计 #9）
   - src/config.rs:389-395（审计 #39）
9. 背景与动机：2026-10-05 双进程并发 init 事故，codegraph 锁竞争 stderr 为空错误无归因（graph.rs:58-63 补 hint）；向量是 codegraph 下游（chunk=图符号表投影），两层同一问题
   - src/tools/graph.rs:58-63（审计 #9）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:7-14（审计 #4）
   - src/vector/build.rs:34-36（审计 #29）

## 死胡同
无

## 置信度
high

## 统计
turns=10 · tool_calls=15 · duration=156118ms · tokens=209242


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
bootlock（D014 索引引导锁）是一把仓库级跨进程文件锁 `.codesleuth/boot.lock`，用于把并发进程的索引引导（codegraph init / index --force 与向量索引构建）串行化，防止两路 check-then-act 竞争撞库；竞争败者以 CS4016 INDEX_LOCKED 判负退出而不降级（重跑秒过），从而保证调用方拿到完整工具面。

## 证据列表
1. bootlock 用 flock 锁文件把两层「check-then-act」索引引导在进程间串行化，背景是 2026-10-05 双进程并发 init 事故（图与向量层共用同一问题）
   - src/bootlock.rs:1-14（审计 #2）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:7-14（审计 #4）
2. 入口在 src/bootlock.rs：acquire 实现 try_lock 首试 + 500ms 轮询等待（Won/Lost 三态，超时 300s），acquire_guard 把 Lost/超时统一收残为 INDEX_LOCKED 结构化错误
   - src/bootlock.rs:50-114（审计 #2）
   - src/bootlock.rs:116-129（审计 #2）
3. 三个调用点：graph.rs start_with_bin 的 init/index --force 引导、cli.rs 的 index --vector 手动构建、cli.rs run 时的向量构建，均通过 `?` 传播判负退出
   - src/tools/graph.rs:75-84（审计 #13）
   - src/cli.rs:157-166（审计 #15）
4. 上游依赖：errors.rs 的 CS4016 INDEX_LOCKED / REPO_NOT_READABLE 错误码；实现依赖 std::fs::File::try_lock（Rust 1.89，无新外部依赖），Guard Drop 时自动 unlock
   - src/bootlock.rs:16（审计 #2）
   - src/bootlock.rs:12-13（审计 #2）
   - src/bootlock.rs:35-48（审计 #2）
5. 下游消费：向量构建（build_vector_index，chunk 集合是 codegraph 图符号表的投影）在锁内执行；锁只罩 init/build，不跨 LLM 调用、不罩 serve
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:14（审计 #4）
   - src/bootlock.rs:116-119（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=6 · duration=16377ms · tokens=28372
