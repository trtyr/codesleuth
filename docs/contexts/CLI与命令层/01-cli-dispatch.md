# CLI入口与命令分发（CLI与命令层）

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
# CLI入口与命令分发（CLI与命令层）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
CLI 入口与命令分发是 codesleuth 的进程入口：main() 解析参数、生成 session_id、初始化结构化日志双轨（stderr + 会话文件），随后把控制权交给 Cli::run，按子命令分派到 Config / Index（向量） / run_task。给用户的价值是「一个二进制、按意图路由」：配置查看与写入、向量索引构建、只读侦察任务三类操作同走同一条入口与日志/审计身份。

## 证据列表
1. main() 在 src/main.rs 解析 CLI、生成 session_id、初始化日志后调用 Cli::run。
   - src/main.rs:5-12（审计 #2）
   - src/main.rs:6（审计 #2）
   - src/main.rs:8（审计 #2）
   - src/main.rs:9-10（审计 #2）
2. 日志双轨由 lib::init_tracing 装配：stderr 人读 + ~/.codesleuth/logs/<session>.log 文件归档，同过滤同格式。
   - src/lib.rs:24-27（审计 #14）
   - src/lib.rs:39-58（审计 #14）
   - src/lib.rs:61-65（审计 #14）
3. Cli 是 clap 派生的命令行结构（任务、--repo、--vector、--fresh-index、--json、--out、-v、Config/Index 子命令等），是一级分派的承载体。
   - src/cli.rs:14-58（审计 #2）
   - src/cli.rs:60-96（审计 #2）
4. Cli::run 用 match self.command 做一级分派：Config → run_config；Index 且 --vector → run_index_vector；其它 Index → 诚实报错；None → run_task。
   - src/cli.rs:99-129（审计 #2）
5. run_task 内调 run_task_inner 装配 config、audit、fence、tools 注册、codegraph MCP、向量层、harness，再交给 agent.run。
   - src/cli.rs:184-192（审计 #2）
   - src/cli.rs:194-371（审计 #2）

## 死胡同
- 未读 src/cli.rs:76-805 中 ConfigAction 详细分支与 parses_* 辅助函数；本次只验证入口与一级分派，不影响四节结论。

## 置信度
high

## 统计
turns=4 · tool_calls=6 · duration=26925ms · tokens=33794


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
CLI 入口与命令分发是 codesleuth 的进程入口：main() 解析参数、生成 session_id、初始化结构化日志双轨（stderr + 会话文件），随后把控制权交给 Cli::run，按子命令分派到 Config（run_config：Get/Set/Path）/ Index（run_index_vector：仅 --vector；其它 index 子命令走 index_structure_error 诚实报错）/ run_task（→ run_task_inner 装配 config、audit、fence、tools 注册、codegraph MCP、向量层、harness，再交给 Harness::run）。给用户的价值是「一个二进制、按意图路由」，日志与审计共享 session_id 同一身份。

## 证据列表
1. main() 在 src/main.rs 解析 CLI、生成 session_id、初始化日志后调用 Cli::run，并把退出码交回 std::process::exit
   - src/main.rs:5-12（审计 #2）
2. session_id 由 audit::new_session_id 生成（纳秒时间 + pid 的 hex），保证日志与审计同身份
   - src/main.rs:8（审计 #2）
   - src/audit.rs:53-60（审计 #15）
3. 日志双轨由 lib::init_tracing 装配：stderr 人读 + ~/.codesleuth/logs/<session>.log 文件归档，同过滤同格式；session span 携带 session_id；stdout 纪律在 --json 下只允许报告 JSON
   - src/lib.rs:24-27（审计 #6）
   - src/lib.rs:39-58（审计 #6）
   - src/lib.rs:61-65（审计 #6）
   - src/logs.rs:10-46（审计 #15）
4. Cli 是 clap 派生的命令行结构：任务、--repo、--focus、--json、--out、--model、--base-url、--profile、--fresh-index、--vector、--repo-map、-v 计数、Config/Index 子命令
   - src/cli.rs:14-58（审计 #4）
   - src/cli.rs:60-96（审计 #4）
5. Cli::run 用 match self.command 做一级分派：Config → run_config(action, profile)；Index 且 --vector → Self::run_index_vector；其它 Index → index_structure_error 诚实报错（CS4010 INDEX_NOT_AVAILABLE + hint 提示用 run --fresh-index）；None → self.run_task(session_id)
   - src/cli.rs:99-129（审计 #4）
   - src/cli.rs:465-478（审计 #4）
6. run_index_vector：dunce::canonicalize → config::load → resolve_api_key → resolve_embed_endpoint（[vector] 缺省跟随 [llm]）→ EmbedClient → bootlock 引导锁 → tokio block_on build_vector_index → eprintln! 摘要 chunks/embedded/reused/gc_removed
   - src/cli.rs:132-182（审计 #4）
   - src/cli.rs:524-536（审计 #4）
7. run_task 内调 run_task_inner 装配：校验 task + repo + canonicalize → config.load → resolve_api_key → info! 配置就绪 → state_dir + Audit::create → OpenAiProvider → Fence + ToolRegistry 注册 read/fuzzy 三件套 → writeguard 考前快照 → tokio Runtime + codegraph MCP（失败按 INDEX_LOCKED 判负或非致命降级留痕）→ 仅 codegraph 有符号时注册 explore/callers/callees/impact/files 五件套，否则下发「地形提示」到首条 user 后缀 → --vector 时 setup_vector_layer（build_vector_index + RecallEngine + vector_search 工具 + 召回块 + 任务导航图）→ 兜底 --repo-map 注入全局导航图 → Harness::new + with_first_user_suffix → rt.block_on(agent.run(task)) → writeguard 考后 diff 留痕 write_check → 报告持久化（~/.codesleuth/reports/<session>.{md,json} + --out）→ stdout 打报告 + stderr 打 turns/tool_calls/报告/审计路径摘要
   - src/cli.rs:184-192（审计 #4）
   - src/cli.rs:194-371（审计 #4）
   - src/cli.rs:372-462（审计 #4）
8. run_config 走 match ConfigAction：Path → global_config_path + project_config_path 双路径打印；Get → config::load + config::to_file_view 或 config::resolved_get；Set → config::load + apply_set + TOML 写回 global_config_path
   - src/cli.rs:480-511（审计 #4）
   - src/cli.rs:667-701（审计 #4）
9. 配置加载链 load：默认 Config ← 全局文件（~/.codesleuth/config.toml，可选）← 项目文件（.codesleuth/config.toml → 旧 ./codesleuth.toml 兼容）← apply_profile（D017 档位）← merge_cli；CliOverrides 仅 base_url/model/profile 三项
   - src/config.rs:150-157（审计 #22）
   - src/config.rs:160-176（审计 #22）
   - src/config.rs:190-223（审计 #22）
   - src/config.rs:303-310（审计 #22）
10. resolve_api_key 只认配置文件直配，env 层已移除；值为空视为未配置（CONFIG_MISSING CS1011）
   - src/config.rs:526-538（审计 #22）
11. 错误体系：CS1xxx→exit 1 用法、CS2xxx→3 LLM、CS3xxx→4 目标库、CS4xxx→5 索引、CS5xxx→6 内部；CsError 含 code/message/hint/retryable/source_text；report_error 打 stderr + 根因 + hint
   - src/errors.rs:8-22（审计 #31）
   - src/errors.rs:31-53（审计 #31）
   - src/errors.rs:56-115（审计 #31）
12. bootlock 跨进程 flock（std::fs::File::try_lock，Rust 1.89）：<repo>/.codesleuth/boot.lock，三态 Won / Lost / 超时统一以 CS4016 INDEX_LOCKED 判负退出，不降级；DEFAULT_TIMEOUT 300s；acquire_guard 收残判负语义，cli 三处调用共享锁
   - src/bootlock.rs:21-24（审计 #15）
   - src/bootlock.rs:27-48（审计 #15）
   - src/bootlock.rs:50-114（审计 #15）
   - src/bootlock.rs:116-129（审计 #15）
   - src/cli.rs:162-166（审计 #4）
   - src/cli.rs:565-569（审计 #4）
13. Audit JSONL：每行 { seq, ts_ms, kind, ...payload }，seq 在锁内取号保证单调不重不漏；create 时若超 64MB 单代轮转 .old；Audit 持 Mutex&lt;File&gt; + AtomicU64 seq；append_line 为会话后宿主级事件续号
   - src/audit.rs:62-103（审计 #15）
   - src/audit.rs:136-163（审计 #15）
   - src/audit.rs:18-51（审计 #15）
14. Harness::new 接 provider/tools/audit/model/context_tokens/compact_percent；with_first_user_suffix 在首条 user 消息末尾追加召回块 + 任务导航图（D005 分层：system 恒定、任务派生数据走 user）
   - src/harness.rs:47-72（审计 #44）
   - src/harness.rs:74-88（审计 #44）
15. index 非 --vector 错误文案由 index_structure_error 构造：rebuild=true 时 hint 指 run --fresh-index（不再撒谎「已记录」），否则 hint 指 --vector；测试在 src/cli.rs:709-718 断言
   - src/cli.rs:465-478（审计 #4）
   - src/cli.rs:709-718（审计 #4）
16. 关键单测：cli_definition_valid、parses_task_mode、parses_full_surface、parses_config_subcommands、missing_task_is_usage_error_exit_1（断言 USER_INPUT → exit 1）
   - src/cli.rs:734-804（审计 #4）

## 死胡同
- 未深入 src/harness.rs:74 之后 RunOutcome 字段填充与熔断主循环的逐行实现（src/harness.rs:91-1149），本报告只确认 run_task_inner 把它作为黑盒调用
- 未读 src/tools/graph.rs:78 之后 codegraph MCP serve 与符号表注入的逐行实现（初稿死胡同延续）
- 未读 src/config.rs:331-540 中 config_key_table 全部 13 键的 set/get 闭包表逐行实现

## 置信度
high

## 统计
turns=12 · tool_calls=19 · duration=102051ms · tokens=328455


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
CLI 入口与命令分发是 codesleuth 的进程入口：main() 解析参数、生成 session_id、初始化结构化日志双轨（stderr + 会话文件），随后把控制权交给 Cli::run，按子命令分派到 Config / Index（向量） / run_task。给用户的价值是「一个二进制、按意图路由」：配置查看与写入、向量索引构建、只读侦察任务三类操作同走同一条入口与日志/审计身份。

## 证据列表
1. main() 在 src/main.rs 解析 CLI、生成 session_id、初始化日志后调用 Cli::run。
   - src/main.rs:5-12（审计 #2）
   - src/main.rs:6（审计 #2）
   - src/main.rs:8（审计 #2）
   - src/main.rs:9-10（审计 #2）
2. 日志双轨由 lib::init_tracing 装配：stderr 人读 + ~/.codesleuth/logs/<session>.log 文件归档，同过滤同格式。
   - src/lib.rs:24-27（审计 #14）
   - src/lib.rs:39-58（审计 #14）
   - src/lib.rs:61-65（审计 #14）
3. Cli 是 clap 派生的命令行结构（任务、--repo、--vector、--fresh-index、--json、--out、-v、Config/Index 子命令等），是一级分派的承载体。
   - src/cli.rs:14-58（审计 #2）
   - src/cli.rs:60-96（审计 #2）
4. Cli::run 用 match self.command 做一级分派：Config → run_config；Index 且 --vector → run_index_vector；其它 Index → 诚实报错；None → run_task。
   - src/cli.rs:99-129（审计 #2）
5. run_task 内调 run_task_inner 装配 config、audit、fence、tools 注册、codegraph MCP、向量层、harness，再交给 agent.run。
   - src/cli.rs:184-192（审计 #2）
   - src/cli.rs:194-371（审计 #2）

## 死胡同
- 未读 src/cli.rs:76-805 中 ConfigAction 详细分支与 parses_* 辅助函数；本次只验证入口与一级分派，不影响四节结论。

## 置信度
high

## 统计
turns=4 · tool_calls=6 · duration=26925ms · tokens=33794
