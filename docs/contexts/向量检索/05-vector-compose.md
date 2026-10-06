# 向量组装（向量检索）

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
# 向量组装（向量检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量组装（src/vector/compose.rs）提供 compose_input(chunk, mode) 将每个代码 chunk 序列化为嵌入输入；EmbedMode 含 Raw（返回 chunk.text 全文）与 Composite（白拿层 header + 标识符层去重封顶 20，默认模式）两种。模式由 CLI 从 cfg.vector.embed_mode 解析（"raw"→Raw，否则→Composite），由 build_vector_index 在嵌入循环中调用，结果送入 EmbedClient::embed 并以 mode 字符串写入索引 meta 参与增量复用与作废判定。

## 证据列表
（无结构化发现——降级报告）

## 死胡同
- grep 默认 plain 模式对 compose_input/EmbedMode 返回 0 命中（0/82 文件），改用 regex 后才定位；非实质性死胡同。
- identifiers_of 仅在 compose.rs 内部被引用（grep 1/82 文件），未在其他模块复用，故下游仅写其作为 compose_input Composite 分支的内部依赖。

## 置信度
high

## 统计
turns=6 · tool_calls=10 · duration=31890ms · tokens=50248


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
向量组装（src/vector/compose.rs）以 compose_input(chunk, EmbedMode) 把每个 Chunk 序列化为嵌入输入。EmbedMode 二选一：Raw 直接返回 chunk.text 全文；Composite（默认）以 format!("{}\n{}", header, identifiers_of(text, 20).join(", ")) 拼接 header（面包屑+签名+docstring，不含函数体）与从 text 中按 ASCII [A-Za-z_]\w{3,} 抽取、去重保序、封顶 20 的标识符串。CLI 在两处将 cfg.vector.embed_mode 字符串解析为 EmbedMode：run_index_vector（src/cli.rs:149-152, index --vector）与 setup_vector_layer（src/cli.rs:546-549, run --vector），规则都是 "raw"→Raw、其它（含默认 "composite"）→Composite。组装在 build_vector_index（src/vector/build.rs:74）嵌入循环中调用，结果送 EmbedClient::embed（src/vector/build.rs:78），最终 mode 字符串以 format!("{mode:?}") 经 commit_build（src/vector/build.rs:86-92, src/vector/store.rs:217-220）写入 vectors.db 的 meta("mode") 表，参与下一次构建的 stale_index 判定（src/vector/build.rs:43-47）。查询侧 RecallEngine::recall 走 instruct_query 指令前缀（src/vector/recall.rs:129），不经 compose_input——这是与初稿需要点明的关键不对称。初稿小瑕疵："白拿层 header" 实际代码字段是 header（chunk.rs:71-72，无"白拿层"命名），功能等价但用语不准。</answer>

## 证据列表
1. compose_input 是 src/vector/compose.rs:42 的 pub fn，按 EmbedMode 分两路：Raw 返回 chunk.text.clone()（compose.rs:44），Composite 走 format!("{}\n{}", header, ids) 拼接（compose.rs:45-49）。
   - src/vector/compose.rs:42-51（审计 #2）
   - src/vector/compose.rs:8-12（审计 #2）
2. EmbedMode = Raw | Composite 枚举由 #[derive(Debug, Clone, Copy, PartialEq, Eq)] 生成，Debug 输出字面量 "Raw"/"Composite" 直接写入 meta。
   - src/vector/compose.rs:8-12（审计 #2）
   - src/vector/build.rs:44（审计 #4）
3. identifiers_of 手写 ASCII 标识符扫描器：接受 [A-Za-z_] 起头，续 [A-Za-z0-9_]，token.len()>=4 门槛（按 char 数），去重保序，cap 封顶；多字节 UTF-8 标识符会被整 token 跳过。
   - src/vector/compose.rs:15-39（审计 #2）
   - src/vector/compose.rs:21-27（审计 #2）
4. Composite 模式 cap=20 硬编码在 compose_input 调用处（compose.rs:48），不是配置项。
   - src/vector/compose.rs:48（审计 #2）
5. Chunk.header 字段定义"白拿层"实际是"面包屑+签名+docstring（不含函数体）"——代码中无"白拿层"命名，但功能等价；初稿用语需修正。
   - src/vector/chunk.rs:71-72（审计 #30）
   - src/vector/compose.rs:4（审计 #2）
6. CLI 在两处解析 embed_mode 字符串：run_index_vector（src/cli.rs:149-152, index --vector）与 setup_vector_layer（src/cli.rs:546-549, run --vector），解析规则完全相同（"raw"→Raw、其它→Composite），非 "raw" 拼写错误也静默回落到 Composite。
   - src/cli.rs:149-152（审计 #7）
   - src/cli.rs:546-549（审计 #7）
7. build_vector_index 用 VectorStore::get_meta("mode") 与新 mode 字符串比对（format!("{mode:?}")）判定 stale_index，model/dim/mode 任一变更即全量重嵌。
   - src/vector/build.rs:41-47（审计 #4）
   - src/vector/store.rs:81（审计 #35）
8. commit_build 在单一 SQLite 事务内完成 GC 删除 + 全部 upsert + meta 翻新（model/dim/mode），任一失败整体回滚；网络调用严格在事务外。
   - src/vector/store.rs:201-232（审计 #35）
   - src/vector/store.rs:217-220（审计 #35）
   - src/vector/build.rs:71（审计 #4）
9. EmbedClient::embed 走 4 路 tokio Semaphore 并发，batch_size=64，瞬时故障 2s 重试一次（无指数退避），去重后按批序号还原；validate_index_alignment 严格 0-based 校验错位即拒绝防投毒。
   - src/vector/embed.rs:16（审计 #4）
   - src/vector/embed.rs:72-146（审计 #4）
   - src/vector/embed.rs:208-220（审计 #4）
10. RecallEngine::recall 走 instruct_query 指令前缀（src/vector/embed.rs:18-20）不经 compose_input；RecallEngine::open 只校验 model/dim（src/vector/recall.rs:36, 38）不读 mode——文档侧与查询侧嵌入空间天然不对称。
   - src/vector/recall.rs:128-129（审计 #4）
   - src/vector/recall.rs:36-38（审计 #4）
   - src/vector/embed.rs:11-20（审计 #4）
11. VectorConfig.embed_mode 默认 "composite"（src/config.rs:136），由 FileVector.embed_mode (Option) 在配置加载层覆写（src/config.rs:35, 271-272）；env 层已整体移除。
   - src/config.rs:35（审计 #11）
   - src/config.rs:98（审计 #11）
   - src/config.rs:133-140（审计 #11）
   - src/config.rs:265-272（审计 #11）
12. 两个 CLI 入口都通过 bootlock::acquire_guard(repo_abs, DEFAULT_TIMEOUT, "向量索引构建") 串行化同一仓库的构建竞争。
   - src/cli.rs:162-166（审计 #7）
   - src/cli.rs:565-569（审计 #7）
13. vector 模块公共导出在 src/vector/mod.rs:16-21，EmbedMode/compose_input 与 build_vector_index/VectorStore/RecallEngine/EmbedClient 同层对外；identifiers_of 在 mod.rs 未重导出，仅供 compose_input 内部使用。
   - src/vector/mod.rs:16-21（审计 #4）
   - src/vector/mod.rs:18（审计 #4）
14. build_vector_index 编排：plan_chunks → existing_hashes 增量复用 → compose_input 组装 → embed.embed 批并发去重 → commit_build 原子落库；网络调用严格在事务外。
   - src/vector/build.rs:25-105（审计 #4）
   - src/vector/build.rs:71-78（审计 #4）
   - src/vector/build.rs:86-92（审计 #4）
15. 请求体构造走 build_request(model, texts, dimensions) 固定四键 {model, input, dimensions, encoding_format:"float"}，响应必须 OpenAI 兼容的 {data:[{index, embedding:[f32,...]}]} 形态。
   - src/vector/embed.rs:22-30（审计 #4）
   - src/vector/embed.rs:149-155（审计 #4）
   - src/vector/embed.rs:178-192（审计 #4）
16. stale_index 判定是 model/dim/mode 三键 OR 组合，任一变更即旧向量全废（build.rs 注释："嵌入输入变了，复用即投毒"），杜绝输入不同但向量复用的静默投毒。
   - src/vector/build.rs:40-47（审计 #4）
17. 嵌入输入仅对文档侧生效；查询侧独立走 instruct_query 指令前缀（embed.rs:18-20），不通过 compose_input，这是 Qwen3 指令感知官方用法（embed.rs:6 注释）。
   - src/vector/embed.rs:6（审计 #4）
   - src/vector/embed.rs:11-20（审计 #4）
   - src/vector/recall.rs:128-129（审计 #4）
18. 降级路径：build_vector_index 失败时 setup_vector_layer 走 ctx.audit.record("degraded", ...) 留痕，RecallEngine::recall 失败同款（cli.rs:584-587, 612-615），符合 P005 R5.2 弹性降级原则。
   - src/cli.rs:584-587（审计 #7）
   - src/cli.rs:612-615（审计 #7）
19. module 公共导出汇总：VectorStore/RecallEngine/build_vector_index/EmbedClient/EmbedMode/Chunk/SymbolRow/plan_chunks/fingerprint/index_path/instruct_query；build/chunk/compose/embed/recall/store/repomap 子模块（mod.rs:16-21）。
   - src/vector/mod.rs:16-21（审计 #4）

## 死胡同
- 未找到 identifiers_of 在 compose.rs 之外的任何 use 引用——确认其仅作为 compose_input Composite 分支的内部辅助（grep regex 命中全在 compose.rs + docs）
- 未找到 RecallEngine::open 读取 meta("mode") 的位置——确认其只校验 model/dim（src/vector/recall.rs:36, 38），不读 mode

## 置信度
medium

## 统计
turns=9 · tool_calls=15 · duration=163010ms · tokens=240918


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量组装（src/vector/compose.rs）提供 compose_input(chunk, mode) 将每个代码 chunk 序列化为嵌入输入；EmbedMode 含 Raw（返回 chunk.text 全文）与 Composite（白拿层 header + 标识符层去重封顶 20，默认模式）两种。模式由 CLI 从 cfg.vector.embed_mode 解析（"raw"→Raw，否则→Composite），由 build_vector_index 在嵌入循环中调用，结果送入 EmbedClient::embed 并以 mode 字符串写入索引 meta 参与增量复用与作废判定。

## 证据列表
（无结构化发现——降级报告）

## 死胡同
- grep 默认 plain 模式对 compose_input/EmbedMode 返回 0 命中（0/82 文件），改用 regex 后才定位；非实质性死胡同。
- identifiers_of 仅在 compose.rs 内部被引用（grep 1/82 文件），未在其他模块复用，故下游仅写其作为 compose_input Composite 分支的内部依赖。

## 置信度
high

## 统计
turns=6 · tool_calls=10 · duration=31890ms · tokens=50248
