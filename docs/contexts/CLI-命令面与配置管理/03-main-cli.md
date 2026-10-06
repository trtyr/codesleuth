# 主侦察命令（CLI 命令面与配置管理）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`03-main-cli-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 主侦察命令（CLI 命令面与配置管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
主侦察命令是 codesleuth 的核心 CLI 入口：用户传入自然语言任务与目标仓库，经配置加载、工具注册（read/grep/graph/vector）与 Agent harness 驱动 LLM 完成检索，最终输出带证据的结构化报告（stdout/--out/落盘 reports/）。调用链：main → Cli::parse → Cli::run → run_task_inner → harness::Harness::run。

## 证据列表
1. Cli 结构定义任务/repo/--json/--out/--fresh-index/--vector/--repo-map 等参数，为 clap Parser 入口
   - src/cli.rs:14-55（审计 #2）
2. main 解析 Cli、生成 session_id 并调用 cli.run
   - src/main.rs:5-11（审计 #12）
3. run_task_inner 校验 task/repo、加载配置、构造 LLM provider 与工具注册表（read/fuzzy），最后由 Harness::run 驱动 agent
   - src/cli.rs:190-244（审计 #2）
   - src/cli.rs:353-365（审计 #2）
4. 可选注册 codegraph 图工具（explore/callers/callees/impact/files）与向量召回层，均支持非致命降级并留痕审计
   - src/cli.rs:252-296（审计 #2）
   - src/cli.rs:299-328（审计 #2）
5. 任务结束后序列化结构化报告，按 --json/--out 输出并落盘 ~/.codesleuth/reports/{session}.md/.json
   - src/cli.rs:423-447（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=4 · tool_calls=5 · duration=23427ms · tokens=22918

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
主侦察命令（无子命令直接 run）的完整链路：main (src/main.rs:5-11) → Cli::parse → Cli::run (src/cli.rs:96-126) → run_task → run_task_inner (src/cli.rs:190-456)。run_task_inner 做七件事：①校验 task/repo 并 canonicalize (cli.rs:191-210)；②三层配置加载（CLI>项目>全局>默认，env 层已整体移除）+ api_key 解析 (cli.rs:212-217, src/config.rs:174-200,458-468)；③建 ~/.codesleuth 审计 Audit (cli.rs:227-229)；④构造 OpenAiProvider 与基础工具注册（read / find_files / grep，经 Fence 只读围栏）(cli.rs:231-244)；⑤codegraph 结构图引擎启动，成功且有符号时注册 explore/callers/callees/impact/files 五个图工具，无符号则不注册并注入「地形提示」（诚实工具面）(cli.rs:252-296)；⑥--vector 时 setup_vector_layer：补建向量索引（引导锁串行）→ 开库 → 注册 vector_search → 任务召回（top10）+ 依赖导航图作为首条用户消息后缀注入 (cli.rs:299-328, 534-659)；⑦Harness::run 驱动 agent 循环（系统提示+任务+后缀 → LLM → 工具去重/熔断/压缩 → submit_report 收敛），结束后做零写入自证 write_check，报告落盘 ~/.codesleuth/reports/{session}.md/.json，再按 --json/--out 输出 (cli.rs:353-455, src/harness.rs:74-419)。配置面：config 子命令 get/set/path 表驱动 13 个键 (cli.rs:474-505, config.rs:273-428)；index 子命令仅支持 --vector，结构索引重建唯一入口是 run --fresh-index (cli.rs:462-472)。初稿行号与结论基本准确，可升级为：调用链补 harness 内部结构、配置全键默认值、降级/熔断边界。

## 证据列表
1. 入口链：main 生成 session_id 并初始化 tracing 后调 Cli::run；run 按 session span 分派到 run_config / run_index_vector / run_task
   - src/main.rs:5-11（审计 #4）
   - src/cli.rs:96-126（审计 #2）
   - src/cli.rs:180-188（审计 #2）
2. run_task_inner 输入校验：task 空 → USER_INPUT(exit 1)；--repo 缺失 → USER_INPUT；路径 canonicalize 失败 → REPO_NOT_FOUND(exit 4)
   - src/cli.rs:190-244（审计 #2）
   - src/config.rs:174-200（审计 #12）
   - src/cli.rs:227-229（审计 #2）
3. 配置链：默认值 ← ~/.codesleuth/config.toml ← .codesleuth/config.toml(兼容 codesleuth.toml) ← CLI --model/--base-url；环境变量层已整体移除；api_key 只认配置文件直配，空白视为未配置
   - src/config.rs:1-3（审计 #12）
   - src/config.rs:107-133（审计 #12）
   - src/config.rs:458-468（审计 #12）
4. 配置默认值：llm.base_url=https://api.openai.com/v1，model=gpt-4o-mini，context.model_context_tokens=1_000_000，compact_at_percent=60，vector.embed_model=Qwen/Qwen3-Embedding-8B(1024维/composite)，repomap_budget=24_000，thinking_disabled=true(默认关思考)，graph.bin=codegraph
   - src/config.rs:110-130（审计 #12）
5. 图工具注册受双重开关：CodegraphEngine::start 成功（graph.bin，--fresh-index 可强制重建）且 .codegraph/codegraph.db 有符号才注册 explore/callers/callees/impact/files；否则注入地形提示引导 find_files/grep/read
   - src/cli.rs:252-296（审计 #2）
6. 向量层：--vector 时 setup_vector_layer 补建索引（bootlock 引导锁，竞争败者判负退出 exit 5，构建段结束即放锁）→ 注册 vector_search → recall(task,10) + --repo-map 任务导航图拼成首条消息后缀
   - src/cli.rs:534-659（审计 #2）
   - src/cli.rs:299-328（审计 #2）
   - src/cli.rs:559-563（审计 #2）
   - src/cli.rs:584（审计 #2）
7. Harness 循环：system=SYSTEM_PROMPT + user=task(+后缀)；内置工具 submit_report（收敛，证据校验失败拒回）与 recall（审计 seq 钻取）由 harness 直处理；普通工具按 同参去重→参数校验→执行；连续无进展 5 步熔断（MAX_NO_PROGRESS_STREAK=5），连续 2 回合零增量注入转向提示；无工具调用时先 prose_steer 引导提交，二次则降级 prose 报告
   - src/harness.rs:74-134（审计 #17）
   - src/harness.rs:222-271（审计 #17）
   - src/harness.rs:300-347（审计 #17）
   - src/harness.rs:176-212（审计 #17）
8. 上下文压缩：should_compact 超阈值（窗口×compact_at_percent）时 handoff + compact 三段式，有驱逐才记 compaction_begin/compaction 审计
   - src/harness.rs:100-124（审计 #17）
   - src/harness.rs:26-33（审计 #17）
9. 零写入自证：考前 writeguard::snapshot（.codegraph 豁免），考后 diff，无变更打 ✓；有不可归因变更 = ERROR 完整性违规，写审计 write_check 行；快照本身失败降级为 available:false 留痕
   - src/cli.rs:246-247（审计 #2）
   - src/cli.rs:369-421（审计 #2）
10. 输出落点：报告必落 ~/.codesleuth/reports/{session_id}.md 与 .json；--out 按是否有 --json 写 human 或 json 体；stdout 输出报告本体，stderr 输出 turns/tool_calls/报告与审计路径
   - src/cli.rs:423-455（审计 #2）
11. 退出码契约：0 就绪·1 用法·2 配置/凭据·3 上游 LLM·4 目标库·5 索引·6 内部；index 子命令不带 --vector 时诚实报错 INDEX_NOT_AVAILABLE，--rebuild 从不记录状态（重建唯一入口 run --fresh-index）
   - src/cli.rs:2（审计 #2）
   - src/cli.rs:462-472（审计 #2）
   - src/cli.rs:119-122（审计 #2）
12. 降级分级：INDEX_LOCKED（引导锁竞争）不降级直接退出；codegraph/向量层装配/索引构建/开库/召回失败均非致命，warn + 审计 degraded 行后继续
   - src/cli.rs:259-262（审计 #2）
   - src/cli.rs:315-327（审计 #2）
   - src/cli.rs:263-271（审计 #2）
   - src/cli.rs:575-583（审计 #2）
   - src/cli.rs:589-594（审计 #2）
13. config 子命令：get(缺省打印全部 to_file_view)/set(键表驱动，写 ~/.codesleuth/config.toml)/path；13 键表驱动，set/get/未知键提示三处同表
   - src/cli.rs:474-505（审计 #2）
   - src/cli.rs:661-692（审计 #2）
   - src/config.rs:273-428（审计 #12）
14. 嵌入供应商独立：vector.base_url/api_key 缺省跟随 llm 段（resolve_embed_endpoint），chat 与 embedding 可用两家供应商
   - src/cli.rs:507-530（审计 #2）
15. CLI 参数全集（Cli struct）：task(位置) repo focus(可多次glob) json out model base_url fresh_index vector repo_map verbose(-v/-vv) + 子命令 config/index
   - src/cli.rs:14-55（审计 #2）
   - src/harness.rs:18-23（审计 #17）

## 死胡同
- explore 一把梭 harness.rs 仅返回到 ~427 行（truncated），Report/render_human/build_report 细节未逐行读（推断与 RunOutcome/submit_report 语义一致，验证：read src/harness.rs:421-末尾）
- fence.rs / audit.rs / writeguard.rs / bootlock.rs 内部实现未逐行读，结论基于 cli.rs/harness.rs 调用点注释与签名（验证：read 对应文件）

## 置信度
high

## 统计
turns=6 · tool_calls=8 · duration=45585ms · tokens=112107


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
主侦察命令是 codesleuth 的核心 CLI 入口：用户传入自然语言任务与目标仓库，经配置加载、工具注册（read/grep/graph/vector）与 Agent harness 驱动 LLM 完成检索，最终输出带证据的结构化报告（stdout/--out/落盘 reports/）。调用链：main → Cli::parse → Cli::run → run_task_inner → harness::Harness::run。

## 证据列表
1. Cli 结构定义任务/repo/--json/--out/--fresh-index/--vector/--repo-map 等参数，为 clap Parser 入口
   - src/cli.rs:14-55（审计 #2）
2. main 解析 Cli、生成 session_id 并调用 cli.run
   - src/main.rs:5-11（审计 #12）
3. run_task_inner 校验 task/repo、加载配置、构造 LLM provider 与工具注册表（read/fuzzy），最后由 Harness::run 驱动 agent
   - src/cli.rs:190-244（审计 #2）
   - src/cli.rs:353-365（审计 #2）
4. 可选注册 codegraph 图工具（explore/callers/callees/impact/files）与向量召回层，均支持非致命降级并留痕审计
   - src/cli.rs:252-296（审计 #2）
   - src/cli.rs:299-328（审计 #2）
5. 任务结束后序列化结构化报告，按 --json/--out 输出并落盘 ~/.codesleuth/reports/{session}.md/.json
   - src/cli.rs:423-447（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=4 · tool_calls=5 · duration=23427ms · tokens=22918
