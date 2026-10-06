# 向量存储（向量检索）

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
# 向量存储（向量检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量存储是向量检索域的唯一持久真相层：vectors.db（SQLite 单文件）持久化 chunk 及其嵌入向量，HNSW 图仅作为会话内影子索引。提供 schema 初始化、增量 upsert、text_hash 复用、GC 清理、原子构建事务、全量加载、cosine 工具与指纹路径等能力，由 build_vector_index 编排构建，由 RecallEngine 消费真相加载影子。

## 证据列表
1. vectors.db 是嵌入索引唯一持久真相，HNSW 仅会话内影子
   - src/vector/store.rs:1-3（审计 #2）
   - src/vector/recall.rs:1-4（审计 #13）
2. VectorStore 持 rusqlite Connection 并在 open 时建表（chunks/meta/descriptions）
   - src/vector/store.rs:41-43（审计 #2）
   - src/vector/store.rs:47-79（审计 #2）
3. commit_build 在单一事务内完成 GC + upsert + meta 写入，原子落库
   - src/vector/store.rs:198-232（审计 #2）
   - src/vector/build.rs:85-95（审计 #6）
4. build_vector_index 编排：plan_chunks → existing_hashes 增量复用 → compose_input + embed.embed → commit_build 落库
   - src/vector/build.rs:24-105（审计 #6）
5. RecallEngine.open 从 store.load_all 拉真相，仅当 ≥ MIN_HNSW_SIZE 时构建 HNSW 影子，否则走暴力
   - src/vector/recall.rs:34-66（审计 #13）
   - src/vector/recall.rs:19-20（审计 #13）
6. 索引路径 index_path(state_dir, fp)/vectors.db 与项目本地目录 .codesleuth/
   - src/vector/store.rs:303-312（审计 #2）
   - src/cli.rs:590-605（审计 #26）
7. VectorSearchTool 持有 Arc<RecallEngine>，execute 调 recall(query,k) 并用 format_recall_block 输出
   - src/tools/vector_search.rs:10-63（审计 #8）
8. 上游依赖 codegraph 符号表、EmbedClient、compose_input、bootlock；下游消费 VectorSearchTool 工具与 run_task 召回先行注入
   - src/vector/build.rs:7-14（审计 #6）
   - src/vector/build.rs:35-36（审计 #6）
   - src/cli.rs:162-172（审计 #26）
   - src/cli.rs:565-608（审计 #26）
   - src/cli.rs:602-605（审计 #26）
9. vector 模块公共导出 VectorStore / RecallEngine / build_vector_index 等
   - src/vector/mod.rs:1-21（审计 #4）

## 死胡同
- grep 全文搜 build_vector_index/VectorStore/RecallEngine/VectorSearchTool 未给出更多新调用点（仅返回 82 文件已读过的命中）
- 未深入 vector/chunk.rs、vector/embed.rs、vector/compose.rs 的实现细节（任务域限定为「向量存储」）

## 置信度
high

## 统计
turns=8 · tool_calls=14 · duration=29732ms · tokens=123453


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
## 功能定位

向量存储（vectors.db）是向量检索域的**唯一持久真相层**：以单个 SQLite 文件持久化所有 chunk 元数据与嵌入向量（BLOB），并用 text_hash 实现增量复用。HNSW 图是会话内影子图，不写盘。`VectorStore` 负责 schema、增删改查、原子提交与指纹路径；`RecallEngine` 在会话内从 store 加载真相并按规模选择 HNSW 或暴力；`build_vector_index` 编排切块→嵌入→落库的全流程；`VectorSearchTool` 暴露给 LLM agent 作为只读工具。同时为 `run_task` 提供「召回先行」注入与任务导航图种子。

## 证据列表
1. VectorStore.open 自动建父目录、打开/创建 SQLite 连接、置 WAL、并执行三张表（chunks/meta/descriptions）+ 唯一索引 idx_chunks_key 的建表批
   - src/vector/store.rs:47-79（审计 #2）
2. 仓库指纹 = sha256(绝对路径 + .git/HEAD) 前 16 hex；索引按此键隔离
   - src/vector/store.rs:19-27（审计 #2）
3. chunk 唯一键 = file+symbol+line_start，三元组用 \u{1} 分隔（line_start 解析失败时按 0 处理 + warn）
   - src/vector/store.rs:13-16（审计 #2）
   - src/vector/store.rs:113-140（审计 #2）
4. 向量在 SQLite 中以 BLOB（f32 小端四字节序列）持久化；blob_to_vec/vec_to_blob 互转在 upsert/load 路径使用
   - src/vector/store.rs:29-39（审计 #2）
   - src/vector/store.rs:175-196（审计 #2）
   - src/vector/store.rs:241-285（审计 #2）
5. commit_build 是单事务内的 GC + 全部 upsert + meta 写入（model/dim/mode），任一步失败整体回滚——消除「旧块已删、新块写一半」中间态；返回实际删除行数（幂等）
   - src/vector/store.rs:198-232（审计 #2）
6. remove_stale/upsert_chunk 都有 _on(&Connection) 内核，可在事务内复用——保证 GC 与 upsert 同一原子
   - src/vector/store.rs:108-140（审计 #2）
   - src/vector/store.rs:170-196（审计 #2）
7. existing_hashes 返回当前库所有 key→text_hash 映射，作为 build 阶段增量复用依据
   - src/vector/store.rs:142-167（审计 #2）
8. load_all 全量加载 chunks 并把 BLOB 反序列化为 Vec<f32>，构造 Chunk 时 text 字段置空（不携带原文）、header 由 breadcrumb 现算——HNSW 影子图原料
   - src/vector/store.rs:241-285（审计 #2）
9. cosine 工具函数在空向量时返回 0.0；用于 recall_by_vector 在 HNSW 候选上的精确复算
   - src/vector/store.rs:288-301（审计 #2）
10. 索引路径由两条函数协作：project_index_dir(repo_abs)=.codesleuth/；index_path(state_dir,fp)=state_dir/indexes/<fp>/vectors.db。run 路径下 state_dir 传 project_index_dir，所以最终落点为 <repo>/.codesleuth/indexes/<fp>/vectors.db——初稿把两条函数并列说有偏差（推断：初稿说「index_path(state_dir, fp)/vectors.db 与项目本地目录 .codesleuth/」，实际 run 路径上 index_path 的 state_dir 参数本身就是 project_index_dir，落到仓库本地而非全局状态目录）
   - src/vector/store.rs:303-312（审计 #2）
   - src/cli.rs:591-594（审计 #20）
   - src/cli.rs:169（审计 #20）
11. hash_of = sha256_hex，与 Chunk.text_hash 同口径；EmbedClient.embed 内部也用 hash_of 做内容级去重（用户立项 2026-10-05），相同文本只嵌一次按哈希回填
   - src/vector/store.rs:314-317（审计 #2）
   - src/vector/embed.rs:76-91（审计 #25）
12. build_vector_index 编排：dunce::canonicalize → fingerprint → symbols_from_codegraph(.codegraph/codegraph.db) → plan_chunks → VectorStore::open → existing_hashes → stale_index 判定（model/dim/mode 任一变更即旧向量全废）→ 按 current_keys 求 gc_keys → 同 text_hash 跳过（reused）→ compose_input 组装 → embed.embed 批并发去重 → commit_build 原子落库；嵌入等网络调用严格在事务外
   - src/vector/build.rs:25-104（审计 #6）
13. BuildReport 字段：chunks_total/embedded/reused/gc_removed/index_path，CLI 端 eprintln! 摘要
   - src/vector/build.rs:15-22（审计 #6）
   - src/cli.rs:173-180（审计 #20）
14. RecallEngine 持 embed/rows/alive/hnsw；open 时校验 meta.model == embed.model 且全行 dim == embed.dimensions（首行维度检查已修复为全行——P005 R6.1），且 rows.len() >= MIN_HNSW_SIZE(100_000) 才构建 HNSW；否则暴力
   - src/vector/recall.rs:24-53（审计 #4）
15. HNSW 参数 M=16、MAX_LAYER=16、EF_CONSTRUCTION=200、EF_SEARCH=64；超采倍数 OVERFETCH_FACTOR=3（k×3 候选再精确复算）
   - src/vector/recall.rs:13-18（审计 #4）
16. MIN_HNSW_SIZE = 100_000（用户硬要求 2026-10-05：暖启动零负担，≤10 万块直接精确暴力免 HNSW 重建分钟级成本）
   - src/vector/recall.rs:19-20（审计 #4）
17. tombstone 移出 alive 集合；tombstone_count 与 needs_compact（TOMBSTONE_REBUILD_RATIO=0.2）；compact 丢死行重编号并按规模决定重建 HNSW
   - src/vector/recall.rs:68-97（审计 #4）
18. recall_by_vector 两条路径：HNSW 走 search→filter alive→cosine 精确复算→按相似度降序截 k；暴力走 alive 全扫 cosine→降序截 k
   - src/vector/recall.rs:100-125（审计 #4）
19. recall(query,k) 用 EmbedClient.embed + instruct_query 加 Qwen3 指令感知前缀，返回 (Chunk, score) 序列；format_recall_block 输出含「可能无关」免责声明与 read 提示，相似度暴露为两位小数
   - src/vector/recall.rs:127-159（审计 #4）
20. VectorSearchTool 持 Arc<RecallEngine> + default_k=8，name=「vector_search」；k 上限 clamp 到 10；缺 query 抛 USER_INPUT；空结果给「换 grep/find_files」友好提示
   - src/tools/vector_search.rs:10-63（审计 #18）
21. EmbedClient 按 batch_size=64 分批、EMBED_CONCURRENCY=4 并发、JoinSet 还原顺序；embeddings 响应 1-based 或乱序会被 validate_index_alignment 拒绝防投毒；429/5xx 标 retryable；瞬时故障 2s 重试一次
   - src/vector/embed.rs:15-16（审计 #25）
   - src/vector/embed.rs:42-220（审计 #25）
22. compose_input 提供 Raw/Composite 两种；Composite = header + identifiers_of(text, cap=20) 提取 ASCII 标识符去重保序封顶 20；模式由 cfg.vector.embed_mode 字符串解析（"raw"→Raw，否则 Composite）
   - src/vector/compose.rs:8-51（审计 #27）
   - src/cli.rs:149-152（审计 #20）
   - src/cli.rs:546-549（审计 #20）
23. CLI 入口 --vector 旗标与 Command::Index --vector 都进同一 EmbedClient+EmbedMode 构造；resolve_embed_endpoint 把 [vector].base_url/api_key 缺省回退到 [llm] 端点（P005 R7.1 去重）
   - src/cli.rs:132-182（审计 #20）
   - src/cli.rs:526-536（审计 #20）
   - src/cli.rs:540-562（审计 #20）
24. build_vector_index 与 run 时 setup_vector_layer 都用 bootlock::acquire_guard(DEFAULT_TIMEOUT=300s, "向量索引构建") 串行化进程间构建；竞争败者按 D014 判负 INDEX_LOCKED 退出，不降级（run 路径上的 build 失败可降级，锁失败不降级）
   - src/cli.rs:162-172（审计 #20）
   - src/cli.rs:565-590（审计 #20）
   - src/bootlock.rs:24（审计 #39）
   - src/bootlock.rs:116-129（审计 #39）
25. run_task 路径：build_vector_index 失败走非致命降级（warn + audit 留痕「degraded」），仍继续 open store → RecallEngine → VectorSearchTool 注册 → 召回 task (k=10) → 若 hits 非空走 task 导航图（repomap，种子来自召回符号，邻居来自 codegraph 的 callers/callees）→ 拼成首条消息后缀注入 harness
   - src/cli.rs:570-665（审计 #20）
26. Index 子命令非 --vector 走 index_structure_error 诚实报错（CS4010 INDEX_NOT_AVAILABLE + hint：--rebuild 提示 run --fresh-index；否则提示加 --vector）
   - src/cli.rs:123-126（审计 #20）
   - src/cli.rs:465-478（审计 #20）
27. vector 模块公共导出 VectorStore / RecallEngine / build_vector_index / EmbedClient / EmbedMode / Chunk / SymbolRow / plan_chunks / fingerprint / index_path / instruct_query；build/chunk/compose/embed/recall/store/repomap 子模块
   - src/vector/mod.rs:1-21（审计 #8）
28. config VectorConfig 字段：embed_model=Qwen/Qwen3-Embedding-8B、embed_dims=1024、embed_mode="composite"、repomap_budget=24_000、base_url/api_key=None（跟随 [llm]）；FileVector 全部 Optional，merge 走 if let Some 覆写
   - src/config.rs:31-39（审计 #37）
   - src/config.rs:92-104（审计 #37）
   - src/config.rs:133-140（审计 #37）
   - src/config.rs:265-275（审计 #37）

## 死胡同
- grep 搜 vector/embed/embed_mode/embed_model/embed_dims/repomap_budget 部分以 fuzzy 收口，仅命中 docs 上下文与已读 src/config.rs 段，未引入新调用点
- 未深读 vector/repomap.rs（任务导航图、属向量层消费端，非存储核心）
- 未深读 vector/chunk.rs 的 plan_chunks 内部三层规则（属切块，不属存储）
- grep 搜 "INDEX_TIMEOUT" 0 命中——run_task_inner 路径上引导锁用 DEFAULT_TIMEOUT（300s），无独立 INDEX_TIMEOUT 常量（推断：搜过 0 命中即证）

## 置信度
high

## 统计
turns=17 · tool_calls=36 · duration=77267ms · tokens=614743


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量存储是向量检索域的唯一持久真相层：vectors.db（SQLite 单文件）持久化 chunk 及其嵌入向量，HNSW 图仅作为会话内影子索引。提供 schema 初始化、增量 upsert、text_hash 复用、GC 清理、原子构建事务、全量加载、cosine 工具与指纹路径等能力，由 build_vector_index 编排构建，由 RecallEngine 消费真相加载影子。

## 证据列表
1. vectors.db 是嵌入索引唯一持久真相，HNSW 仅会话内影子
   - src/vector/store.rs:1-3（审计 #2）
   - src/vector/recall.rs:1-4（审计 #13）
2. VectorStore 持 rusqlite Connection 并在 open 时建表（chunks/meta/descriptions）
   - src/vector/store.rs:41-43（审计 #2）
   - src/vector/store.rs:47-79（审计 #2）
3. commit_build 在单一事务内完成 GC + upsert + meta 写入，原子落库
   - src/vector/store.rs:198-232（审计 #2）
   - src/vector/build.rs:85-95（审计 #6）
4. build_vector_index 编排：plan_chunks → existing_hashes 增量复用 → compose_input + embed.embed → commit_build 落库
   - src/vector/build.rs:24-105（审计 #6）
5. RecallEngine.open 从 store.load_all 拉真相，仅当 ≥ MIN_HNSW_SIZE 时构建 HNSW 影子，否则走暴力
   - src/vector/recall.rs:34-66（审计 #13）
   - src/vector/recall.rs:19-20（审计 #13）
6. 索引路径 index_path(state_dir, fp)/vectors.db 与项目本地目录 .codesleuth/
   - src/vector/store.rs:303-312（审计 #2）
   - src/cli.rs:590-605（审计 #26）
7. VectorSearchTool 持有 Arc<RecallEngine>，execute 调 recall(query,k) 并用 format_recall_block 输出
   - src/tools/vector_search.rs:10-63（审计 #8）
8. 上游依赖 codegraph 符号表、EmbedClient、compose_input、bootlock；下游消费 VectorSearchTool 工具与 run_task 召回先行注入
   - src/vector/build.rs:7-14（审计 #6）
   - src/vector/build.rs:35-36（审计 #6）
   - src/cli.rs:162-172（审计 #26）
   - src/cli.rs:565-608（审计 #26）
   - src/cli.rs:602-605（审计 #26）
9. vector 模块公共导出 VectorStore / RecallEngine / build_vector_index 等
   - src/vector/mod.rs:1-21（审计 #4）

## 死胡同
- grep 全文搜 build_vector_index/VectorStore/RecallEngine/VectorSearchTool 未给出更多新调用点（仅返回 82 文件已读过的命中）
- 未深入 vector/chunk.rs、vector/embed.rs、vector/compose.rs 的实现细节（任务域限定为「向量存储」）

## 置信度
high

## 统计
turns=8 · tool_calls=14 · duration=29732ms · tokens=123453
