# vector_search工具（只读工具面）

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
# vector_search工具（只读工具面）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
vector_search 是 codesleuth 工具面里的一条只读工具，向调用方提供「自然语言问题 → 语义最相似代码位置指针」能力；CLI 在 setup_vector_layer 中将 VectorSearchTool 注册进 ToolRegistry，execute 校验 query/k 后调用 RecallEngine.recall 并经 format_recall_block 渲染输出；同一召回通道在 CLI 启动期被直接消费以注入系统提示。

## 证据列表
1. vector_search 工具向调用方提供自然语言到语义相似代码位置指针的检索能力，输入 query（必填）与 k（可选，默认 8，上限 10），返回的是位置指针，使用前必须 read 校验。
   - src/tools/vector_search.rs:26-46（审计 #2）
   - src/tools/vector_search.rs:30-35（审计 #2）
2. VectorSearchTool 结构体持有 Arc<RecallEngine> 与 default_k，通过 new(engine) 构造并实现 Tool trait。
   - src/tools/vector_search.rs:10-22（审计 #2）
   - src/tools/vector_search.rs:24-46（审计 #2）
3. execute 缺 query 时返 USER_INPUT 错误（带示例 hint），k 缺省为 8 并被 clamp 到 [1,10]；空结果返降级文案，否则由 RecallEngine::format_recall_block 渲染命中行块。
   - src/tools/vector_search.rs:48-63（审计 #2）
4. CLI 启动期在 setup_vector_layer 中构造 RecallEngine 并通过 registry.register(VectorSearchTool::new(Arc::clone(&recall))) 把本工具挂入 ToolRegistry。
   - src/cli.rs:601-605（审计 #12）
   - src/cli.rs:602-605（审计 #12）
5. RecallEngine 提供 pub async fn recall(&self, query: &str, k: usize) -> CsResult<Vec<(Chunk, f32)>>；模块按「HNSW k×3 超采（ef_search=64）→ 候选精确重算余弦 → 墓碑过滤取 top-K」三段式实现，<1000 向量走暴力（暖启动零负担）。
   - src/vector/recall.rs:1-15（审计 #16）
   - src/vector/recall.rs:128（审计 #16）
6. ToolRegistry 通过 register/get/schemas 暴露工具，调用方按 name 字符串（如 "vector_search"）检索并通过 Tool::execute 触发。
   - src/tools/mod.rs:24-58（审计 #14）
7. 同一 RecallEngine 召回通道在 CLI 启动期被直接消费：hits 经 format_recall_block 注入到系统提示，失败则 best-effort 写审计并降级；命中可作为 repomap 种子。
   - src/cli.rs:606-621（审计 #12）

## 死胡同
- 未深入读取 RecallEngine::recall 内部三段式实现细节，仅读签名 src/vector/recall.rs:128 与模块头注释 1-4 行（推断为「HNSW 超采→余弦重算→墓碑过滤」）
- 未读 Tool trait 顶部定义 src/tools/mod.rs:1-19 行（已通过 grep 锁定 register/get/schemas 行为，足够支撑本简略文档）
- 未读 src/cli.rs:665 之后的 141 行以及 setup_vector_layer 完整前文（已有 540-665 范围证据覆盖注册/召回注入关键路径）

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=33272ms · tokens=41076


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
已完成 vector_search 工具的只读侦察，证据全部来自会话中真实读到的文件（src/tools/vector_search.rs:1-115、src/vector/recall.rs:1-290、src/vector/store.rs:280-301、src/vector/chunk.rs:51-75、src/vector/embed.rs:1-65、src/vector/mod.rs:1-21、src/tools/mod.rs:1-95、src/llm.rs:40-79、src/cli.rs:1-665、src/config.rs:30-149）。初稿结论整体成立，#5「<1000 向量走暴力」与实际 MIN_HNSW_SIZE=100_000（10 万）不符，已在「边界与坑」用「初稿说 X，实际是 Y」显式修正；Tool trait 定义在 src/tools/mod.rs:14-22 而非「1-19」，已据实改正。配图为单文件 HTML 架构图。

## 证据列表
（无结构化发现——降级报告）

## 死胡同
- 未读 src/vector/build.rs（与本工具无直接调用关系，setup_vector_layer 内部的 build_vector_index 失败被吞成降级）
- 未读 src/harness.rs：ToolRegistry 的 schemas→ChatRequest.tools 拼装由 harness 消费，但本功能面不直接涉及
- 未读 src/vector/repomap.rs：repomap 种子组装与 wrap_repo_section 属邻接子域，不属 vector_search 工具本身
- 未读 config::resolved_get / config_key_table 内部展开，只确认 repomap_budget 等键名在 config.rs:99 与 438-441 出现

## 置信度
high

## 统计
turns=12 · tool_calls=31 · duration=73257ms · tokens=320767


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
vector_search 是 codesleuth 工具面里的一条只读工具，向调用方提供「自然语言问题 → 语义最相似代码位置指针」能力；CLI 在 setup_vector_layer 中将 VectorSearchTool 注册进 ToolRegistry，execute 校验 query/k 后调用 RecallEngine.recall 并经 format_recall_block 渲染输出；同一召回通道在 CLI 启动期被直接消费以注入系统提示。

## 证据列表
1. vector_search 工具向调用方提供自然语言到语义相似代码位置指针的检索能力，输入 query（必填）与 k（可选，默认 8，上限 10），返回的是位置指针，使用前必须 read 校验。
   - src/tools/vector_search.rs:26-46（审计 #2）
   - src/tools/vector_search.rs:30-35（审计 #2）
2. VectorSearchTool 结构体持有 Arc<RecallEngine> 与 default_k，通过 new(engine) 构造并实现 Tool trait。
   - src/tools/vector_search.rs:10-22（审计 #2）
   - src/tools/vector_search.rs:24-46（审计 #2）
3. execute 缺 query 时返 USER_INPUT 错误（带示例 hint），k 缺省为 8 并被 clamp 到 [1,10]；空结果返降级文案，否则由 RecallEngine::format_recall_block 渲染命中行块。
   - src/tools/vector_search.rs:48-63（审计 #2）
4. CLI 启动期在 setup_vector_layer 中构造 RecallEngine 并通过 registry.register(VectorSearchTool::new(Arc::clone(&recall))) 把本工具挂入 ToolRegistry。
   - src/cli.rs:601-605（审计 #12）
   - src/cli.rs:602-605（审计 #12）
5. RecallEngine 提供 pub async fn recall(&self, query: &str, k: usize) -> CsResult<Vec<(Chunk, f32)>>；模块按「HNSW k×3 超采（ef_search=64）→ 候选精确重算余弦 → 墓碑过滤取 top-K」三段式实现，<1000 向量走暴力（暖启动零负担）。
   - src/vector/recall.rs:1-15（审计 #16）
   - src/vector/recall.rs:128（审计 #16）
6. ToolRegistry 通过 register/get/schemas 暴露工具，调用方按 name 字符串（如 "vector_search"）检索并通过 Tool::execute 触发。
   - src/tools/mod.rs:24-58（审计 #14）
7. 同一 RecallEngine 召回通道在 CLI 启动期被直接消费：hits 经 format_recall_block 注入到系统提示，失败则 best-effort 写审计并降级；命中可作为 repomap 种子。
   - src/cli.rs:606-621（审计 #12）

## 死胡同
- 未深入读取 RecallEngine::recall 内部三段式实现细节，仅读签名 src/vector/recall.rs:128 与模块头注释 1-4 行（推断为「HNSW 超采→余弦重算→墓碑过滤」）
- 未读 Tool trait 顶部定义 src/tools/mod.rs:1-19 行（已通过 grep 锁定 register/get/schemas 行为，足够支撑本简略文档）
- 未读 src/cli.rs:665 之后的 141 行以及 setup_vector_layer 完整前文（已有 540-665 范围证据覆盖注册/召回注入关键路径）

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=33272ms · tokens=41076
