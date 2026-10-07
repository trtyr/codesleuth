# codesleuth · 总览

> docs-init 第三步总览升级 · 2026-10-07

> 全局架构图：`00-architecture.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。这个仓库的 docs/contexts/ 下已经有一套文档：00-overview.md 是旧的粗框架，各功能域子目录下是逐功能详稿与配图。你的任务：重写一份新的总览文档（它将替换 00-overview.md，成为整套文档的门面）。

【执行纪律——逐条遵守，违反任何一条即为失败任务】
1. 详稿是深挖产物、已带 path:line 证据——以它们为准，引用即可，不要重新深挖代码；
   只有存疑处才回源码核实。
2. 严格按以下五节输出，节标题逐字照写，顺序不许变：
   「## 项目定位」一段话：是什么 / 给谁用 / 核心价值（在旧总览基础上按详稿修正）。
   「## 全局架构图」按文末【画图技能规格】画系统级架构图——跨功能的组件与数据流，
   组件名用真实模块/服务名，关系带标签。完整单文件 HTML 放在单独围栏块：

   「## 功能域导航」按 docs/contexts/ 目录结构逐篇列出：相对路径 + 一句话，
   让读者能一跳直达；配图文件名也列出。
   「## 全局交互」一段话：功能域之间怎么协作（数据 / 事件 / 调用）。
   「## 未解问题」汇总各详稿标注（推断）/ 相互冲突的点；
   冲突必须列出双方说法，不要自行裁决。
3. 五节全部输出完后立即停止。
4. 凭据只写变量名；禁止 TODO / 占位符 / 空节。
5. 不要复述本任务书。

【docs/ 文档清单】
{listing}

【画图技能规格】
{diagram_skill}

【docs/ 文档清单】
- CLI与命令层/01-cli-dispatch.md
- CLI与命令层/02-config-cmd.md
- CLI与命令层/03-index-cmd.md
- LLM与配置/01-llm-provider.md
- LLM与配置/02-config-loading.md
- 上下文与报告/01-context-compaction.md
- 上下文与报告/02-report.md
- 侦察编排/01-harness.md
- 侦察编排/02-system-prompt.md
- 只读工具面/01-tool-registry.md
- 只读工具面/02-read-tool.md
- 只读工具面/03-fuzzy-search.md
- 只读工具面/04-vector-search-tool.md
- 向量检索/01-vector-embed.md
- 向量检索/02-vector-chunk.md
- 向量检索/03-vector-store.md
- 向量检索/04-vector-recall.md
- 向量检索/05-vector-compose.md
- 向量检索/06-repo-map.md
- 安全与防护/01-fence.md
- 安全与防护/02-bootlock.md
- 安全与防护/03-writeguard.md
- 安全与防护/04-audit.md
- 安全与防护/05-evidence.md
- 结构图与MCP/01-graph-tools.md
- 结构图与MCP/02-mcp-client.md
- 错误与日志/01-error-codes.md
- 错误与日志/02-logging.md

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
codesleuth 是一个 Rust 实现的只读代码侦察 Agent CLI：main.rs 入口 + Cli::run 三分支（Config / Index --vector / run_task → Harness）派发；Harness 主循环驱动 LlmProvider（OpenAI 兼容 + 重试退避）+ ToolRegistry（read / find_files / grep / vector_search / 5 个 graph 工具；接口根上无写能力）+ 结构图 CodegraphEngine→McpClient→外部 codegraph CLI + 向量检索六子层（embed/chunk/store/recall/compose/repomap，以 vectors.db 为唯一持久真相、HNSW 为会话内影子）；安全由 Fence（路径层 dunce canonicalize 解 symlink）+ BootLock（<repo>/.codesleuth/boot.lock 跨进程 flock 串行化 codegraph 引导与向量构建）+ WriteGuard（任务前后 per-file 指纹 diff 零写入自证）+ Audit JSONL 账本 + EvidenceStore（cite_seq 作为 report↔audit 锚点）五闸共同保证；上下文三段式压缩（build_handoff 零 LLM → compact 保留头 2 + handoff + 最近 KEEP_RECENT=6 → 审计留痕，被驱逐原文按 audit seq 范围 recall 续读），报告走 REPORT_SCHEMA_VERSION=1 版本化 schema + build_report 校验 evidence（cite_seq 必须查到）+ degraded 兜底；错误用 CS-段位码（CS1xxx→exit 1 / CS2xxx→3 / CS3xxx→4 / CS4xxx→5 / 其它→6），日志 stderr + ~/.codesleuth/logs/&lt;session&gt;.log 同过滤同格式双轨。会话级协作分三条业务总线（检索请求 / LLM 决策 / 向量与结构图数据）和一条审计总线。

## 证据列表
1. codesleuth 是只读代码侦察 Agent CLI：main.rs 入口、Cli::run 三分支派发（Config / Index --vector / run_task → Harness），Harness 主循环驱动 LlmProvider + ToolRegistry + CodegraphEngine + 向量检索六子层 + 五道安全闸共同保证只读契约。
   - docs/contexts/00-overview.md:5-7（审计 #2）
   - docs/contexts/CLI与命令层/01-cli-dispatch.md:22-44（审计 #7）
2. CLI 入口 main.rs 生成 session_id、init_tracing、调用 Cli::run；Cli::run 按 Command::{Config, Index, None} 三分支派发，Index --vector 走 run_index_vector，None 走 run_task → Harness。
   - docs/contexts/CLI与命令层/01-cli-dispatch.md:26-44（审计 #7）
   - docs/contexts/错误与日志/02-logging.md:26-35（审计 #67）
3. config 子命令 get / set / path 三动作走 config::load 合并（CLI > 项目 > 全局 > 默认值，环境变量层已整体移除）与 ~/.codesleuth/config.toml 持久化；未知名报 CS1012。
   - docs/contexts/CLI与命令层/02-config-cmd.md:26-46（审计 #9）
   - docs/contexts/LLM与配置/02-config-loading.md:26-51（审计 #15）
4. index --vector 触发 run_index_vector → vector::build_vector_index：plan_chunks → 增量 text_hash 复用 → EmbedClient 嵌入 → VectorStore::commit_build 在单事务内完成 GC + upsert + meta。
   - docs/contexts/CLI与命令层/03-index-cmd.md:26-34（审计 #11）
5. LlmProvider trait 只暴露 async fn chat；OpenAiProvider 走 OpenAI 兼容 + 自定义 base_url + 500ms 指数退避封顶 4 次 + 429/5xx 标记 retryable=true 重试；max_retries 写死 2；终局 retryable=false；SharedProvider = Arc&lt;dyn LlmProvider&gt;。
   - docs/contexts/LLM与配置/01-llm-provider.md:26-44（审计 #13）
6. Harness::run 是主循环：构造 system（SYSTEM_PROMPT 恒定）+ user 首消息 + 注入 [repo map] 后缀；MAX_NO_PROGRESS_STREAK=5 熔断；build_report 校验 evidence cite_seq；submit_report 收敛 RunOutcome，否则 degraded 兜底。
   - docs/contexts/侦察编排/01-harness.md:26-51（审计 #22）
   - docs/contexts/上下文与报告/02-report.md:26-55（审计 #20）
7. SYSTEM_PROMPT 定稿「只读身份 + 行为边界 + 检索纪律（层进协议）+ 无空转 + 输出契约」，D005 抬升为产品资产；Harness::run 启动时注入首条 system 消息，任务细节永不进 system。
   - docs/contexts/侦察编排/02-system-prompt.md:26-39（审计 #24）
8. 上下文压缩（D013 v2）：阈值 = 模型窗口 × 百分比（默认 1M × 60%，&gt;100 封顶）；三段式 = build_handoff（零 LLM，承上启下）→ compact（保留头 2 + handoff User + 最近 KEEP_RECENT=6，中间驱逐）→ 审计留痕（compaction_begin / compaction）；被驱逐原文经 recall 工具按 audit seq 范围回灌，不重不漏。
   - docs/contexts/上下文与报告/01-context-compaction.md:26-60（审计 #18）
9. Report 是版本化 schema（REPORT_SCHEMA_VERSION=1）：含 report_schema_version / task / answer / findings / dead_ends / confidence / degraded / stats；build_report 校验 evidence 必须本会话真实观察过（cite_seq 能查到），零 findings 须死胡同交代，confidence 必须在 high/medium/low 内；模型未提交 submit_report 时走 degraded_prose 兜底。
   - docs/contexts/上下文与报告/02-report.md:26-55（审计 #20）
10. ToolRegistry 是 Vec&lt;Box&lt;dyn Tool&gt;&gt; 薄包装；Tool trait 接口根上无写能力（D009 第一道闸）；CLI 启动期 register ReadTool / FileFinderTool / GrepTool，Harness 持有 tools 字段并按 name 取出 execute。
   - docs/contexts/只读工具面/01-tool-registry.md:26-58（审计 #27）
11. read 工具提供行号 + 12-bit 行哈希锚点 / offset+limit 分页（DEFAULT_LIMIT=200, MAX_LIMIT=2000）/ 二进制探测拒倾倒 / lossy UTF-8 消毒并声明 U+FFFD / 空文件与越界诚实反馈；execute 经 fence.resolve(&amp;path_arg) 守门，越界返 REPO_NOT_READABLE。
   - docs/contexts/只读工具面/02-read-tool.md:26-46（审计 #29）
12. find_files（frecency 模糊路径搜索）+ grep（plain / regex / fuzzy 三态）共用 FuzzyEngine 持有的 fff-search FilePicker 会话级索引；CLI 启动期 Arc&lt;FuzzyEngine&gt; 注册两个工具。
   - docs/contexts/只读工具面/03-fuzzy-search.md:26-48（审计 #31）
13. vector_search 工具：query 必填，k 缺省 8 并被 clamp 到 [1,10]；execute 调 RecallEngine::recall → format_recall_block 渲染位置指针；CLI 启动期 setup_vector_layer 中构造 RecallEngine 并把工具挂入 ToolRegistry，同一召回通道被直接消费以注入 system 提示。
   - docs/contexts/只读工具面/04-vector-search-tool.md:26-45（审计 #33）
14. EmbedClient：默认 1024 维 Matryoshka 降维（注释允许实验上调 4096），batch_size=64，EMBED_CONCURRENCY=4（Semaphore），2s 兜底重试一次；查询侧加 QUERY_INSTRUCT 前缀，文档侧不加；响应 index 必须严格 0-based 对齐（validate_index_alignment）防静默投毒；上游由 cfg.vector 构造，下游被 build_vector_index、RecallEngine::recall、VectorSearchTool 三处消费。
   - docs/contexts/向量检索/01-vector-embed.md:26-51（审计 #36）
15. 切块：以 codegraph 符号表为边界真源 + 三层规则（层 1 符号主块 / 层 2 超大 SUB_WINDOW_LINES=100, SUB_OVERLAP_LINES=15 滑窗 / 层 3 兜底 FALLBACK_WINDOW_LINES=100, FALLBACK_OVERLAP_LINES=20）；MAX_CHUNK_CHARS=6000；KEPT_KINDS=function / method / struct / class / interface / impl / trait。
   - docs/contexts/向量检索/02-vector-chunk.md:26-61（审计 #38）
16. VectorStore：vectors.db（SQLite 单文件）是嵌入索引唯一持久真相，HNSW 仅会话内影子；commit_build 在单事务内完成 GC + upsert + meta 写入；chunk_key = file + symbol + line_start 作唯一键 + 唯一索引。
   - docs/contexts/向量检索/03-vector-store.md:26-54（审计 #40）
17. RecallEngine：≤10w 走暴力，否则 HNSW 三段式（HNSW_EF_SEARCH=64、OVERFETCH_FACTOR=3 超采 → cosine 精确复算 → tombstone 墓碑过滤取 top-K）；MIN_HNSW_SIZE=100_000、TOMBSTONE_REBUILD_RATIO=0.2；model / dim 不符时跳过 HNSW 降级暴力；format_recall_block 注入固定提示头 [语义召回 · 起步线索] + 可能无关 + 必须 read 原文免责。
   - docs/contexts/向量检索/04-vector-recall.md:26-46（审计 #42）
18. compose_input(chunk, mode)：EmbedMode = Raw（返回 chunk.text 全文）/ Composite（白拿层 header + 标识符层去重封顶 20，默认模式）；模式由 cfg.vector.embed_mode 解析（raw→Raw，否则→Composite），mode 字符串写入索引 meta 参与增量复用与作废判定。
   - docs/contexts/向量检索/05-vector-compose.md:22-24（审计 #45）
19. repo map 预算注入：默认 24_000 字符（≈6k token），单符号行 200 字符上限；从 codegraph SQLite 读符号表与边表度数，按度数降序 → 名称升序 → 文件路径升序贪心装填；wrap_repo_section 把地图包成 [repo map] 段；build_task_map 优先召回命中种子 + 邻居再按度数填，build_repo_map 走全图度数降序；导航构建失败走 tracing::warn + audit degraded 事件 + 跳过注入，不阻断主流程。
   - docs/contexts/向量检索/06-repo-map.md:26-66（审计 #47）
20. Fence 路径围栏：词汇层归一化（折叠 . 与 ..，不解 symlink）+ starts_with_root 预检 + dunce::canonicalize 解 symlink 复检；越界返 CS3003 FENCE_DENIED，不存在路径返 CS3001 REPO_NOT_FOUND；Windows 下 starts_with_root 按 ASCII 大小写不敏感比较；CLI 启动期装配 Arc&lt;Fence&gt; 注入 ReadTool 与 FuzzyEngine。
   - docs/contexts/安全与防护/01-fence.md:26-49（审计 #49）
21. BootLock：仓库级引导互斥原语，&lt;repo&gt;/.codesleuth/boot.lock 上的 std::fs::File::try_lock（flock）；acquire 返回 Won / Lost / 超时三态，Lost / 超时 → CS4016 INDEX_LOCKED（带 hint 重跑本命令即可）；BootLockGuard 在 Drop 中 unlock 释放；三处消费：src/tools/graph.rs:78 codegraph 引导（init / index --force 段）、src/cli.rs:162 手动 index --vector、src/cli.rs:565 run 时向量构建兜底引导。
   - docs/contexts/安全与防护/02-bootlock.md:26-41（审计 #51）
22. WriteGuard：snapshot(root) + diff(before, after)；Manifest = BTreeMap&lt;relpath, sha256&gt;，Change = Added | Removed | Modified；walk 跳过 symlink 与 SKIP_DIRS（.git / .codegraph / .codesleuth / target / node_modules / dist / __pycache__ / .venv），&gt;1MB 文件用大小+首 64KB 指纹；CLI 任务起止两次 snapshot，diff 后写 audit kind=write_check 行，含 files_snapshotted / unattributed_changes / samples（前 5 条示例）。
   - docs/contexts/安全与防护/03-writeguard.md:26-41（审计 #54）
23. 审计日志：append_line 取现有 max seq + 1 写入 JSONL 保持单调不重不漏；new_session_id = 纳秒时间 + pid hex；Audit 单代轮转 64MB；main 启动期生成 session_id 同时注入日志与审计；Harness 通过 self.audit.record 留痕 compaction_begin / compaction / llm_error，context::build_handoff 引用 audit_from..audit_to 续读被驱逐原文。
   - docs/contexts/安全与防护/04-audit.md:26-46（审计 #56）
24. EvidenceStore：内部 path→seq 的 HashMap；observe_exact（read 专用，逐字入库，含空格 / 中文路径）+ observe（按空白与 []()&lt;&gt;"'`,; 切词经 path_like 过滤）；cite_seq 是 report↔audit 互查锚点；Harness 持有 evidence: Mutex&lt;EvidenceStore&gt; 字段。
   - docs/contexts/安全与防护/05-evidence.md:26-46（审计 #58）
25. 结构图工具集：CodegraphEngine::start 走 init / force → spawn codegraph serve --mcp → initialize 握手；CodegraphEngine::call 统一 MCP 调用自动追加 projectPath 并把工具名加 codegraph_ 前缀；cg_symbol_tool! 宏生成 CallersTool / CalleesTool / ImpactTool；ExploreTool 接自然语言 query；FilesTool 接可选 glob filter；索引非空时 5 工具入 registry，索引空 / 非代码仓按 FINDING-009 不注册并给地形提示。
   - docs/contexts/结构图与MCP/01-graph-tools.md:26-44（审计 #60）
26. McpClient：持 child + stdin + reader + next_id + timeout 五字段；McpClient::spawn 用 tokio Command 拉起 server，stderr 置 null；initialize 走 initialize 请求 + notifications/initialized 通知；call_tool 走 tools/call，extract_tool_text 拼接 text 内容；request 自增 next_id 后写一行 JSON-RPC，循环 read_line 直到匹配 id 响应（90s 超时），非法行与服务端通知一律跳过；Drop 中 try_lock child 并 start_kill。
   - docs/contexts/结构图与MCP/02-mcp-client.md:26-48（审计 #63）
27. CS-段位码体系：CS1xxx → exit 1（用法，含 CS1012 未知名）/ CS2xxx → 3（LLM 段位 2001/2002/2003/2004/2099）/ CS3xxx → 4（目标库，含 CS3001 REPO_NOT_FOUND、CS3003 FENCE_DENIED）/ CS4xxx → 5（索引，含 CS4016 INDEX_LOCKED）/ 其它 → 6；cli 三个顶层入口（run / run_task / run_config）捕获 CsError 后调 report_error 打 stderr 并按 exit_code 退出。
   - docs/contexts/错误与日志/01-error-codes.md:22-24（审计 #65）
   - docs/contexts/安全与防护/01-fence.md:36-38（审计 #49）
   - docs/contexts/安全与防护/02-bootlock.md:30-32（审计 #51）
28. 日志双轨：init_tracing 通过 tracing registry 同时挂 stderr_layer + file_layer；stderr 人读 + ~/.codesleuth/logs/&lt;session&gt;.log 归档，共用同一 EnvFilter 与同一格式；--json 模式下 stdout 仍只承载报告 JSON；SessionLog 单代 64MB 轮转（超限 rename .log.old，append 打开）；session span 内自动带 session_id 串线；文件层打开失败仅 eprintln 警告并降级为仅 stderr。
   - docs/contexts/错误与日志/02-logging.md:26-39（审计 #67）

## 死胡同
- 结构图与MCP/01-graph-tools.md 与 02-mcp-client.md 详稿确认 McpClient 在 Drop 中 try_lock 并 start_kill 回收子进程，但 codegraph serve --mcp 在 5 个 tool 调用之间崩溃时 McpClient 错误传播路径与 Harness 主循环是否把它计入 no_progress 熔断计数，未在本次侦察中走通完整闭环。
- 安全与防护/01-fence.md 标注 FuzzyEngine::new 内部自建一份 Fence 仅取规范化根（fence.root()），未走 fence.resolve() 守门；与 ReadTool 严格守门形成不对称，是否构成风险未在本会话裁定。
- 向量检索/04-vector-recall.md 标注 RecallEngine::tombstone / compact 路径 callers 查询仅返回 2 个测试调用方（recall.rs:219、245），未发现生产调用方；据此判断 compact 路径在当前仓库内为预留入口，是否被 build / commit_build 触发未有定论。
- 上下文与报告/01-context-compaction.md 与 02-report.md 共同标注：grep write_markdown / finish_report / report_path / finish_session 0 命中，报告持久化写盘函数与触发时序（run_task 退出前 / 异常路径）未在本会话被读到，仅由 D013 决策 + AGENTS.md + config.rs:164 约定 ~/.codesleuth/reports/&lt;session&gt;.md|.json 落点。
- 向量检索/06-repo-map.md 模块头注释（src/vector/repomap.rs:1-4 任务书措辞说写入 system 尾部 [repo map] 段）与 harness.rs:67-72, 74-88 实际实现（用 with_first_user_suffix 走 messages.last_mut() 把段追加到首条 User 消息 content，system 消息保持 SYSTEM_PROMPT 恒定）不一致；详稿以代码为准，文档侧未拉齐。
- tools/vector_search.rs 等各 Tool 中 observe / observe_exact 的精确调用点（仅确认 Harness 持有 evidence 字段）以及 src/report.rs Finding 结构如何消费 audit_seq 字段未深挖；submit_report 路径闭环已通过 build_report 校验覆盖。

## 置信度
high

## 统计
turns=11 · tool_calls=31 · duration=176509ms · tokens=623755
