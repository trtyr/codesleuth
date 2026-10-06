# 命令行旗标集（CLI 命令面与配置管理）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`04-cli-flags-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 命令行旗标集（CLI 命令面与配置管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
CLI 命令旗标集由 src/cli.rs 的 Cli 结构体（clap derive）集中定义（--repo/--focus/--json/--out/--model/--base-url/--fresh-index/--vector/--repo-map/-v），入口 main.rs 调 Cli::parse → Cli::run 分发：无子命令时进入 run_task_inner 逐旗标生效（--repo 校验 canonicalize、--fresh-index 传给 codegraph 启动、--vector 装配向量层、--repo-map 构建导航图注入首条消息、--json/--out 控制 stdout/落盘），index --vector 子命令走 run_index_vector 预建向量索引，config 子命令走 run_config。上游依赖 config::load（合并 --model/--base-url 覆盖）与 bootlock 引导锁，下游消费工具注册表、harness::Harness（驱动 LLM 侦察）、审计日志与报告持久化。

## 证据列表
1. 全部旗标在 Cli 结构体集中定义：--repo(--repo) --focus(可多次 glob) --json --out --model --base-url --fresh-index(强制重建索引) --vector(向量召回) --repo-map(repo map 注入) -v 日志级别
   - src/cli.rs:20-55（审计 #2）
2. 入口为 main.rs：Cli::parse → init_tracing(verbose) → cli.run(session_id)；Cli::run 按子命令分发到 run_config / run_index_vector / run_task
   - src/main.rs:5-11（审计 #4）
   - src/cli.rs:96-126（审计 #2）
3. run_task_inner 校验 task 与 --repo（缺失报 USER_INPUT 并给用法提示，canonicalize 报 REPO_NOT_FOUND），随后 config::load(overrides) 合并 --model/--base-url 覆盖
   - src/cli.rs:190-223（审计 #2）
4. --fresh-index 透传给 CodegraphEngine::start；--vector 触发 setup_vector_layer；仅 --repo-map 时用 vector::repomap 构建全局导航图注入首条消息
   - src/cli.rs:252-253（审计 #2）
   - src/cli.rs:299-311（审计 #2）
   - src/cli.rs:334-351（审计 #2）
5. --out 将报告写入指定文件（--json 时为 JSON 否则 Markdown），--json 控制 stdout 仅输出 JSON 报告；报告同时持久化到 ~/.codesleuth/reports/（cli.rs:428-436）
   - src/cli.rs:437-447（审计 #2）
6. index 子命令仅支持 --vector 走 run_index_vector（预建向量索引，先 acquire_guard 引导锁），非 vector 时 index_structure_error 诚实报错并提示 run --fresh-index
   - src/cli.rs:128-163（审计 #2）
   - src/cli.rs:459-472（审计 #2）
7. 旗标集下游消费：工具注册表（read/fuzzy/graph 五工具，vector 层按 --vector 注册）、harness::Harness::run 驱动 LLM 侦察
   - src/cli.rs:240-296（审计 #2）
   - src/cli.rs:353-365（审计 #2）
8. config 子命令（path/get/set）由 run_config 分发到 config 模块，属于配置管理面
   - src/cli.rs:474-505（审计 #2）

## 死胡同
- grep 'cli.focus' 无命中：--focus 的运行时消费点未定位，仅在 Cli 定义（src/cli.rs:26-28）与测试/README 中出现
- grep 'fn run\(|CliOverrides' 正则模式 0 命中，改用 plain 'run_task' 检索后定位

## 置信度
high

## 统计
turns=12 · tool_calls=11 · duration=42588ms · tokens=88164

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
CLI 旗标集由 src/cli.rs 的 Cli（clap derive, :20-55）集中定义，入口 main.rs:5-12 Cli::parse → init_tracing → cli.run 三分发：config→run_config(:474)、index --vector→run_index_vector(:128)、无子命令→run_task_inner(:190)。配置经 config::load 三层合并（CLI > 项目 .codesleuth/config.toml > 全局 ~/.codesleuth/config.toml > 默认，环境变量层已整体移除，src/config.rs:3,174-201），下游装配 Provider/审计/工具注册表/CodegraphEngine/向量层，最终经 harness::Harness::run 产出报告落 ~/.codesleuth/reports/ 与 --out。新发现：--focus 旗标确认无运行时消费点（已定义未接线，README:96 却在宣传它）；INDEX_LOCKED 是唯一不降级的索引错误（引导锁败者判负退出）；index --rebuild 是不实旗标（重建唯一入口 run --fresh-index）。

## 证据列表
1. 入口链：main → Cli::parse → new_session_id → init_tracing(verbose) → cli.run(session_id) → exit(code)；Cli::run 组 CliOverrides{base_url,model} 后按子命令三分发
   - src/main.rs:5-12（审计 #4）
   - src/cli.rs:96-126（审计 #2）
2. run_task_inner 主链：task/repo 校验+canonicalize(:191-210) → config::load 合并 CLI 覆盖(:216) → resolve_api_key(:217) → global_state_dir+Audit::create(:227-229) → Provider(:231) → 工具注册(:239-244)
   - src/cli.rs:190-229（审计 #2）
   - src/config.rs:174-201（审计 #17）
3. --fresh-index 透传 CodegraphEngine::start(:252-253)；图工具按符号非空诚实注册否则地形提示(:275-295)；--vector 走 setup_vector_layer(:299-328, 实现 :534-659)；仅 --repo-map 时全局导航图注入(:334-351)
   - src/cli.rs:252-296（审计 #2）
   - src/cli.rs:299-351（审计 #2）
4. 配置三层合并链：默认值←全局 ~/.codesleuth/config.toml←项目 .codesleuth/config.toml(兼容 codesleuth.toml)←CLI --model/--base-url；环境变量层已整体移除，api_key 只认配置文件（默认值见 Config::default :107-133）
   - src/config.rs:3（审计 #17）
   - src/config.rs:107-133（审计 #17）
   - src/config.rs:458-468（审计 #17）
   - src/cli.rs:245-252（审计 #2）
5. 错误处理契约：INDEX_LOCKED 引导锁竞争败者判负退出不降级（:259-262, :315-318）；codegraph 未就绪/向量构建失败/召回失败均弹性降级并审计 degraded 留痕；退出码 0-6 定义于 cli.rs:2
   - src/cli.rs:259-271（审计 #2）
   - src/cli.rs:315-326（审计 #2）
   - src/cli.rs:575-582（审计 #2）
6. 数据落点：报告持久化 ~/.codesleuth/reports/{sid}.md|.json(:428-436)，--out 落盘(:437-441)，stdout --json 时仅 JSON(:443-447)；考前/考后 writeguard 快照零写入自证(:247, :370-421)；会话日志 ~/.codesleuth/logs/{sid}.log(src/lib.rs:44-46)
   - src/cli.rs:428-447（审计 #2）
   - src/cli.rs:370-421（审计 #2）
   - src/lib.rs:43-58（审计 #39）
7. index --rebuild 从不记录状态（不实旗标），结构重建唯一入口 run --fresh-index（index_structure_error, cli.rs:459-472）；run_index_vector 先 acquire_guard 引导锁再 build_vector_index(:128-178)
   - src/cli.rs:459-472（审计 #2）
   - src/cli.rs:128-178（审计 #2）
8. 初稿死胡同已收口：grep 全仓 focus 仅命中定义(cli.rs:28)、测试(cli.rs:745,747,760)与 README 示例(README.md:96)——--focus 运行时确无消费点，已定义未接线
   - src/cli.rs:28（审计 #2）
   - src/cli.rs:745-760（审计 #2）
   - README.md:92-99（审计 #44）
9. 嵌入端点独立供应商：[vector].base_url/api_key 缺省跟随 [llm]（resolve_embed_endpoint, cli.rs:520-530）；向量构建引导锁构建段结束即放、不跨 LLM 调用(:584)
   - src/cli.rs:520-530（审计 #2）
   - src/cli.rs:557-563（审计 #2）

## 死胡同
- grep env::var / XDG / CODESLEUTH 全仓 0 命中——佐证环境变量层已移除而非死胡同
- docs/contexts/CLI-命令面与配置管理/04-cli-flags.md 即初稿自身，无增量信息
- harness::Harness 内部首条消息 suffix 拼接未逐行读（相邻功能域）

## 置信度
high

## 统计
turns=13 · tool_calls=16 · duration=148653ms · tokens=266591


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
CLI 命令旗标集由 src/cli.rs 的 Cli 结构体（clap derive）集中定义（--repo/--focus/--json/--out/--model/--base-url/--fresh-index/--vector/--repo-map/-v），入口 main.rs 调 Cli::parse → Cli::run 分发：无子命令时进入 run_task_inner 逐旗标生效（--repo 校验 canonicalize、--fresh-index 传给 codegraph 启动、--vector 装配向量层、--repo-map 构建导航图注入首条消息、--json/--out 控制 stdout/落盘），index --vector 子命令走 run_index_vector 预建向量索引，config 子命令走 run_config。上游依赖 config::load（合并 --model/--base-url 覆盖）与 bootlock 引导锁，下游消费工具注册表、harness::Harness（驱动 LLM 侦察）、审计日志与报告持久化。

## 证据列表
1. 全部旗标在 Cli 结构体集中定义：--repo(--repo) --focus(可多次 glob) --json --out --model --base-url --fresh-index(强制重建索引) --vector(向量召回) --repo-map(repo map 注入) -v 日志级别
   - src/cli.rs:20-55（审计 #2）
2. 入口为 main.rs：Cli::parse → init_tracing(verbose) → cli.run(session_id)；Cli::run 按子命令分发到 run_config / run_index_vector / run_task
   - src/main.rs:5-11（审计 #4）
   - src/cli.rs:96-126（审计 #2）
3. run_task_inner 校验 task 与 --repo（缺失报 USER_INPUT 并给用法提示，canonicalize 报 REPO_NOT_FOUND），随后 config::load(overrides) 合并 --model/--base-url 覆盖
   - src/cli.rs:190-223（审计 #2）
4. --fresh-index 透传给 CodegraphEngine::start；--vector 触发 setup_vector_layer；仅 --repo-map 时用 vector::repomap 构建全局导航图注入首条消息
   - src/cli.rs:252-253（审计 #2）
   - src/cli.rs:299-311（审计 #2）
   - src/cli.rs:334-351（审计 #2）
5. --out 将报告写入指定文件（--json 时为 JSON 否则 Markdown），--json 控制 stdout 仅输出 JSON 报告；报告同时持久化到 ~/.codesleuth/reports/（cli.rs:428-436）
   - src/cli.rs:437-447（审计 #2）
6. index 子命令仅支持 --vector 走 run_index_vector（预建向量索引，先 acquire_guard 引导锁），非 vector 时 index_structure_error 诚实报错并提示 run --fresh-index
   - src/cli.rs:128-163（审计 #2）
   - src/cli.rs:459-472（审计 #2）
7. 旗标集下游消费：工具注册表（read/fuzzy/graph 五工具，vector 层按 --vector 注册）、harness::Harness::run 驱动 LLM 侦察
   - src/cli.rs:240-296（审计 #2）
   - src/cli.rs:353-365（审计 #2）
8. config 子命令（path/get/set）由 run_config 分发到 config 模块，属于配置管理面
   - src/cli.rs:474-505（审计 #2）

## 死胡同
- grep 'cli.focus' 无命中：--focus 的运行时消费点未定位，仅在 Cli 定义（src/cli.rs:26-28）与测试/README 中出现
- grep 'fn run\(|CliOverrides' 正则模式 0 命中，改用 plain 'run_task' 检索后定位

## 置信度
high

## 统计
turns=12 · tool_calls=11 · duration=42588ms · tokens=88164
