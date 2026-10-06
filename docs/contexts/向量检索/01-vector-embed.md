# 向量嵌入（向量检索）

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
# 向量嵌入（向量检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量嵌入（src/vector/embed.rs）提供 EmbedClient：把代码块文本送进 Qwen3 嵌入服务的 OpenAI 兼容 /embeddings 接口，默认输出 1024 维（Matryoshka 降维，注释中允许实验上调到 4096），batch_size 默认 64、EMBED_CONCURRENCY=4（tokio Semaphore 限并发），查询侧通过 QUERY_INSTRUCT 拼指令前缀（instruct_query），文档侧不拼。响应 index 必须严格 0-based 对齐，错位即拒绝以防静默投毒（validate_index_alignment）。上游由 CLI 在 vector build 与 run 引导处从 cfg.vector 构造；下游被 build_vector_index 写 SQLite、RecallEngine::recall 做三段式召回、VectorSearchTool 对外暴露为 vector_search 工具。

## 证据列表
1. 向量嵌入模块基于 Qwen3 Embedding（OpenAI 兼容 /embeddings），默认 1024 维 Matryoshka 降维，batch_size 默认 64，EMBED_CONCURRENCY=4，查询侧加 Instruct 前缀
   - src/vector/embed.rs:1-7（审计 #2）
   - src/vector/embed.rs:12-16（审计 #2）
   - src/vector/embed.rs:42-49（审计 #2）
   - src/vector/embed.rs:52-64（审计 #2）
2. EmbedClient::embed 流程：去重（按 hash）→ split_batches 切批 → Semaphore(EMBED_CONCURRENCY=4) + JoinSet 并发 → 失败 2s 兜底重试一次 → 按批序号还原顺序回填
   - src/vector/embed.rs:33-39（审计 #2）
   - src/vector/embed.rs:71-146（审计 #2）
3. 单批请求经 embed_one 调用 /embeddings，build_request 固定 model/input/dimensions/encoding_format=float；响应 index 必须严格 0-based 对齐，validate_index_alignment 错位即拒绝以防静默投毒
   - src/vector/embed.rs:22-30（审计 #2）
   - src/vector/embed.rs:148-203（审计 #2）
   - src/vector/embed.rs:206-220（审计 #2）
4. instruct_query 给查询文本拼 QUERY_INSTRUCT 前缀（文档侧不拼），recall 链路上嵌入前调用
   - src/vector/embed.rs:11-20（审计 #2）
   - src/vector/recall.rs:127-139（审计 #4）
5. 上游：CLI 在 vector build（src/cli.rs:153）与 run 引导（src/cli.rs:557）两处构造 EmbedClient，参数从 cfg.vector.embed_model / embed_dims 取，base_url/api_key 缺省跟随 [llm]；embed 文本由 compose_input 按 EmbedMode::Raw / Composite 组装
   - src/cli.rs:140-172（审计 #19）
   - src/cli.rs:545-579（审计 #19）
6. 下游：build_vector_index 调 embed.embed 拿 Vec<Vec<f32>> 落 SQLite；RecallEngine::recall 单条嵌入做 HNSW/暴力召回；VectorSearchTool::execute 通过 engine.recall 对外暴露为 vector_search 工具
   - src/vector/build.rs:24-105（审计 #6）
   - src/vector/recall.rs:24-53（审计 #4）
   - src/vector/recall.rs:127-139（审计 #4）
   - src/tools/vector_search.rs:48-63（审计 #28）
   - src/vector/mod.rs:1-21（审计 #23）

## 死胡同
- 用 mode=regex 检索 "vector::embed|use.*embed::" 返回 0 命中（grep 语法不支持此模式）
- grep "recall_by_vector|\\.recall\\(" 返回 0 命中（regex 模式未启用）

## 置信度
high

## 统计
turns=8 · tool_calls=13 · duration=30246ms · tokens=100819


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
向量嵌入（src/vector/embed.rs）提供 EmbedClient：把代码块文本送进 Qwen3 嵌入服务的 OpenAI 兼容 /embeddings 接口，默认输出 1024 维（Matryoshka 降维，注释中允许实验上调到 4096），batch_size 默认 64、EMBED_CONCURRENCY=4（tokio Semaphore 限并发），查询侧通过 QUERY_INSTRUCT 拼指令前缀（instruct_query），文档侧不拼。响应 index 必须严格 0-based 对齐，错位即拒绝以防静默投毒（validate_index_alignment）。上游由 CLI 在 vector build 与 run 引导处从 cfg.vector 构造；下游被 build_vector_index 写 SQLite、RecallEngine::recall 做三段式召回、VectorSearchTool 对外暴露为 vector_search 工具。初稿此条结论与实际代码一致；初稿说 EmbedClient 默认 dimensions 1024 实为构造时不设默认值，由 cfg.vector.embed_dims 传入（默认 1024 来自 src/config.rs）。

## 证据列表
1. EmbedClient 默认 base_url/model/dimensions 由 cfg.vector 注入；dimensions 在 EmbedClient::new 构造时不设默认值，1024 来自 cfg.vector.embed_dims 默认；构造时 batch_size=64、HTTP 超时 120s、API key 走 bearer_auth。
   - src/vector/embed.rs:41-64（审计 #2）
   - src/config.rs:133-140（审计 #21）
2. EmbedClient::embed 流程：sha256 hash 去重 → split_batches 切批 → Semaphore(EMBED_CONCURRENCY=4) + JoinSet 并发 → 失败 2s 重试一次 → 按批序号还原顺序回填；进展每 25 批日志一次。
   - src/vector/embed.rs:71-146（审计 #2）
   - src/vector/embed.rs:11-20（审计 #2）
3. build_request 固定 model/input/dimensions/encoding_format=float；embed_one POST {base}/embeddings + bearer_auth；HTTP 429/5xx 标 retryable、reqwest 错也标 retryable；data[] 按 index 排序、长度不匹配直接报错。
   - src/vector/embed.rs:22-30（审计 #2）
   - src/vector/embed.rs:148-203（审计 #2）
4. validate_index_alignment（P005 R4.2）：响应 index 与请求位次必须严格 0-based 对齐，1-based 序号/乱序/空缺一律拒绝以防向量静默投毒；单测覆盖在 226-236。
   - src/vector/embed.rs:200-220（审计 #2）
   - src/vector/embed.rs:226-236（审计 #2）
5. QUERY_INSTRUCT 常量 = "Instruct: 给定代码库的自然语言问题，检索能回答它的代码片段\nQuery: "；instruct_query 仅在 recall 链路上调用，文档侧不拼前缀。
   - src/vector/embed.rs:11-20（审计 #2）
   - src/vector/recall.rs:127-139（审计 #4）
6. RecallEngine::recall 单条嵌入做三段式召回：embed(instruct_query) → recall_by_vector(HNSW k×3 超采 ef_search=64 → cosine 精确复算 → alive 墓碑过滤取 top-K)，无 HNSW 则降级全量暴力；< MIN_HNSW_SIZE=100k 直接走暴力（暖启动零负担）。
   - src/vector/recall.rs:99-139（审计 #4）
   - src/vector/recall.rs:13-22（审计 #4）
7. RecallEngine::open 做全行 dim 校验（不只是首行）和 model 校验，任一不一致降级暴力并省略 HNSW；tombstone 仅移出 alive 集合，TOMBSTONE_REBUILD_RATIO=0.2 触发 compact（丢死行 + 重编号 + 重建图）。
   - src/vector/recall.rs:34-97（审计 #4）
8. HNSW 参数：M=16、MAX_LAYER=16、EF_CONSTRUCTION=200、EF_SEARCH=64、OVERFETCH_FACTOR=3、MIN_HNSW_SIZE=100_000、TOMBSTONE_REBUILD_RATIO=0.2。
   - src/vector/recall.rs:13-22（审计 #4）
9. build_vector_index 编排：fingerprint 仓库键 → 读 codegraph.db 节点 → plan_chunks → 打开 vectors.db → 判 stale_index（model/dim/mode 任一变更即废旧向量）→ existing_hashes 增量复用 text_hash 未变 chunk → compose_input 组装（Raw/Composite）→ embed.embed → commit_build 单事务（GC 删除 + upsert + meta）。
   - src/vector/build.rs:24-105（审计 #9）
10. compose_input 两种模式：Raw = chunk.text（裸代码）；Composite = header + identifiers_of(text, cap=20) 去重保序封顶（4 字符起、ASCII 标识符）。
   - src/vector/compose.rs:14-51（审计 #25）
11. VectorStore 用 SQLite WAL，schema：chunks(id PK, file, line_start, line_end, symbol, kind, language, text_hash, embedding BLOB, dim) + 唯一索引 (file,symbol,line_start)；meta 表存 model/dim/mode；descriptions 表（兼容旧描述）。
   - src/vector/store.rs:29-79（审计 #23）
12. commit_build 同一事务内 remove_stale_on（按实际 DELETE 影响行数计）+ upsert_chunk_on 全部 + meta 三键写入；任一失败整体回滚，杜绝"旧块已删、新块写一半"中间态；cosine 实现点积+模长归一化（a/b 零模返回 0.0）。
   - src/vector/store.rs:201-301（审计 #23）
13. embedding BLOB 用 f32 little-endian 编/解码（vec_to_blob/blob_to_vec）；fingerprint = sha256(绝对路径 + .git/HEAD)[:16] hex；索引路径 = <repo>/.codesleuth/indexes/<fp>/vectors.db；hash_of 复用 sha256_hex。
   - src/vector/store.rs:29-39（审计 #23）
   - src/vector/store.rs:288-317（审计 #23）
14. CLI 两处构造 EmbedClient：index --vector（run_index_vector，src/cli.rs:132-182）与 run（setup_vector_layer，src/cli.rs:540-605）都从 cfg.vector 读 embed_model/embed_dims，base_url/api_key 缺省时通过 resolve_embed_endpoint 跟随 [llm]。
   - src/cli.rs:132-182（审计 #14）
   - src/cli.rs:540-605（审计 #14）
15. resolve_embed_endpoint 纯函数：[vector].base_url/api_key 缺省跟随 [llm].base_url/resolve_api_key；显式 [vector] 覆盖优先；测试断言双向（缺省跟随 + 显式覆盖）均成立。
   - src/cli.rs:524-536（审计 #14）
   - src/cli.rs:720-732（审计 #14）
16. 两处都拿 bootlock::acquire_guard(DEFAULT_TIMEOUT=300s, "向量索引构建") 跨进程串行化，与 graph 引导共用同一把锁，CS4016 INDEX_LOCKED 判负；run 路径在 drop(vguard) 之后才开库 + RecallEngine::open + 注册 VectorSearchTool，build/recall 失败降级记 audit "degraded"；run 暖启动在 src/cli.rs:607 直接消费 recall(ctx.task, 10) → format_recall_block 当首条 user 后缀。
   - src/cli.rs:564-621（审计 #14）
   - src/cli.rs:100-128（审计 #14）
17. bootlock：acquire 用 try_lock + 轮询，三态统一 CS4016 INDEX_LOCKED 判负（Won/Lost/超时），BootLockGuard Drop 中 unlock 双保险；acquire_guard 把 Lost 也归一为 CS4016（带"重跑本命令"hint）；DEFAULT_TIMEOUT=300s 供 index --vector、run --vector、graph 引导三处共用。
   - src/bootlock.rs:22-130（审计 #71）
18. VectorSearchTool name = "vector_search"，parameters = {query:string, k:int[1,10] default 8}；execute 缺 query 返 USER_INPUT，命中经 format_recall_block 渲染为「[语义召回 · 起步线索]...可能无关…必须 read 原文」块（带相似度、K≤10 截断）。
   - src/tools/vector_search.rs:25-63（审计 #18）
   - src/vector/recall.rs:141-159（审计 #4）
19. format_recall_block 注入固定头"[语义召回 · 起步线索]"、每行 "N. file:line_start-line_end symbol（相似度 x.xx）"、尾部免责"（召回内容未经验证；作为证据使用前必须 read 原文。）"；recall 返回 Vec<(Chunk, f32)>。
   - src/vector/recall.rs:141-159（审计 #4）
20. VectorConfig 字段：embed_model=Qwen/Qwen3-Embedding-8B、embed_dims=1024、embed_mode=composite、repomap_budget=24_000、base_url/api_key 缺省跟随 [llm]；Config 默认值在 src/config.rs:133-140 一处定义。
   - src/config.rs:92-140（审计 #21）
21. 模块对外门面：pub use build::{BuildReport, build_vector_index}; pub use compose::{EmbedMode, compose_input}; pub use embed::{EmbedClient, instruct_query}; pub use recall::RecallEngine; pub use store::{VectorStore, fingerprint, index_path}。
   - src/vector/mod.rs:1-21（审计 #11）
22. Chunk 字段：file/line_start/line_end/symbol/kind/language/header（白拿层 = 面包屑+签名+docstring）/text/text_hash；KEPT_KINDS = function/method/struct/class/interface/impl/trait；SKIP_DIRS 包含 .codesleuth/.codegraph；symbols_from_codegraph 由 codegraph.db 的 nodes 读符号边界作为切块真源。
   - src/vector/chunk.rs:13-75（审计 #44）

## 死胡同
- grep "recall_by_vector|\\.recall\\(" 默认 plain 模式返回 0 命中（regex 模式未启用），改 mode=regex 后 recall_by_vector 在 src/vector/recall.rs:100,135 可命中。
- grep "register.*vector_search|setup_vector_layer" 默认 0/0 命中，改 mode=regex 后 setup_vector_layer 在 src/cli.rs:306,540 命中。
- grep "format_recall_block|recall\(.*ctx\.task" 默认 0/0 命中，改 regex 后命中 src/cli.rs:621、src/vector/recall.rs:142、src/tools/vector_search.rs:62。
- 未读 src/vector/repomap.rs 实现细节（与本子命令数据流正交，仅在 setup_vector_layer 中消费）——非侦察重点。

## 置信度
high

## 统计
turns=15 · tool_calls=27 · duration=204513ms · tokens=568735


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量嵌入（src/vector/embed.rs）提供 EmbedClient：把代码块文本送进 Qwen3 嵌入服务的 OpenAI 兼容 /embeddings 接口，默认输出 1024 维（Matryoshka 降维，注释中允许实验上调到 4096），batch_size 默认 64、EMBED_CONCURRENCY=4（tokio Semaphore 限并发），查询侧通过 QUERY_INSTRUCT 拼指令前缀（instruct_query），文档侧不拼。响应 index 必须严格 0-based 对齐，错位即拒绝以防静默投毒（validate_index_alignment）。上游由 CLI 在 vector build 与 run 引导处从 cfg.vector 构造；下游被 build_vector_index 写 SQLite、RecallEngine::recall 做三段式召回、VectorSearchTool 对外暴露为 vector_search 工具。

## 证据列表
1. 向量嵌入模块基于 Qwen3 Embedding（OpenAI 兼容 /embeddings），默认 1024 维 Matryoshka 降维，batch_size 默认 64，EMBED_CONCURRENCY=4，查询侧加 Instruct 前缀
   - src/vector/embed.rs:1-7（审计 #2）
   - src/vector/embed.rs:12-16（审计 #2）
   - src/vector/embed.rs:42-49（审计 #2）
   - src/vector/embed.rs:52-64（审计 #2）
2. EmbedClient::embed 流程：去重（按 hash）→ split_batches 切批 → Semaphore(EMBED_CONCURRENCY=4) + JoinSet 并发 → 失败 2s 兜底重试一次 → 按批序号还原顺序回填
   - src/vector/embed.rs:33-39（审计 #2）
   - src/vector/embed.rs:71-146（审计 #2）
3. 单批请求经 embed_one 调用 /embeddings，build_request 固定 model/input/dimensions/encoding_format=float；响应 index 必须严格 0-based 对齐，validate_index_alignment 错位即拒绝以防静默投毒
   - src/vector/embed.rs:22-30（审计 #2）
   - src/vector/embed.rs:148-203（审计 #2）
   - src/vector/embed.rs:206-220（审计 #2）
4. instruct_query 给查询文本拼 QUERY_INSTRUCT 前缀（文档侧不拼），recall 链路上嵌入前调用
   - src/vector/embed.rs:11-20（审计 #2）
   - src/vector/recall.rs:127-139（审计 #4）
5. 上游：CLI 在 vector build（src/cli.rs:153）与 run 引导（src/cli.rs:557）两处构造 EmbedClient，参数从 cfg.vector.embed_model / embed_dims 取，base_url/api_key 缺省跟随 [llm]；embed 文本由 compose_input 按 EmbedMode::Raw / Composite 组装
   - src/cli.rs:140-172（审计 #19）
   - src/cli.rs:545-579（审计 #19）
6. 下游：build_vector_index 调 embed.embed 拿 Vec<Vec<f32>> 落 SQLite；RecallEngine::recall 单条嵌入做 HNSW/暴力召回；VectorSearchTool::execute 通过 engine.recall 对外暴露为 vector_search 工具
   - src/vector/build.rs:24-105（审计 #6）
   - src/vector/recall.rs:24-53（审计 #4）
   - src/vector/recall.rs:127-139（审计 #4）
   - src/tools/vector_search.rs:48-63（审计 #28）
   - src/vector/mod.rs:1-21（审计 #23）

## 死胡同
- 用 mode=regex 检索 "vector::embed|use.*embed::" 返回 0 命中（grep 语法不支持此模式）
- grep "recall_by_vector|\\.recall\\(" 返回 0 命中（regex 模式未启用）

## 置信度
high

## 统计
turns=8 · tool_calls=13 · duration=30246ms · tokens=100819
