# vector_search 语义召回（向量语义检索）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`02-vector-search-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# vector_search 语义召回（向量语义检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
vector_search 是仓库里的语义向量检索工具：接受自然语言问题（无需包含代码符号名），返回语义最相似的代码位置指针，适合概念型查询与跨词汇表述（src/tools/vector_search.rs:30-34）。入口为 VectorSearchTool（实现通用 Tool trait，注册名 "vector_search"），核心逻辑在 RecallEngine；由 CLI 装配层 setup_vector_layer 在向量层初始化时注册（src/cli.rs:597）。运作链路：execute → RecallEngine::recall（查询经指令前缀嵌入成向量）→ recall_by_vector 三段式召回（小库精确暴力 / 大库 HNSW 超采 + 余弦精算 + 墓碑过滤取 top-K）。上游依赖嵌入服务 EmbedClient、向量库 VectorStore 与 hnsw_rs 图索引；下游供 Agent 工具注册表（ToolRegistry）消费，返回结果提示使用者必须 read 原文验证。

## 证据列表
1. 工具描述与参数：输入自然语言 query，可选 k（默认 8 上限 10），返回语义最相似代码位置指针，证据需 read 原文
   - src/tools/vector_search.rs:30-46（审计 #2）
2. 入口结构 VectorSearchTool 实现通用 Tool trait，名字为 vector_search；execute 校验参数后调用 engine.recall 并格式化输出，空结果时引导改用 grep/find_files
   - src/tools/vector_search.rs:10-21（审计 #2）
   - src/tools/vector_search.rs:48-63（审计 #2）
3. 召回引擎 RecallEngine：SQLite 真相 + 会话内 HNSW 影子图 + 三段式查询；recall 将查询经 instruct_query 前缀嵌入，recall_by_vector 在无 HNSW（≤10 万块）时全量暴力余弦，有 HNSW 时 k×3 超采后精算并墓碑过滤
   - src/vector/recall.rs:1-4（审计 #4）
   - src/vector/recall.rs:127-139（审计 #4）
   - src/vector/recall.rs:99-125（审计 #4）
4. 装配入口：CLI 的 setup_vector_layer 补建向量索引、打开 VectorStore，并把 VectorSearchTool 注册进 ToolRegistry；构建失败弹性降级不致命
   - src/cli.rs:532-540（审计 #10）
   - src/cli.rs:589-597（审计 #10）
5. 上下游：依赖 vector::embed::EmbedClient、vector::store::{VectorStore, cosine}、hnsw_rs；作为 Box<dyn Tool> 注册进 ToolRegistry 供 Agent 调用
   - src/vector/recall.rs:6-10（审计 #4）
   - src/vector/recall.rs:26-29（审计 #4）
   - src/tools/vector_search.rs:109-114（审计 #2）

## 死胡同
- grep "VectorSearchTool::new|setup_vector_layer" 组合词零命中（grep 对多词 plain 模式不支持），拆成单符号后命中

## 置信度
high

## 统计
turns=5 · tool_calls=5 · duration=18381ms · tokens=31170


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
vector_search 是 --vector 旗标下装配的语义检索工具：LLM 经 Harness 主循环以 {"query":string,"k":int} 调用 VectorSearchTool::execute → RecallEngine::recall（查询加 Qwen3 Instruct 前缀嵌入为 1024 维向量）→ recall_by_vector 三段式召回（≤10 万块全量暴力余弦；>10 万块且 model/维度匹配才建 HNSW：k×3 超采→精确重算余弦→alive 墓碑过滤取 top-K）→ format_recall_block 格式化为「起步线索」文本块回给 LLM，并强制提示必须 read 原文验证。同一 RecallEngine 实例还在会话启动时用 recall(task,10) 生成首条消息后缀并给任务导航图供种子。配置全部走 [vector] TOML 段（无环境变量层），嵌入端点缺省跟随 [llm]。全链路弹性降级：构建失败/存储打不开/召回失败均 warn+审计 degraded 后继续，唯一致命分支是 bootlock 竞争败者 INDEX_LOCKED 判负退出。

## 证据列表
1. 总开关与调用链起点：仅当 CLI 旗标 --vector 为真，run_task_inner 才调 setup_vector_layer（src/cli.rs:190 附近定义的 run_task_inner 是唯一调用者，结构图证实）；装配 Err 时 INDEX_LOCKED 判负直接返回 Err（进程 exit 5），其他 Err 弹性降级 warn+审计 degraded 后任务继续
   - src/cli.rs:299-328（审计 #7）
2. setup_vector_layer 装配流程：embed_mode 字符串→EmbedMode（非 "raw" 一律 Composite）→ resolve_embed_endpoint（[vector].base_url/api_key 缺省跟随 [llm]，cli.rs:520-530）→ EmbedClient::new → bootlock::acquire_guard 引导锁 → build_vector_index 补建（失败不致命，warn+审计降级）→ drop(vguard) 锁只罩构建段 → VectorStore::open（Err 则 Ok(None) 整层跳过）→ RecallEngine::open → registry.register(VectorSearchTool)（src/cli.rs:597-599）
   - src/cli.rs:532-599（审计 #7）
3. 同一引擎的第二消费路径：装配完立即 recall(task, 10)（cli.rs:601），命中经 format_recall_block（cli.rs:615）与 --repo-map 导航图拼成首条消息后缀；Harness::with_first_user_suffix 注入首条 User 消息（harness.rs:69-72、83-88），失败降级为空后缀
   - src/cli.rs:600-616（审计 #7）
   - src/harness.rs:67-88（审计 #66）
4. Agent 运行时调用链：LLM 回 tool_calls → Harness 主循环依次做 submit_report 拦截 → recall(审计) 拦截 → 同参 canonical 去重（重复调用=无进展步计熔断）→ JSON 合法性 → tools.get(name) → audit record tool_call → tool.execute(args)（harness.rs:362）；输出作为 Tool 消息回填，并经 evidence.observe 喂证据库、info_keys 判信息增量（零增量计打转熔断）
   - src/harness.rs:331-362（审计 #66）
   - src/tools/mod.rs:25-59（审计 #66）
5. 工具参数契约：query 必填（缺则 USER_INPUT CS1001 + hint 示例），k 缺省 default_k=8（VectorSearchTool::new 硬编码，vector_search.rs:19），clamp(1,10)；execute 调 engine.recall(query,k).await，空结果返回引导文案「换关键词用 grep，或用 find_files」
   - src/tools/vector_search.rs:48-63（审计 #2）
   - src/vector/recall.rs:127-139（审计 #4）
6. 数据流·查询侧：query 先经 instruct_query 包上 Qwen3 指令前缀（QUERY_INSTRUCT = "Instruct: 给定代码库的自然语言问题，检索能回答它的代码片段\nQuery: "，embed.rs:12-13），再 POST {base_url}/embeddings（bearer 认证，body 含 model/dimensions/float，embed.rs:23-30、150-156），取回首条向量，空向量报 INDEX_NOT_AVAILABLE
   - src/vector/embed.rs:11-20（审计 #9）
   - src/vector/recall.rs:129-133（审计 #4）
7. EmbedClient 网络层：HTTP 超时 120s，batch_size=64，EMBED_CONCURRENCY=4 信号量并发，批失败 2s 后重试一次，429/5xx 标记 retryable；调用前按内容哈希去重（相同文本只嵌一次按槽位回填）
   - src/vector/embed.rs:15-16（审计 #9）
   - src/vector/embed.rs:76-114（审计 #9）
   - src/vector/embed.rs:166-177（审计 #9）
8. 嵌入防投毒校验：响应条数必须等于批大小，且 index 必须严格 0-based 对齐请求位次（validate_index_alignment，错位即 INDEX_EMBED_FAILED 拒绝，防 1-based 序号静默错位投毒）
   - src/vector/embed.rs:194-219（审计 #9）
9. RecallEngine::open：从 VectorStore.load_all 全量载入内存 rows（SQLite 真相，位于 <repo>/.codesleuth/indexes/<fingerprint>/vectors.db，store.rs:306-311），meta.model 与 embed.model 不一致或任一行维度不符则降级暴力并省略 HNSW；仅 rows.len() >= MIN_HNSW_SIZE 且双校验通过才 build_hnsw
   - src/vector/recall.rs:24-53（审计 #4）
   - src/vector/store.rs:303-312（审计 #50）
10. 硬编码调参常量（非配置项）：HNSW_M=16、MAX_LAYER=16、EF_CONSTRUCTION=200、EF_SEARCH=64、OVERFETCH_FACTOR=3、MIN_HNSW_SIZE=100_000（注释：≤10 万块直接精确暴力，免会话启动 HNSW 重建分钟级成本）、TOMBSTONE_REBUILD_RATIO=0.2
   - src/vector/recall.rs:13-22（审计 #4）
   - src/vector/recall.rs:55-66（审计 #4）
11. recall_by_vector 双路径：有 HNSW 时 search(k*3, ef=64) 超采 → 过滤 alive 墓碑 → 逐候选用 store::cosine 精确重算 → 降序截断 k；无 HNSW 时对 alive 全量暴力余弦排序取 k。NaN 相似度排序回退 Equal（partial_cmp unwrap_or）
   - src/vector/recall.rs:99-125（审计 #4）
12. 墓碑与压缩：tombstone(row) 从 alive 集摘除（供索引更新/删除标记旧块）；needs_compact 墓碑占比 >20% 时调用方应执行 compact（丢死行、重编号、hnsw=None 并按规模条件重建）。compact/tombstone 由索引更新侧触发，本检索链路只读
   - src/vector/recall.rs:68-97（审计 #4）
13. 输出格式契约：format_recall_block 生成「[语义召回 · 起步线索]」块，每条 = 序号. file:line_start-line_end symbol（相似度 X.XX），尾部固定附「可能无关/必须 read 原文」免责声明；fallback/leftover kind 也会照常输出（repomap 种子侧才过滤这两种 kind，cli.rs:624-626）
   - src/vector/recall.rs:141-159（审计 #4）
14. 与 repo-map 的交互契约：--repo-map 时召回命中作为种子（跳过 fallback/leftover kind），经 chunk::relations_for_symbol 扩一跳 caller/callee 邻居，vector::repomap::build_task_map 按 repomap_budget 生成任务导航图，wrap_repo_section 后拼进首条消息后缀
   - src/cli.rs:617-652（审计 #7）
15. 配置面（无环境变量，env 层已整体移除）：三层 TOML 合并（CLI>项目 .codesleuth/config.toml>全局 ~/.codesleuth/config.toml>默认）。[vector] 段：embed_model 默认 "Qwen/Qwen3-Embedding-8B"、embed_dims 默认 1024、embed_mode 默认 "composite"（仅 "raw" 字面量走 Raw）、repomap_budget 默认 24_000、base_url/api_key 默认 None 跟随 [llm]
   - src/config.rs:1-3（审计 #20）
   - src/config.rs:31-39（审计 #20）
   - src/config.rs:78-90（审计 #20）
   - src/config.rs:119-126（审计 #20）
16. 只读边界契约：Tool trait 注释明确 execute 为只读执行；工具链无任何写入能力。错误码段位：USER_INPUT=1001、INDEX_NOT_AVAILABLE=4010、INDEX_EMBED_FAILED=4015、INDEX_LOCKED=4016，段位决定进程退出码（4xxx→exit 5）
   - src/tools/mod.rs:1-22（审计 #66）
   - src/errors.rs:45-51（审计 #66）

## 死胡同
- grep plain 多词组合（如 "setup_vector_layer("、".execute("）多次零命中，改用 callers/callees/explore 结构图完成调用链追踪
- EmbedMode Raw/Composite 在 build.rs（索引构建侧）的消费细节未读，属相邻功能（索引构建），对本召回链路影响仅 meta.model 一致性校验（已读 recall.rs:36）

## 置信度
high

## 统计
turns=23 · tool_calls=30 · duration=173275ms · tokens=665719


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
vector_search 是仓库里的语义向量检索工具：接受自然语言问题（无需包含代码符号名），返回语义最相似的代码位置指针，适合概念型查询与跨词汇表述（src/tools/vector_search.rs:30-34）。入口为 VectorSearchTool（实现通用 Tool trait，注册名 "vector_search"），核心逻辑在 RecallEngine；由 CLI 装配层 setup_vector_layer 在向量层初始化时注册（src/cli.rs:597）。运作链路：execute → RecallEngine::recall（查询经指令前缀嵌入成向量）→ recall_by_vector 三段式召回（小库精确暴力 / 大库 HNSW 超采 + 余弦精算 + 墓碑过滤取 top-K）。上游依赖嵌入服务 EmbedClient、向量库 VectorStore 与 hnsw_rs 图索引；下游供 Agent 工具注册表（ToolRegistry）消费，返回结果提示使用者必须 read 原文验证。

## 证据列表
1. 工具描述与参数：输入自然语言 query，可选 k（默认 8 上限 10），返回语义最相似代码位置指针，证据需 read 原文
   - src/tools/vector_search.rs:30-46（审计 #2）
2. 入口结构 VectorSearchTool 实现通用 Tool trait，名字为 vector_search；execute 校验参数后调用 engine.recall 并格式化输出，空结果时引导改用 grep/find_files
   - src/tools/vector_search.rs:10-21（审计 #2）
   - src/tools/vector_search.rs:48-63（审计 #2）
3. 召回引擎 RecallEngine：SQLite 真相 + 会话内 HNSW 影子图 + 三段式查询；recall 将查询经 instruct_query 前缀嵌入，recall_by_vector 在无 HNSW（≤10 万块）时全量暴力余弦，有 HNSW 时 k×3 超采后精算并墓碑过滤
   - src/vector/recall.rs:1-4（审计 #4）
   - src/vector/recall.rs:127-139（审计 #4）
   - src/vector/recall.rs:99-125（审计 #4）
4. 装配入口：CLI 的 setup_vector_layer 补建向量索引、打开 VectorStore，并把 VectorSearchTool 注册进 ToolRegistry；构建失败弹性降级不致命
   - src/cli.rs:532-540（审计 #10）
   - src/cli.rs:589-597（审计 #10）
5. 上下游：依赖 vector::embed::EmbedClient、vector::store::{VectorStore, cosine}、hnsw_rs；作为 Box<dyn Tool> 注册进 ToolRegistry 供 Agent 调用
   - src/vector/recall.rs:6-10（审计 #4）
   - src/vector/recall.rs:26-29（审计 #4）
   - src/tools/vector_search.rs:109-114（审计 #2）

## 死胡同
- grep "VectorSearchTool::new|setup_vector_layer" 组合词零命中（grep 对多词 plain 模式不支持），拆成单符号后命中

## 置信度
high

## 统计
turns=5 · tool_calls=5 · duration=18381ms · tokens=31170
