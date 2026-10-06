# 向量切块（向量检索）

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
# 向量切块（向量检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量切块（chunk）位于 src/vector/chunk.rs，以 codegraph 符号表为边界真源，按「主块 / 超大滑窗 / 兜底」三层规则把仓库源码切成可嵌入的 Chunk；主入口 plan_chunks（chunk.rs:258）由 build_vector_index（build.rs:25）调用，产物经 compose_input、VectorStore、RecallEngine、CLI 报告消费。

## 证据列表
1. 模块定位为「codegraph 符号表为边界真源 + 三层规则」：层1 符号主块、层2 超大滑窗、层3 兜底。
   - src/vector/chunk.rs:1-5（审计 #4）
2. 关键常量：MAX_CHUNK_CHARS=6000、SUB_WINDOW_LINES=100、SUB_OVERLAP_LINES=15、FALLBACK_WINDOW_LINES=100、FALLBACK_OVERLAP_LINES=20；KEPT_KINDS=function/method/struct/class/interface/impl/trait。
   - src/vector/chunk.rs:14-47（审计 #4）
3. codegraph 适配器：symbols_from_codegraph 只读打开 SQLite nodes 表返回 SymbolRow 列表；relations_for_symbol 提供 callers/callees（去重封顶 8）。
   - src/vector/chunk.rs:84-134（审计 #4）
   - src/vector/chunk.rs:137-166（审计 #4）
4. 子窗口切分在超大符号体内按 SUB_WINDOW_LINES 切、每窗重拼面包屑 breadcrumb。
   - src/vector/chunk.rs:177-179（审计 #4）
   - src/vector/chunk.rs:227-244（审计 #4）
5. 主入口 plan_chunks 编排三层：按文件分组的 KEPT_KINDS 过滤 + 文档行上提 → 主块或 sub_window_chunks 滑窗 → covered gaps 产 leftover → 磁盘兜底产 fallback；最后按 (file,line_start,symbol) 排序。
   - src/vector/chunk.rs:258-474（审计 #4）
   - src/vector/chunk.rs:280-340（审计 #4）
   - src/vector/chunk.rs:342-380（审计 #4）
   - src/vector/chunk.rs:407-468（审计 #4）
6. build_vector_index 调 symbols_from_codegraph → plan_chunks → 用 text_hash 增量复用 → 嵌入 → commit_build 原子落库。
   - src/vector/build.rs:8-11（审计 #11）
   - src/vector/build.rs:25-69（审计 #11）
7. 模块对外契约：pub use 暴露 Chunk、SymbolRow、plan_chunks。
   - src/vector/mod.rs:1-21（审计 #9）
   - src/vector/mod.rs:17（审计 #9）
8. compose_input 消费 Chunk：Raw 模式返回 chunk.text，Composite 模式 = header + 标识符层（去重封顶 20）。
   - src/vector/compose.rs:6（审计 #39）
   - src/vector/compose.rs:42-51（审计 #39）
9. VectorStore 用 chunk_key(file + symbol + line_start) 作为 chunk 唯一键、存 SQLite chunks 表 + 唯一索引。
   - src/vector/store.rs:7（审计 #41）
   - src/vector/store.rs:14-16（审计 #41）
10. RecallEngine 持 rows: Vec<(Chunk, Vec<f32>)>；recall 按文本查询返回 (Chunk, f32) 命中；format_recall_block 把 hits 渲染为「起步线索」格式。
   - src/vector/recall.rs:7（审计 #43）
   - src/vector/recall.rs:128-139（审计 #43）
   - src/vector/recall.rs:142-159（审计 #43）
11. CLI 在两条路径上触发 build_vector_index 并打印 chunk/嵌入/复用/GC 报告；run_task 阶段还通过 relations_for_symbol 取每个 hit 的 callers/callees 入图扩展种子。
   - src/cli.rs:167-180（审计 #45）
   - src/cli.rs:571-589（审计 #45）
   - src/cli.rs:630-643（审计 #45）

## 死胡同
- grep 关键字 "Chunk\b|chunk_key" / "plan_chunks|chunk::" 在词边界模式下 0 命中（grep 默认 plain 行为），改用宽松词 "chunk" 后获得完整调用链。
- 未读 chunk.rs:474 之后的测试实现细节（仅用于验证行为，不影响主流程理解）。
- 未读 src/vector/embed.rs（嵌入客户端，与切块逻辑正交）。
- 首次提交因引用了未读过的 compose/store/recall/cli 行被拒，read 后重提。

## 置信度
high

## 统计
turns=11 · tool_calls=19 · duration=50011ms · tokens=166943


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
向量切块位于 src/vector/chunk.rs，以 codegraph SQLite 的 nodes 表为边界真源，按「主块 / 超大滑窗 / 兜底」三层规则把仓库源码切成可嵌入的 Chunk；主入口 plan_chunks（src/vector/chunk.rs:258）由 build_vector_index（src/vector/build.rs:25）调用，组装后经 EmbedClient（src/vector/embed.rs:42）嵌入、VectorStore commit_build 事务（src/vector/store.rs:201-232）原子落库 .codesleuth/indexes/<fingerprint>/vectors.db，最终被 RecallEngine（src/vector/recall.rs:25）和 format_recall_block（src/vector/recall.rs:142）召回消费；召回命中还通过 relations_for_symbol 喂 caller/callee 给 repomap 作任务导航图种子。

## 证据列表
1. 切块模块以 codegraph 符号表为边界真源，按 KEPT_KINDS（function/method/struct/class/interface/impl/trait）过滤并丢弃 file 容器节点。
   - src/vector/chunk.rs:1-5（审计 #2）
   - src/vector/chunk.rs:14-49（审计 #2）
   - src/vector/chunk.rs:280-303（审计 #2）
2. codegraph 适配器只读打开 SQLite nodes 表返回 SymbolRow；relations_for_symbol 通过 edges 联结 nodes 取 callers/callees（去重封顶 8）。
   - src/vector/chunk.rs:84-134（审计 #2）
   - src/vector/chunk.rs:137-166（审计 #2）
3. 超大符号体按 SUB_WINDOW_LINES=100 切、每窗重拼面包屑（重叠 15 行），line 区间按原始行号回填。
   - src/vector/chunk.rs:177-179（审计 #2）
   - src/vector/chunk.rs:227-244（审计 #2）
   - src/vector/chunk.rs:321-335（审计 #2）
4. plan_chunks 编排三层：按文件分组的 KEPT_KINDS 过滤 + docstring 上提 → 主块或 sub_window_chunks 滑窗 → 覆盖区间合并产 leftover → 磁盘 walk 产 fallback；最终按 (file, line_start, symbol) 排序。
   - src/vector/chunk.rs:258-381（审计 #2）
   - src/vector/chunk.rs:383-468（审计 #2）
   - src/vector/chunk.rs:470-474（审计 #2）
5. build_vector_index 走「fingerprint → codegraph 读 nodes → plan_chunks → 哈希复用 → 嵌入 → commit_build 原子落库」，model/dim/mode 任一变更即全量重建。
   - src/vector/build.rs:8-11（审计 #2）
   - src/vector/build.rs:25-92（审计 #2）
6. 模块对外契约：mod.rs 重导出 Chunk/SymbolRow/plan_chunks/EmbedClient/RecallEngine/VectorStore 等。
   - src/vector/mod.rs:1-21（审计 #2）
   - src/vector/mod.rs:16-21（审计 #2）
7. compose_input：Raw 模式返回 chunk.text，Composite 模式 = header + 标识符层（去重封顶 20）。
   - src/vector/compose.rs:14-51（审计 #2）
8. VectorStore 唯一键 chunk_key = file + \u{1} + symbol + \u{1} + line_start；chunks 表唯一索引 idx_chunks_key 建于 (file, symbol, line_start)。
   - src/vector/store.rs:14-16（审计 #2）
   - src/vector/store.rs:57-75（审计 #2）
   - src/vector/store.rs:241-285（审计 #2）
9. commit_build 用 unchecked_transaction 包裹 GC 删 + 全部 upsert + meta(model/dim/mode) 翻新，失败整体回滚。
   - src/vector/store.rs:108-140（审计 #2）
   - src/vector/store.rs:170-232（审计 #2）
   - src/vector/store.rs:306-312（审计 #2）
10. RecallEngine 持 rows: Vec<(Chunk, Vec<f32>)> + alive HashSet + 可选 HNSW 影子图；≤10 万向量走暴力、否则 HNSW；tombstone 超 20% 触发 compact。
   - src/vector/recall.rs:13-22（审计 #22）
   - src/vector/recall.rs:25-97（审计 #22）
   - src/vector/recall.rs:100-139（审计 #22）
11. format_recall_block 把 hits 渲染为「起步线索」模板：K≤10、相似度暴露、含「可能无关」「必须 read 原文」免责。
   - src/vector/recall.rs:142-159（审计 #22）
12. EmbedClient 按 batch_size=64 分批、EMBED_CONCURRENCY=4 并发；hash 去重后按槽位回填；429/5xx 标 retryable，单批失败 2s 重试一次；validate_index_alignment 强制 0-based 对齐防投毒。
   - src/vector/embed.rs:13-16（审计 #2）
   - src/vector/embed.rs:42-69（审计 #2）
   - src/vector/embed.rs:72-220（审计 #2）
13. CLI 有两条触发路径：子命令 codesleuth index <path> --vector 走 run_index_vector；主流程 --vector 旗标走 setup_vector_layer；二者都通过 bootlock 引导锁串行化、失败非致命进 audit degraded 行。
   - src/cli.rs:60-78（审计 #27）
   - src/cli.rs:113-180（审计 #27）
   - src/cli.rs:194-334（审计 #27）
   - src/cli.rs:540-665（审计 #27）
14. run_task_inner 中召回命中通过 relations_for_symbol 喂 caller/callee 给 repomap::build_task_map 作任务导航图种子。
   - src/cli.rs:607-654（审计 #27）
15. VectorConfig 含 embed_model/embed_dims/embed_mode/repomap_budget/base_url/api_key 6 字段；嵌入端点缺省跟随 [llm]。
   - src/config.rs:93-104（审计 #41）
   - src/config.rs:133-140（审计 #41）
   - src/cli.rs:143-152（审计 #27）
   - src/cli.rs:551-556（审计 #27）
16. 初稿偏差：初稿说 store.rs:7 是 chunk_key 定义，实际 store.rs:7 是 use import 行，chunk_key 在 store.rs:14-16；初稿说 store.rs:14-16 是唯一索引，实际唯一索引 SQL 在 store.rs:69；初稿说子命令入口是 cli.rs:167-180，实际该段是 run_index_vector（子命令路径），主流程入口是 cli.rs:540-589 setup_vector_layer。
   - src/vector/store.rs:7-7（审计 #2）
   - src/vector/store.rs:14-16（审计 #2）
   - src/vector/store.rs:69-69（审计 #2）
   - src/cli.rs:113-127（审计 #27）
   - src/cli.rs:167-180（审计 #27）
   - src/cli.rs:540-589（审计 #27）

## 死胡同
- grep 关键字 \u{1} 分隔等特殊模式 0 命中（grep 工具默认 plain 行为），改用 explore 拿全图。
- 未读 src/vector/embed.rs 中 validate_index_alignment 之后部分测试细节（仅影响嵌入客户端的额外容错，主切块流程无关）。
- 未读 src/vector/recall.rs 260 行后剩余单测（仅验证行为，不影响主流程理解）。
- 初稿引用了未读过的 compose/store/recall/cli 行（如 store.rs:7 当作 chunk_key），read 后纠正为 store.rs:14-16 定义、store.rs:69 唯一索引 SQL。

## 置信度
high

## 统计
turns=10 · tool_calls=25 · duration=152792ms · tokens=381508


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量切块（chunk）位于 src/vector/chunk.rs，以 codegraph 符号表为边界真源，按「主块 / 超大滑窗 / 兜底」三层规则把仓库源码切成可嵌入的 Chunk；主入口 plan_chunks（chunk.rs:258）由 build_vector_index（build.rs:25）调用，产物经 compose_input、VectorStore、RecallEngine、CLI 报告消费。

## 证据列表
1. 模块定位为「codegraph 符号表为边界真源 + 三层规则」：层1 符号主块、层2 超大滑窗、层3 兜底。
   - src/vector/chunk.rs:1-5（审计 #4）
2. 关键常量：MAX_CHUNK_CHARS=6000、SUB_WINDOW_LINES=100、SUB_OVERLAP_LINES=15、FALLBACK_WINDOW_LINES=100、FALLBACK_OVERLAP_LINES=20；KEPT_KINDS=function/method/struct/class/interface/impl/trait。
   - src/vector/chunk.rs:14-47（审计 #4）
3. codegraph 适配器：symbols_from_codegraph 只读打开 SQLite nodes 表返回 SymbolRow 列表；relations_for_symbol 提供 callers/callees（去重封顶 8）。
   - src/vector/chunk.rs:84-134（审计 #4）
   - src/vector/chunk.rs:137-166（审计 #4）
4. 子窗口切分在超大符号体内按 SUB_WINDOW_LINES 切、每窗重拼面包屑 breadcrumb。
   - src/vector/chunk.rs:177-179（审计 #4）
   - src/vector/chunk.rs:227-244（审计 #4）
5. 主入口 plan_chunks 编排三层：按文件分组的 KEPT_KINDS 过滤 + 文档行上提 → 主块或 sub_window_chunks 滑窗 → covered gaps 产 leftover → 磁盘兜底产 fallback；最后按 (file,line_start,symbol) 排序。
   - src/vector/chunk.rs:258-474（审计 #4）
   - src/vector/chunk.rs:280-340（审计 #4）
   - src/vector/chunk.rs:342-380（审计 #4）
   - src/vector/chunk.rs:407-468（审计 #4）
6. build_vector_index 调 symbols_from_codegraph → plan_chunks → 用 text_hash 增量复用 → 嵌入 → commit_build 原子落库。
   - src/vector/build.rs:8-11（审计 #11）
   - src/vector/build.rs:25-69（审计 #11）
7. 模块对外契约：pub use 暴露 Chunk、SymbolRow、plan_chunks。
   - src/vector/mod.rs:1-21（审计 #9）
   - src/vector/mod.rs:17（审计 #9）
8. compose_input 消费 Chunk：Raw 模式返回 chunk.text，Composite 模式 = header + 标识符层（去重封顶 20）。
   - src/vector/compose.rs:6（审计 #39）
   - src/vector/compose.rs:42-51（审计 #39）
9. VectorStore 用 chunk_key(file + symbol + line_start) 作为 chunk 唯一键、存 SQLite chunks 表 + 唯一索引。
   - src/vector/store.rs:7（审计 #41）
   - src/vector/store.rs:14-16（审计 #41）
10. RecallEngine 持 rows: Vec<(Chunk, Vec<f32>)>；recall 按文本查询返回 (Chunk, f32) 命中；format_recall_block 把 hits 渲染为「起步线索」格式。
   - src/vector/recall.rs:7（审计 #43）
   - src/vector/recall.rs:128-139（审计 #43）
   - src/vector/recall.rs:142-159（审计 #43）
11. CLI 在两条路径上触发 build_vector_index 并打印 chunk/嵌入/复用/GC 报告；run_task 阶段还通过 relations_for_symbol 取每个 hit 的 callers/callees 入图扩展种子。
   - src/cli.rs:167-180（审计 #45）
   - src/cli.rs:571-589（审计 #45）
   - src/cli.rs:630-643（审计 #45）

## 死胡同
- grep 关键字 "Chunk\b|chunk_key" / "plan_chunks|chunk::" 在词边界模式下 0 命中（grep 默认 plain 行为），改用宽松词 "chunk" 后获得完整调用链。
- 未读 chunk.rs:474 之后的测试实现细节（仅用于验证行为，不影响主流程理解）。
- 未读 src/vector/embed.rs（嵌入客户端，与切块逻辑正交）。
- 首次提交因引用了未读过的 compose/store/recall/cli 行被拒，read 后重提。

## 置信度
high

## 统计
turns=11 · tool_calls=19 · duration=50011ms · tokens=166943
