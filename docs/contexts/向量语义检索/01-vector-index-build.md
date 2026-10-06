# 向量索引构建流水线（向量语义检索）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`01-vector-index-build-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 向量索引构建流水线（向量语义检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
向量索引构建流水线把代码仓库切块、嵌入并向量化后落进项目本地的 SQLite 单文件索引（.codesleuth/indexes/<fingerprint>/vectors.db），为语义检索（RecallEngine/HNSW 召回）提供下游数据源，并通过 text_hash 增量复用与失效块 GC 保持索引与代码同步。

## 证据列表
1. 功能价值：把仓库代码切块→组装→嵌入→SQLite 落地，供向量召回使用；text_hash 未变的 chunk 整体复用、向量零开销。
   - src/vector/build.rs:1-5（审计 #2）
   - src/vector/build.rs:24-30（审计 #2）
2. 入口与关键文件：src/vector/build.rs:25 build_vector_index 编排整条流水线；src/vector/store.rs:306 project_index_dir 指定索引存于 <repo>/.codesleuth/，store.rs:310 index_path 具体为 indexes/<fingerprint>/vectors.db；src/vector/mod.rs 是模块汇总。
   - src/vector/build.rs:25-105（审计 #2）
   - src/vector/store.rs:303-312（审计 #9）
   - src/vector/mod.rs:1-14（审计 #4）
3. 运作方式：CLI 两个入口（src/cli.rs:163 手动预建、src/cli.rs:565 run 时引导补建，均在 bootlock 引导锁内）调用 build_vector_index；后者从 codegraph.db 取符号→plan_chunks 切块→比对 existing_hashes 做 text_hash 复用→compose_input 组装→embed.embed 网络调用→store.commit_build 在单事务内完成 GC 删除+upsert+meta 写入（store.rs 原子落库）。
   - src/cli.rs:163-168（审计 #17）
   - src/cli.rs:565-583（审计 #17）
   - src/vector/build.rs:31-92（审计 #2）
4. 交互关系：上游依赖 codegraph 符号库（build.rs:34-35 读 .codegraph/codegraph.db）与嵌入客户端 EmbedClient（模型/维度/模式变更则旧向量全部作废，build.rs:40-47）；下游消费方是 RecallEngine（recall.rs:34 open 从索引库构建 HNSW 影子召回），cli.rs:585 构建完成后立即打开该索引供检索用。
   - src/vector/build.rs:34-35（审计 #2）
   - src/vector/recall.rs:33-36（审计 #24）
   - src/cli.rs:584-589（审计 #17）
   - src/vector/build.rs:40-47（审计 #2）
   - src/vector/build.rs:49-57（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=6 · tool_calls=10 · duration=39901ms · tokens=31835


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
向量索引构建流水线由 CLI 两入口（index --vector 手动预建 cli.rs:163、run 时 setup_vector_layer 自动补建 cli.rs:565）在 bootlock 引导锁内调用 build_vector_index（build.rs:25-105）：从 .codegraph/codegraph.db 只读 nodes 表切块（chunk.rs:258-474 三层：符号主块/超大滑窗/leftover+fallback）→ compose_input 组装（Raw 或默认 Composite=header+标识符，compose.rs:42-51）→ EmbedClient.embed 批量并发网络调用（Qwen3-Embedding-8B，dim 1024，batch 64，并发 4，去重+错位即拒绝，embed.rs:72-220）→ commit_build 在单事务内完成 GC 删除+upsert+meta 写入（store.rs:201-232）落至 <repo>/.codesleuth/indexes/<fingerprint>/vectors.db。text_hash 未变复用、模型/维度/模式变更全量重嵌；下游 RecallEngine（recall.rs:34）加载该索引，≤10 万块暴力精确、巨型仓才建 HNSW。配置全部在 [vector] 段（默认 Qwen3-Embedding-8B/1024/composite，config.rs:119-126），嵌入端点缺省跟随 [llm]（resolve_embed_endpoint cli.rs:520-530），环境变量层已移除。

## 证据列表
1. 调用链：CLI 两入口（cli.rs:163 / cli.rs:565）在 bootlock 锁内调用 build_vector_index；run 侧构建失败弹性降级复用旧索引，bootlock 竞争判负硬退出
   - src/vector/build.rs:25-105（审计 #2）
   - src/cli.rs:128-178（审计 #22）
   - src/cli.rs:534-599（审计 #22）
2. 切块三层：codegraph nodes 表为边界真源（chunk.rs:137-166，缺失被 build.rs:35 unwrap_or_default 静默吞掉——最大误用陷阱），KEPT_KINDS 主块+6000 字符滑窗（100 行窗/15 重叠）+leftover/fallback
   - src/vector/chunk.rs:258-474（审计 #12）
   - src/vector/chunk.rs:137-166（审计 #12）
   - src/vector/build.rs:34-35（审计 #2）
3. 嵌入：内容去重→64/批→并发4→失败2s重试；响应 index 严格 0-based 校验防投毒；网络调用在事务外，commit_build 单事务原子落库（GC+upsert+meta，store.rs:201-232）
   - src/vector/embed.rs:72-146（审计 #20）
   - src/vector/embed.rs:206-220（审计 #20）
   - src/vector/store.rs:201-232（审计 #7）
4. 增量与失效：text_hash 复用（build.rs:63），meta 的 model/dim/mode 任一变更→全量重嵌（build.rs:40-47）；索引路径 <repo>/.codesleuth/indexes/<fingerprint>/vectors.db（store.rs:306-312）；[vector] 配置段默认 Qwen3-Embedding-8B/1024/composite，嵌入端点缺省跟随 [llm]（config.rs:119-126, cli.rs:520-530），环境变量层已移除（config.rs:3）
   - src/vector/build.rs:40-47（审计 #2）
   - src/vector/store.rs:303-312（审计 #7）
   - src/config.rs:119-126（审计 #33）
5. 下游契约：RecallEngine::open 从索引库加载并校验 meta/维度（recall.rs:33-53），MIN_HNSW_SIZE=100_000 以下暴力精确（初稿「HNSW 召回」表述需修正为常态暴力）；召回命中喂 repo-map 种子（cli.rs:600+）
   - src/vector/recall.rs:20-53（审计 #41）
   - src/cli.rs:589-599（审计 #22）

## 死胡同
无

## 置信度
high

## 统计
turns=13 · tool_calls=19 · duration=123394ms · tokens=316392


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
向量索引构建流水线把代码仓库切块、嵌入并向量化后落进项目本地的 SQLite 单文件索引（.codesleuth/indexes/<fingerprint>/vectors.db），为语义检索（RecallEngine/HNSW 召回）提供下游数据源，并通过 text_hash 增量复用与失效块 GC 保持索引与代码同步。

## 证据列表
1. 功能价值：把仓库代码切块→组装→嵌入→SQLite 落地，供向量召回使用；text_hash 未变的 chunk 整体复用、向量零开销。
   - src/vector/build.rs:1-5（审计 #2）
   - src/vector/build.rs:24-30（审计 #2）
2. 入口与关键文件：src/vector/build.rs:25 build_vector_index 编排整条流水线；src/vector/store.rs:306 project_index_dir 指定索引存于 <repo>/.codesleuth/，store.rs:310 index_path 具体为 indexes/<fingerprint>/vectors.db；src/vector/mod.rs 是模块汇总。
   - src/vector/build.rs:25-105（审计 #2）
   - src/vector/store.rs:303-312（审计 #9）
   - src/vector/mod.rs:1-14（审计 #4）
3. 运作方式：CLI 两个入口（src/cli.rs:163 手动预建、src/cli.rs:565 run 时引导补建，均在 bootlock 引导锁内）调用 build_vector_index；后者从 codegraph.db 取符号→plan_chunks 切块→比对 existing_hashes 做 text_hash 复用→compose_input 组装→embed.embed 网络调用→store.commit_build 在单事务内完成 GC 删除+upsert+meta 写入（store.rs 原子落库）。
   - src/cli.rs:163-168（审计 #17）
   - src/cli.rs:565-583（审计 #17）
   - src/vector/build.rs:31-92（审计 #2）
4. 交互关系：上游依赖 codegraph 符号库（build.rs:34-35 读 .codegraph/codegraph.db）与嵌入客户端 EmbedClient（模型/维度/模式变更则旧向量全部作废，build.rs:40-47）；下游消费方是 RecallEngine（recall.rs:34 open 从索引库构建 HNSW 影子召回），cli.rs:585 构建完成后立即打开该索引供检索用。
   - src/vector/build.rs:34-35（审计 #2）
   - src/vector/recall.rs:33-36（审计 #24）
   - src/cli.rs:584-589（审计 #17）
   - src/vector/build.rs:40-47（审计 #2）
   - src/vector/build.rs:49-57（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=6 · tool_calls=10 · duration=39901ms · tokens=31835
