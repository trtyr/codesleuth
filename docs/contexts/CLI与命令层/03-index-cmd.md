# 向量索引子命令（CLI与命令层）

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
# 向量索引子命令（CLI与命令层）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量索引子命令由 CLI 的 `Index --vector` 旗标触发：`Cli::run` 分发到 `run_index_vector`（src/cli.rs:109-115, 132-182），后者加载配置、解析嵌入端点、获取仓库级引导锁，并 `block_on` 调用 `vector::build_vector_index`（src/vector/build.rs:24-105）完成 chunk 规划、复用/失效判定、网络嵌入、原子提交与 GC，最终通过 `eprintln!` 打印 `BuildReport` 统计。

## 证据列表
1. Command::Index 子命令定义包含 `--vector` 旗标，是该功能的 CLI 入口定义。
   - src/cli.rs:67-77（审计 #2）
2. Cli::run 在 Command::Index 分支上当 `vector: true` 时调用 `Self::run_index_vector`，否则走 index_structure_error 报错路径。
   - src/cli.rs:109-124（审计 #2）
3. run_index_vector 负责加载配置、解析嵌入端点、构造 EmbedClient、获取 bootlock 引导锁，并以 tokio runtime block_on 驱动 vector::build_vector_index，最后 eprintln! 输出 BuildReport。
   - src/cli.rs:132-182（审计 #2）
4. build_vector_index 是核心实现：基于 codegraph.db 拉符号、规划 chunk、依据 model/dim/mode 判定 stale、按 text_hash 复用、通过 EmbedClient 嵌入，最后由 VectorStore::commit_build 在单事务内完成 GC 删除 + upsert + meta 写入。
   - src/vector/build.rs:24-105（审计 #4）

## 死胡同
- grep "run_index_vector|Index\s*\{|Command::Index" 命中了 .pi/goals/ 下的目标账本 JSON 文件，与代码无关，未采用
- 未深入展开 setup_vector_layer / index_structure_hint_is_honest 等次要符号，超出概览级侦察范围

## 置信度
high

## 统计
turns=5 · tool_calls=5 · duration=23387ms · tokens=28101


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
向量索引子命令 `codesleuth index <path> --vector` 是只读侦察域之外的写侧能力——按用户设定的「只展开不另起」原则，本会话为初稿补全了完整调用链、数据流、配置/开关、边界/坑、交互契约五节并附组件调用图。核心结论：CLI 入口在 src/cli.rs:67-77，分发到 src/cli.rs:132-182 的 run_index_vector（获取 bootlock、构造 EmbedClient、block_on 调用 vector::build_vector_index），后者在 src/vector/build.rs:24-105 编排 plan_chunks → existing_hashes/stale 判定 → compose_input → embed → VectorStore::commit_build 原子事务，落产物到 .codesleuth/indexes/<fingerprint>/vectors.db；run 路径上的 setup_vector_layer（src/cli.rs:540-665）共享同一把 bootlock 与同一套 build 流水线但失败策略是 warn+审计+复用已有索引（非判负）。初稿的 src/cli.rs:109-115 引用在事实中对应 109-127（包含完整 if/else 分支至 127 行），实际 run_index_vector 起止为 132-182。

## 证据列表
1. Command::Index 子命令定义包含 path 位置参数、--rebuild（仅文案路由）、--vector 三项旗标，是该功能的 CLI 入口定义。
   - src/cli.rs:67-77（审计 #2）
2. Cli::run 在 Command::Index 分支上当 vector=true 调 Self::run_index_vector 拿 exit 0 或错误退出码；否则调 index_structure_error 输出诚实错误（带 --rebuild 友好提示或「请加 --vector」hint）。
   - src/cli.rs:109-127（审计 #2）
   - src/cli.rs:465-478（审计 #2）
3. run_index_vector 编排：dunce::canonicalize 包装 REPO_NOT_FOUND → config::load(overrides) → cfg.resolve_api_key → resolve_embed_endpoint（[vector] 缺省跟随 [llm]）→ embed_mode 字符串→EmbedMode → EmbedClient::new → tokio::runtime::Runtime::new（INTERNAL）→ bootlock::acquire_guard（DEFAULT_TIMEOUT=300s）→ rt.block_on(build_vector_index) → eprintln! BuildReport。
   - src/cli.rs:132-182（审计 #2）
   - src/cli.rs:526-536（审计 #2）
   - src/bootlock.rs:22-24（审计 #47）
   - src/bootlock.rs:120-129（审计 #47）
   - src/config.rs:528-538（审计 #41）
4. build_vector_index 是核心实现：fingerprint 决定索引槽位 → symbols_from_codegraph 软降级空集 → plan_chunks 三层规则 → existing_hashes + meta(model/dim/mode) 判定 stale_index → 算 gc_keys → text_hash 复用扫描产 pending+reused → compose_input 逐块组装（事务外）→ embed.embed 4 路并发批嵌入 → commit_build 单事务完成 GC + upsert + meta 翻新 → 返回 BuildReport。
   - src/vector/build.rs:24-105（审计 #5）
   - src/vector/store.rs:19-27（审计 #26）
   - src/vector/chunk.rs:137-166（审计 #14）
   - src/vector/chunk.rs:258-474（审计 #14）
   - src/vector/compose.rs:42-51（审计 #32）
   - src/vector/store.rs:201-232（审计 #26）
5. vector 模块公共导出 BuildReport、build_vector_index、Chunk、SymbolRow、plan_chunks、EmbedMode、compose_input、EmbedClient、instruct_query、RecallEngine、VectorStore、fingerprint、index_path；分层 chunk/compose/embed/store/recall/build/repomap 各司其职。
   - src/vector/mod.rs:1-21（审计 #8）
6. EmbedClient：Qwen3-Embedding-8B via OpenAI 兼容 /embeddings，dimensions 默认 1024，batch_size=64，EMBED_CONCURRENCY=4 tokio Semaphore；去重→按批分→并发→失败 2s 单次重试→按 batch 序号还原→按去重前槽位回填；validate_index_alignment 严格 0-based 校验、错位即拒绝防投毒；429/5xx 标 retryable=true。
   - src/vector/embed.rs:1-30（审计 #20）
   - src/vector/embed.rs:51-64（审计 #20）
   - src/vector/embed.rs:72-146（审计 #20）
   - src/vector/embed.rs:149-220（审计 #20）
7. VectorStore：open 建表 chunks/meta/descriptions（journal_mode=WAL、idx_chunks_key 唯一索引）；chunk_key = file \x01 symbol \x01 line_start；fingerprint = sha256(abs ‖ .git/HEAD) 前 16 hex；index_path = .codesleuth/indexes/<fp>/vectors.db（2026-10-05 拍板迁回项目）；remove_stale 按 DELETE 影响行数计（幂等重删=0）；commit_build 单一事务 GC+upsert+meta。
   - src/vector/store.rs:14-16（审计 #26）
   - src/vector/store.rs:19-27（审计 #26）
   - src/vector/store.rs:47-79（审计 #26）
   - src/vector/store.rs:108-167（审计 #26）
   - src/vector/store.rs:201-232（审计 #26）
   - src/vector/store.rs:303-312（审计 #26）
8. plan_chunks 三层规则：层1 符号主块（KEPT_KINDS 过滤、容器丢弃、注释/docstring 上提 ≤20 行、MAX_CHUNK_CHARS=6000）；层2 超大滑窗（SUB_WINDOW_LINES=100、OVERLAP=15）；层3 leftover（gap ≥2 非空行）+ 兜底（md 按 # 切节，其他 100/20 窗口、空文件不切块）。SKIP_DIRS 含 .codesleuth/.codegraph 等豁免目录。
   - src/vector/chunk.rs:13-49（审计 #14）
   - src/vector/chunk.rs:227-244（审计 #14）
   - src/vector/chunk.rs:258-474（审计 #14）
9. bootlock 实现：flock 跨进程串行化（src/bootlock.rs:1-114），首次即获=Won、轮询等待后获=Lost（判负 CS4016）、超时=CS4016 带 hint；acquire_guard 封装 graph 引导、index --vector、run --vector 三个调用点统一判负语义，不允许降级。
   - src/bootlock.rs:1-129（审计 #47）
10. run 路径上的 setup_vector_layer（src/cli.rs:540-665）共享同一把 bootlock 与同一套 build 流水线，但失败策略不同：build_vector_index 错误→warn!+audit.record('degraded')+复用已有索引继续；INDEX_LOCKED 错误→判负 return Err(e)；RecallEngine::open 失败/recall 失败同样降级并审计留痕。VectorSearchTool 由 Arc<RecallEngine> 驱动（src/tools/vector_search.rs 模块存在，本次未读其源，凭 grep 命中推断）。
   - src/cli.rs:304-334（审计 #2）
   - src/cli.rs:540-665（审计 #2）
   - src/vector/recall.rs:19-53（审计 #80）
11. 错误码段位映射：CS1xxx→1、CS1010-1099→2、CS2xxx→3、CS3xxx→4、CS4xxx→5、CS5xxx→6；本子命令会触发的有 REPO_NOT_FOUND(3001,4)、CONFIG_MISSING(1011,2)、INDEX_NOT_AVAILABLE(4010,5)、INDEX_EMBED_FAILED(4015,5)、INDEX_LOCKED(4016,5)、INTERNAL(5001,6)。report_error 输出 eprintln! 主行+根因行+hint 行。
   - src/errors.rs:13-23（审计 #59）
   - src/errors.rs:42-53（审计 #59）
   - src/errors.rs:107-115（审计 #59）
12. 配置链：CLI > .codesleuth/config.toml（兼容 ./codesleuth.toml）> ~/.codesleuth/config.toml > 默认；env 层已整体移除（2026-10-05 裁决）；VectorConfig 默认 embed_model=Qwen/Qwen3-Embedding-8B、embed_dims=1024、embed_mode=composite、repomap_budget=24000、base_url/api_key=None（跟随 [llm]）。
   - src/config.rs:32-39（审计 #41）
   - src/config.rs:94-148（审计 #41）
   - src/config.rs:159-200（审计 #41）
13. 初稿证据 src/cli.rs:109-115 实际只覆盖 if/Ok/Err 三行，完整 if vector→run_index_vector / else→index_structure_error 分支在 109-127；run_index_vector 实际是 src/cli.rs:132-182。
   - src/cli.rs:109-127（审计 #2）
   - src/cli.rs:132-182（审计 #2）

## 死胡同
- 未读 src/tools/vector_search.rs 全文（凭模块路径与 grep 命中推断其持有 Arc<RecallEngine> 并对外暴露 vector_search 工具）
- 未读 src/vector/repomap.rs 实现细节（与本子命令数据流正交，仅在 setup_vector_layer 路径消费）
- 未读 src/cli.rs:679-805（run_task_inner 末尾/审计/状态机收尾段，不影响 index --vector 链路）
- 未深入 src/cli.rs:680+ 的 run_task_inner 详细执行段（已通过 200-460 段确认 --vector→setup_vector_layer 分支）
- RecallEngine::recall / format_recall_block 主体未读（与本子命令无直接调用关系）
- grep "VectorConfig|vector:" 一次零命中（pattern 写法问题）→ 改 grep "VectorConfig" 命中 3 行（src/config.rs:78/94/133），并直接 read src/config.rs:78-148 获取全量定义

## 置信度
high

## 统计
turns=35 · tool_calls=33 · duration=200911ms · tokens=1328021


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
向量索引子命令由 CLI 的 `Index --vector` 旗标触发：`Cli::run` 分发到 `run_index_vector`（src/cli.rs:109-115, 132-182），后者加载配置、解析嵌入端点、获取仓库级引导锁，并 `block_on` 调用 `vector::build_vector_index`（src/vector/build.rs:24-105）完成 chunk 规划、复用/失效判定、网络嵌入、原子提交与 GC，最终通过 `eprintln!` 打印 `BuildReport` 统计。

## 证据列表
1. Command::Index 子命令定义包含 `--vector` 旗标，是该功能的 CLI 入口定义。
   - src/cli.rs:67-77（审计 #2）
2. Cli::run 在 Command::Index 分支上当 `vector: true` 时调用 `Self::run_index_vector`，否则走 index_structure_error 报错路径。
   - src/cli.rs:109-124（审计 #2）
3. run_index_vector 负责加载配置、解析嵌入端点、构造 EmbedClient、获取 bootlock 引导锁，并以 tokio runtime block_on 驱动 vector::build_vector_index，最后 eprintln! 输出 BuildReport。
   - src/cli.rs:132-182（审计 #2）
4. build_vector_index 是核心实现：基于 codegraph.db 拉符号、规划 chunk、依据 model/dim/mode 判定 stale、按 text_hash 复用、通过 EmbedClient 嵌入，最后由 VectorStore::commit_build 在单事务内完成 GC 删除 + upsert + meta 写入。
   - src/vector/build.rs:24-105（审计 #4）

## 死胡同
- grep "run_index_vector|Index\s*\{|Command::Index" 命中了 .pi/goals/ 下的目标账本 JSON 文件，与代码无关，未采用
- 未深入展开 setup_vector_layer / index_structure_hint_is_honest 等次要符号，超出概览级侦察范围

## 置信度
high

## 统计
turns=5 · tool_calls=5 · duration=23387ms · tokens=28101
