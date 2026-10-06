# 嵌入供应商独立配置（向量语义检索）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`04-embedding-provider-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 嵌入供应商独立配置（向量语义检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
「嵌入供应商独立配置」让向量语义检索的嵌入服务脱离主 LLM 单独选供应商：通过 config 的 [vector] 段（VectorConfig）独立指定嵌入 base_url、api_key、模型、维度与 raw/composite 模式，缺省时自动回退跟随主 [llm] 段，解除 chat 与 embedding 必须同一家供应商的限制。装配时 resolve_embed_endpoint 做 [vector] 优先、[llm] 兜底的解析，再用解析结果构造 EmbedClient 供索引构建使用。

## 证据列表
1. VectorConfig 收编嵌入专用 base_url/api_key（None=跟随 [llm]）及 embed_model/embed_dims/embed_mode 等参数
   - src/config.rs:78-90（审计 #4）
2. resolve_embed_endpoint 实现 [vector] 优先、[llm] 兜底的端点解析
   - src/cli.rs:518-530（审计 #6）
3. setup_vector_layer 调用 resolve_embed_endpoint 后用结果构造 EmbedClient
   - src/cli.rs:545-556（审计 #6）
4. EmbedClient 持有独立 base_url/api_key/model/dimensions，经 OpenAI 兼容 /embeddings 调用嵌入服务
   - src/vector/embed.rs:41-52（审计 #11）
5. 既有初稿文档与本结论一致（结论与证据链可互证）
   - docs/向量语义检索/04-embedding-config.md:5-7（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=3 · tool_calls=5 · duration=8716ms · tokens=13403

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
初稿结论核实无误并细化：嵌入供应商独立配置由 [vector].base_url/api_key（Option，None=跟随 [llm]）+ resolve_embed_endpoint（cli.rs:520-530）实现，两入口（run 的 setup_vector_layer cli.rs:534-659、index --vector 的 run_index_vector cli.rs:128-178）共用。关键新发现：EmbedClient 同时服务索引构建与查询侧召回（recall.rs:129 用同一客户端嵌查询），独立供应商对两条路径同时生效。坑：resolve_api_key 只认 [llm].api_key（config.rs:458-468），独立 vector key 不豁免主 key 缺失；embed_mode 非"raw"静默当 composite（cli.rs:540-543）；model/dim/mode 漂移触发全量重嵌（build.rs:40-47）；CLI --base-url 不能覆盖 vector 端点。

## 证据列表
1. VectorConfig 收编嵌入专用 base_url/api_key（None=跟随 [llm]），默认值 embed_model=Qwen/Qwen3-Embedding-8B、dims=1024、mode=composite；配置三层合并（默认←全局←项目←CLI，CLI 只覆盖 [llm]）
   - src/config.rs:78-90,107-133（审计 #2）
   - src/config.rs:203-243（审计 #2）
   - src/cli.rs:520-530（审计 #4）
2. resolve_embed_endpoint 实现 [vector] 优先、[llm] 兜底；两个入口（setup_vector_layer 与 run_index_vector）都经它构造 EmbedClient
   - src/cli.rs:518-556（审计 #4）
   - src/cli.rs:128-178（审计 #4）
   - src/config.rs:458-468（审计 #2）
3. EmbedClient 持独立 base_url/api_key/model/dimensions，POST {base_url}/embeddings（bearer_auth，batch=64，并发4，失败2s重试一次，429/5xx retryable）；查询侧 RecallEngine::recall 用同一客户端加 instruct_query 前缀嵌入
   - src/vector/embed.rs:41-64,72-146,149-177（审计 #9）
   - src/vector/recall.rs:128-139（审计 #11）
   - src/vector/build.rs:25-105（审计 #11）
4. 落库 <repo>/.codesleuth/indexes/<fingerprint>/vectors.db（SQLite，commit_build 原子事务，meta 记 model/dim/mode）；入口A构建失败弹性降级+审计degraded，入口B直接报错；引导锁竞争判负 INDEX_LOCKED；召回块注入首条消息、命中喂 repo-map 种子
   - src/cli.rs:559-599,601-615（审计 #4）
   - src/cli.rs:299-328,361-364（审计 #4）
   - src/vector/store.rs:303-312（审计 #50）
   - src/vector/build.rs:40-47,63（审计 #11）
5. config get 对 vector.base_url/api_key 展示同口径 llm 兜底；但 resolve_api_key 只认 [llm].api_key，配置独立 vector key 不豁免主 key 缺失；embed_mode 非"raw"静默当 composite；RecallEngine::open 全行维度校验，model 不符降级暴力省 HNSW；≤10万块暴力精确召回
   - src/config.rs:341-367（审计 #2）
   - src/config.rs:458-468（审计 #2）
   - src/cli.rs:540-543（审计 #4）
   - src/vector/recall.rs:36-49（审计 #11）
   - src/vector/recall.rs:99-125（审计 #11）
6. 初稿文档与代码结论一致，无初稿错误；初稿仅遗漏查询侧召回同样走独立嵌入端点这一事实
   - docs/contexts/向量语义检索/04-embedding-provider.md:5-18（审计 #6）

## 死胡同
- grep 查询 instruct_query/dimensions 单独关键词零命中（grep 对 | 交替按字面处理），改用 RecallEngine 关键词命中
- 未深读 store.rs 全文（经 explore 拿到关键段 303-312）

## 置信度
high

## 统计
turns=14 · tool_calls=19 · duration=176663ms · tokens=342267


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
「嵌入供应商独立配置」让向量语义检索的嵌入服务脱离主 LLM 单独选供应商：通过 config 的 [vector] 段（VectorConfig）独立指定嵌入 base_url、api_key、模型、维度与 raw/composite 模式，缺省时自动回退跟随主 [llm] 段，解除 chat 与 embedding 必须同一家供应商的限制。装配时 resolve_embed_endpoint 做 [vector] 优先、[llm] 兜底的解析，再用解析结果构造 EmbedClient 供索引构建使用。

## 证据列表
1. VectorConfig 收编嵌入专用 base_url/api_key（None=跟随 [llm]）及 embed_model/embed_dims/embed_mode 等参数
   - src/config.rs:78-90（审计 #4）
2. resolve_embed_endpoint 实现 [vector] 优先、[llm] 兜底的端点解析
   - src/cli.rs:518-530（审计 #6）
3. setup_vector_layer 调用 resolve_embed_endpoint 后用结果构造 EmbedClient
   - src/cli.rs:545-556（审计 #6）
4. EmbedClient 持有独立 base_url/api_key/model/dimensions，经 OpenAI 兼容 /embeddings 调用嵌入服务
   - src/vector/embed.rs:41-52（审计 #11）
5. 既有初稿文档与本结论一致（结论与证据链可互证）
   - docs/向量语义检索/04-embedding-config.md:5-7（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=3 · tool_calls=5 · duration=8716ms · tokens=13403
