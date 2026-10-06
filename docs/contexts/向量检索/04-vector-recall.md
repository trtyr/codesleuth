# 向量召回引擎（向量检索）

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
# 向量召回引擎（向量检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量召回引擎（src/vector/recall.rs 中的 RecallEngine）为上层工具与 CLI 任务编排提供「自然语言/向量 → top-K 余弦相似度代码位置」能力；采用「≤10 万向量走暴力、否则 HNSW 三段式（k×3 超采 → 余弦精确复算 → 墓碑过滤取 top-K）」策略，由 VectorSearchTool 与 cli::setup_vector_layer 共同消费。

## 证据列表
1. RecallEngine 结构定义在 src/vector/recall.rs:25-30，持 rows（真相快照）、alive（活跃集合）与可选 HNSW 影子图。
   - src/vector/recall.rs:25-30（审计 #2）
2. 三段式策略与常量定义明确：HNSW_EF_SEARCH=64、OVERFETCH_FACTOR=3、MIN_HNSW_SIZE=100_000、TOMBSTONE_REBUILD_RATIO=0.2。
   - src/vector/recall.rs:13-22（审计 #2）
3. RecallEngine::open 是构造入口，模型/维度不符时跳过 HNSW、降级为暴力；满足 MIN_HNSW_SIZE 才调用 build_hnsw。
   - src/vector/recall.rs:34-66（审计 #2）
4. recall_by_vector 实现 HNSW 三段式（k×OVERFETCH_FACTOR 超采 → cosine 精确复算 → alive 墓碑过滤 → 截断 top-K）与无 HNSW 时的暴力回退两套分支。
   - src/vector/recall.rs:100-125（审计 #2）
5. recall 异步入口负责 query 嵌入（instruct_query 指令前缀）并把 row 索引映射回 Chunk。
   - src/vector/recall.rs:128-139（审计 #2）
6. 墓碑与压缩：tombstone 维护 alive 集合；compact 在 needs_compact 为真时丢死行、重编号、必要时重建 HNSW。
   - src/vector/recall.rs:69-97（审计 #2）
7. format_recall_block 注入固定提示头「[语义召回 · 起步线索]」、带「可能无关」与「必须 read 原文」免责。
   - src/vector/recall.rs:141-159（审计 #2）
8. VectorSearchTool 持有 Arc<RecallEngine>，在 execute 中调用 engine.recall 并用 RecallEngine::format_recall_block 渲染。
   - src/tools/vector_search.rs:10-22（审计 #4）
   - src/tools/vector_search.rs:48-63（审计 #4）
9. CLI 侧 setup_vector_layer 用 VectorStore 构造 Arc<RecallEngine>，注入 VectorSearchTool，并直接消费 recall + format_recall_block 当任务提示种子。
   - src/cli.rs:602-621（审计 #17）

## 死胡同
- grep 用 plain 模式对 RecallEngine::open 检索返回 0 命中，改用 regex 模式后正常返回（已规避）。
- callers 查询 RecallEngine::tombstone 仅返回 2 个测试调用方（recall.rs:219、245），未发现生产调用方；据此判断 compact 路径在当前仓库内为预留入口。
- 未读取 src/vector/chunk.rs / src/vector/embed.rs / src/vector/store.rs 内部实现细节，仅依赖 recall.rs 中的引用与 import 关系，外部模块的细节作为隐含依赖未深入。

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=37969ms · tokens=54582


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
向量召回引擎（RecallEngine，src/vector/recall.rs:25）为 codesleuth 提供「自然语言/向量 → top-K 余弦相似度代码位置」能力，真相在 SQLite（vectors.db，src/vector/store.rs:41），HNSW 影子图按规模自适应（≤10 万向量走精确暴力、>10 万才建图，MIN_HNSW_SIZE=100_000）。两路消费：① CLI 启动期 setup_vector_layer（src/cli.rs:540）经 build_vector_index 补建索引 → RecallEngine::open → 注册 VectorSearchTool（src/tools/vector_search.rs:10）→ 召回 task(k=10) 拼首条消息后缀注入 Harness（src/cli.rs:367）；② LLM agent 经 Tool::execute 触发同一 VectorSearchTool（src/tools/vector_search.rs:48-63）。召回走 EmbedClient（src/vector/embed.rs:42, instruct_query 加 QUERY_INSTRUCT 前缀）单条嵌入后，HNSW 路径 k×3 超采→cosine 精确复算→墓碑过滤取 top-K（src/vector/recall.rs:101-114），暴力路径 alive 全扫 cosine（src/vector/recall.rs:115-124），结果由 format_recall_block 渲染（src/vector/recall.rs:141-159, "[语义召回 · 起步线索]" 模板）。配置受 [vector] 块（embed_model/embed_dims/embed_mode/repomap_budget/base_url/api_key，src/config.rs:92-104）控制，--vector 旗标启用（src/cli.rs:48-49）。失败降级策略：build/recall/vector_layer 错误进 audit "degraded" 留痕（cli.rs:584-587/612-615/328-331），INDEX_LOCKED 判负退出（cli.rs:321-324, D014），embed 瞬时失败 2s 重试一次（src/vector/embed.rs:106-112）。tombstone/compact 路径在当前仓库无生产调用方（callers 仅返回测试，recall.rs:219/245），为预留入口。

## 证据列表
1. RecallEngine 结构定义持 rows/alive/可选 HNSW 影子图，三段式策略常量 HNSW_EF_SEARCH=64、OVERFETCH_FACTOR=3、MIN_HNSW_SIZE=100_000、TOMBSTONE_REBUILD_RATIO=0.2 全部在 src/vector/recall.rs:13-30 声明
   - src/vector/recall.rs:13-30（审计 #2）
2. RecallEngine::open 是构造入口，从 VectorStore::load_all 全量加载（text 留空仅取位置指针），model/dim 不匹配跳过 HNSW 降级暴力，≥MIN_HNSW_SIZE 才 build_hnsw
   - src/vector/recall.rs:34-66（审计 #2）
   - src/vector/store.rs:241-285（审计 #19）
3. recall_by_vector 实现 HNSW 三段式（k×3 超采 → cosine 精确复算 → alive 墓碑过滤 → 截断 top-K）与无 HNSW 时的暴力回退两套分支；HNSW 走 ef_search=64，暴力走 alive 全扫 cosine 排序截 k
   - src/vector/recall.rs:100-125（审计 #2）
   - src/vector/store.rs:289-301（审计 #19）
4. recall 异步入口负责 query 嵌入（instruct_query 指令前缀）并把 row 索引映射回 Chunk；嵌空返 INDEX_NOT_AVAILABLE
   - src/vector/recall.rs:128-139（审计 #2）
   - src/vector/embed.rs:18-20（审计 #16）
5. 墓碑与压缩：tombstone 维护 alive 集合；compact 在 needs_compact 为真时丢死行、重编号、必要时重建 HNSW；needs_compact 阈值严格 >0.2
   - src/vector/recall.rs:69-97（审计 #2）
6. format_recall_block 注入固定提示头「[语义召回 · 起步线索]」、带「可能无关」与「必须 read 原文」免责，是 associated function（无 &self）
   - src/vector/recall.rs:141-159（审计 #2）
7. VectorSearchTool 持有 Arc&lt;RecallEngine&gt;，default_k=8，name="vector_search"，execute 校验 query 必填（USER_INPUT）并 clamp(1, 10)，成功后调 format_recall_block 渲染，空结果返回换 grep/find_files 提示
   - src/tools/vector_search.rs:10-22（审计 #4）
   - src/tools/vector_search.rs:48-63（审计 #4）
8. CLI 侧 setup_vector_layer 用 VectorStore 构造 Arc&lt;RecallEngine&gt;，注入 VectorSearchTool，并直接消费 recall(ctx.task, 10) + format_recall_block 当任务提示种子；task 导航图种子过滤 fallback/leftover kind 并按 symbol 去重，邻居来自 codegraph 的 callers/callees
   - src/cli.rs:540-665（审计 #9）
   - src/vector/chunk.rs:84-134（审计 #14）
9. setup_vector_layer 失败降级：build_vector_index 失败 warn+audit 留痕「degraded」后复用已有索引继续；RecallEngine::open 失败 / recall 失败同样降级；INDEX_LOCKED 判负 return Err(e)；最终 first_suffix 由 Harness::with_first_user_suffix 应用
   - src/cli.rs:565-665（审计 #9）
   - src/cli.rs:305-334（审计 #9）
   - src/cli.rs:367-370（审计 #9）
10. --vector 旗标启用整个向量层装配；run_task_inner 在该分支后构造 Harness，writeguard 考前快照+考后 diff 自证零写入
   - src/cli.rs:48-49（审计 #9）
   - src/cli.rs:252-256（审计 #9）
   - src/cli.rs:375-396（审计 #9）
11. VectorConfig 字段：embed_model=Qwen/Qwen3-Embedding-8B、embed_dims=1024、embed_mode="composite"、repomap_budget=24_000、base_url/api_key=None（跟随 [llm]）；resolve_embed_endpoint 把 [vector] 缺省回退到 [llm] 端点（P005 R7.1 去重）
   - src/config.rs:92-148（审计 #41）
   - src/cli.rs:524-562（审计 #9）
12. vectors.db 表 chunks(file,line_start,line_end,symbol,kind,language,text_hash,embedding BLOB,dim) + 唯一索引 (file,symbol,line_start) + meta(key,value) + descriptions(key,text_hash,description)；commit_build 单事务内 GC+upsert+meta 翻新
   - src/vector/store.rs:57-78（审计 #19）
   - src/vector/store.rs:201-232（审计 #19）
13. 索引文件路径 = state_dir/indexes/&lt;fingerprint&gt;/vectors.db；fingerprint = sha256(绝对路径 + git HEAD) 前 16 hex；项目级目录 .codesleuth（与 .codegraph 同 writeguard 豁免）
   - src/vector/store.rs:19-27（审计 #19）
   - src/vector/store.rs:303-312（审计 #19）
14. EmbedClient 默认 1024 维、batch_size=64、EMBED_CONCURRENCY=4（tokio Semaphore）、timeout 120s；内容去重 hash 槽位回填；HTTP 5xx/429 标 retryable；瞬时失败 2s 重试一次；index 错位即拒绝（防静默投毒）
   - src/vector/embed.rs:52-64（审计 #16）
   - src/vector/embed.rs:72-199（审计 #16）
   - src/vector/embed.rs:11-20（审计 #16）
15. bootlock 引导锁被 run_index_vector 与 run_task_inner 内的 setup_vector_layer 共享（"向量索引构建" 标，DEFAULT_TIMEOUT=300s），D014 锁不跨 LLM 调用、不罩检索，构建段结束即 drop(vguard)
   - src/cli.rs:162-166（审计 #9）
   - src/cli.rs:565-590（审计 #9）
16. 测试三段式覆盖：brute_path_finds_exact_match、warm_start_brute_respects_alive（断言千级向量 hnsw.is_none）、compact_renumbers_and_rebuilds（1000 墓碑后 compact 重编号为 200）、recall_block_format_has_disclaimer、meta_mismatch_skips_hnsw_but_brute_still_works
   - src/vector/recall.rs:209-289（审计 #2）
17. callers 工具显示 RecallEngine 仅被 vector_search.rs:16 new、recall.rs:195 engine_for、vector/mod.rs:1 import 引用；setup_vector_layer 仅被 run_task_inner (cli.rs:194) 调用
   - src/tools/vector_search.rs:10-22（审计 #4）
   - src/vector/recall.rs:195-199（审计 #2）
   - src/cli.rs:194-334（审计 #9）

## 死胡同
- grep plain 模式搜 compose_input/EmbedMode/recall_by_vector 0 命中，已规避（仅在 doc/import 间接定位）
- 未读取 src/vector/build.rs 全文（依赖 docs/contexts/向量检索/03-vector-store.md + cli.rs 调度点 + 已有 store/recall/embed/chunk 边界证据交叉印证）
- 未读取 src/vector/repomap.rs 实现细节（仅在 setup_vector_layer src/cli.rs:625-653 消费层，build_task_map / wrap_repo_section 为推断+最简验证途径=read 该文件）
- 未读取 src/harness.rs ToolRegistry 装载上下文（Harness::new 接收 registry 已通过 src/cli.rs:359-371 证实）
- 未读取 src/vector/mod.rs 公共导出（依赖 grep 命中与 cli.rs import 推断）
- callers 查 RecallEngine::tombstone 仅返回 2 个测试调用方（recall.rs:219、245），未发现生产调用方，故 compact 路径在当前仓库内为预留入口

## 置信度
high

## 统计
turns=13 · tool_calls=23 · duration=133931ms · tokens=430678


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量召回引擎（src/vector/recall.rs 中的 RecallEngine）为上层工具与 CLI 任务编排提供「自然语言/向量 → top-K 余弦相似度代码位置」能力；采用「≤10 万向量走暴力、否则 HNSW 三段式（k×3 超采 → 余弦精确复算 → 墓碑过滤取 top-K）」策略，由 VectorSearchTool 与 cli::setup_vector_layer 共同消费。

## 证据列表
1. RecallEngine 结构定义在 src/vector/recall.rs:25-30，持 rows（真相快照）、alive（活跃集合）与可选 HNSW 影子图。
   - src/vector/recall.rs:25-30（审计 #2）
2. 三段式策略与常量定义明确：HNSW_EF_SEARCH=64、OVERFETCH_FACTOR=3、MIN_HNSW_SIZE=100_000、TOMBSTONE_REBUILD_RATIO=0.2。
   - src/vector/recall.rs:13-22（审计 #2）
3. RecallEngine::open 是构造入口，模型/维度不符时跳过 HNSW、降级为暴力；满足 MIN_HNSW_SIZE 才调用 build_hnsw。
   - src/vector/recall.rs:34-66（审计 #2）
4. recall_by_vector 实现 HNSW 三段式（k×OVERFETCH_FACTOR 超采 → cosine 精确复算 → alive 墓碑过滤 → 截断 top-K）与无 HNSW 时的暴力回退两套分支。
   - src/vector/recall.rs:100-125（审计 #2）
5. recall 异步入口负责 query 嵌入（instruct_query 指令前缀）并把 row 索引映射回 Chunk。
   - src/vector/recall.rs:128-139（审计 #2）
6. 墓碑与压缩：tombstone 维护 alive 集合；compact 在 needs_compact 为真时丢死行、重编号、必要时重建 HNSW。
   - src/vector/recall.rs:69-97（审计 #2）
7. format_recall_block 注入固定提示头「[语义召回 · 起步线索]」、带「可能无关」与「必须 read 原文」免责。
   - src/vector/recall.rs:141-159（审计 #2）
8. VectorSearchTool 持有 Arc<RecallEngine>，在 execute 中调用 engine.recall 并用 RecallEngine::format_recall_block 渲染。
   - src/tools/vector_search.rs:10-22（审计 #4）
   - src/tools/vector_search.rs:48-63（审计 #4）
9. CLI 侧 setup_vector_layer 用 VectorStore 构造 Arc<RecallEngine>，注入 VectorSearchTool，并直接消费 recall + format_recall_block 当任务提示种子。
   - src/cli.rs:602-621（审计 #17）

## 死胡同
- grep 用 plain 模式对 RecallEngine::open 检索返回 0 命中，改用 regex 模式后正常返回（已规避）。
- callers 查询 RecallEngine::tombstone 仅返回 2 个测试调用方（recall.rs:219、245），未发现生产调用方；据此判断 compact 路径在当前仓库内为预留入口。
- 未读取 src/vector/chunk.rs / src/vector/embed.rs / src/vector/store.rs 内部实现细节，仅依赖 recall.rs 中的引用与 import 关系，外部模块的细节作为隐含依赖未深入。

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=37969ms · tokens=54582
