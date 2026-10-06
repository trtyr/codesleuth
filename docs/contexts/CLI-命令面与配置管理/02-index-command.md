# 向量索引构建命令（CLI 命令面与配置管理）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`02-index-command-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 向量索引构建命令（CLI 命令面与配置管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
`index <path> --vector` 命令为指定目标仓库手动预构建向量语义索引：基于 codegraph 结构库切分符号 chunk，调用嵌入服务生成向量并增量落库（复用未变块、GC 清理死块），供运行时语义检索复用。

## 证据列表
1. 命令入口 run_index_vector：规范化仓库路径、加载配置并解析嵌入端点（独立 [vector] 供应商，缺省跟随 [llm]），最后打印构建报告。
   - src/cli.rs:128-178（审计 #2）
2. 入口在 tokio runtime 上调用 vector::build_vector_index 完成实际构建，且与运行时引导共用 bootlock 引导锁（竞争败者退出）。
   - src/cli.rs:155-168（审计 #2）
3. 构建核心：从 .codegraph/codegraph.db 取符号并 plan_chunks，校验模型/维度/模式变更导致旧向量作废，仅嵌入变更块，GC 删除失效块后 commit_build 单事务原子落库。
   - src/vector/build.rs:25-105（审计 #4）
4. 索引存储于仓库内 .codesleuth/indexes/<fingerprint>/vectors.db，由 project_index_dir/index_path 决定路径。
   - src/vector/store.rs:303-312（审计 #7）

## 死胡同
- 未确认 CLI 层 index 子命令 clap 定义的具体行（未读 run_task/clap 解析段）

## 置信度
high

## 统计
turns=3 · tool_calls=3 · duration=11094ms · tokens=12541

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
`index <path> --vector` 由 cli.rs:65-74 clap 子命令分发（cli.rs:105-117）到 run_index_vector（cli.rs:128-178）：规范化路径 → config::load + resolve_api_key → resolve_embed_endpoint（[vector] 优先、[llm] 兜底）→ EmbedClient → bootlock::acquire_guard（与 run 引导共用锁，Lost 即 INDEX_LOCKED 退出）→ tokio 上跑 build_vector_index（build.rs:25-105）。构建流：从 .codegraph/codegraph.db 只读 nodes 表（chunk.rs:137-166，缺失则 unwrap_or_default 静默空集）→ plan_chunks 三层切块（chunk.rs:258+，主块≤6000字符/超大滑窗/leftover兜底）→ compose_input（Composite=header+20标识符，compose.rs:42-51）→ embed.embed 并发4/batch64/去重/重试1次（embed.rs:72-176）→ commit_build 单事务原子落库（GC删除+upsert+meta，store.rs:201-232）。落点 <repo>/.codesleuth/indexes/<fingerprint>/vectors.db（fingerprint=sha256(路径+git HEAD)前16hex，store.rs:19-27,306-312）。配置：vector.embed_model 默认 Qwen/Qwen3-Embedding-8B、embed_dims 1024、embed_mode composite（config.rs:119-126），model/dims/mode 任一变更即全量重嵌（build.rs:40-47）；env 层已移除，密钥只认配置文件（config.rs:458-468）。坑：codegraph.db 缺失不报错反而 GC 删光旧索引（build.rs:35）；--rebuild 是摆设旗标（cli.rs:68）；embed_mode 拼错静默回退 composite（cli.rs:145-148）。下游：run --vector 经 setup_vector_layer（cli.rs:534-556）加载同一 vectors.db 建 RecallEngine 供 vector_search。

## 证据列表
1. CLI 子命令 Index{path,rebuild,vector} 定义与 --rebuild 仅为提示旗标（初稿死胡同已补齐）
   - src/cli.rs:65-74（审计 #2）
2. 分发逻辑：vector=true 走 run_index_vector，否则 index_structure_error
   - src/cli.rs:105-123（审计 #2）
3. run_index_vector 全流程：canonicalize→config→embed端点解析→EmbedClient→bootlock→build_vector_index→打印报告
   - src/cli.rs:128-178（审计 #2）
4. resolve_embed_endpoint：[vector] 优先、[llm] 兜底
   - src/cli.rs:520-530（审计 #2）
5. build_vector_index 七步编排；codegraph.db 打不开时 unwrap_or_default 静默空集（初稿未提的坑）
   - src/vector/build.rs:25-105, 35（审计 #4）
6. 切块三层规则与常量（6000字符/100行滑窗/重叠15/兜底重叠20/KEPT_KINDS/SKIP_DIRS）
   - src/vector/chunk.rs:1-49（审计 #16）
   - src/vector/chunk.rs:258-334（审计 #16）
7. compose_input Raw/Composite 语义
   - src/vector/compose.rs:42-51（审计 #25）
8. EmbedClient：batch64、并发 EMBED_CONCURRENCY=4、内容去重回填、失败2s后重试1次、429/5xx retryable
   - src/vector/embed.rs:15-16（审计 #27）
   - src/vector/embed.rs:72-146（审计 #27）
   - src/vector/embed.rs:149-177（审计 #27）
9. VectorStore schema（chunks/meta/descriptions）、existing_hashes、commit_build 单事务原子落库
   - src/vector/store.rs:45-79（审计 #7）
   - src/vector/store.rs:143-167（审计 #7）
   - src/vector/store.rs:201-232（审计 #7）
10. 索引路径 = repo/.codesleuth/indexes/<fingerprint>/vectors.db；fingerprint 含 git HEAD（换分支产生新索引目录）
   - src/vector/store.rs:19-27（审计 #7）
   - src/vector/store.rs:303-312（审计 #7）
11. 配置默认值与 VectorConfig 字段；配置链 CLI>项目>全局>默认，env 层移除
   - src/config.rs:1-3（审计 #30）
   - src/config.rs:80-90（审计 #30）
   - src/config.rs:119-126（审计 #30）
12. resolve_api_key 只认配置文件直配，缺失报 CONFIG_MISSING
   - src/config.rs:458-468（审计 #30）
13. bootlock acquire_guard 判负语义：Lost → INDEX_LOCKED，三个调用点共用
   - src/bootlock.rs:116-129（审计 #41）
14. 下游消费：setup_vector_layer 复用同一解析/客户端构造加载向量层（补建语义）
   - src/cli.rs:532-556（审计 #2）
15. RecallEngine 消费 vectors.db（≤10万块暴力、HNSW 影子图）
   - src/vector/recall.rs:1-30（审计 #32）

## 死胡同
- 初稿未确认的 clap index 子命令定义已定位（src/cli.rs:65-74），无遗留死胡同
- 未深读 plan_chunks 335 行之后的 leftover/README 兜底细节（对调用链结论无影响）

## 置信度
high

## 统计
turns=9 · tool_calls=21 · duration=118432ms · tokens=192810


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
`index <path> --vector` 命令为指定目标仓库手动预构建向量语义索引：基于 codegraph 结构库切分符号 chunk，调用嵌入服务生成向量并增量落库（复用未变块、GC 清理死块），供运行时语义检索复用。

## 证据列表
1. 命令入口 run_index_vector：规范化仓库路径、加载配置并解析嵌入端点（独立 [vector] 供应商，缺省跟随 [llm]），最后打印构建报告。
   - src/cli.rs:128-178（审计 #2）
2. 入口在 tokio runtime 上调用 vector::build_vector_index 完成实际构建，且与运行时引导共用 bootlock 引导锁（竞争败者退出）。
   - src/cli.rs:155-168（审计 #2）
3. 构建核心：从 .codegraph/codegraph.db 取符号并 plan_chunks，校验模型/维度/模式变更导致旧向量作废，仅嵌入变更块，GC 删除失效块后 commit_build 单事务原子落库。
   - src/vector/build.rs:25-105（审计 #4）
4. 索引存储于仓库内 .codesleuth/indexes/<fingerprint>/vectors.db，由 project_index_dir/index_path 决定路径。
   - src/vector/store.rs:303-312（审计 #7）

## 死胡同
- 未确认 CLI 层 index 子命令 clap 定义的具体行（未读 run_task/clap 解析段）

## 置信度
high

## 统计
turns=3 · tool_calls=3 · duration=11094ms · tokens=12541
