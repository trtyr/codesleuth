# -v 日志详细度（LLM 接入与运维）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`03-verbose-logging-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# -v 日志详细度（LLM 接入与运维）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
-v 旗标是 CLI 的日志详细度计数器（-v=info，-vv=debug，默认 warn），入口解析后在 main 中传给 init_tracing 建立 stderr + 会话日志文件的双轨 tracing 过滤，让用户/运维按需增减日志噪音并保留归档。

## 证据列表
1. -v 是 clap 计数旗标 verbose: u8，注释写明「-v info / -vv debug」
   - src/cli.rs:50-52（审计 #12）
2. init_tracing 将计数映射为过滤级别：0→warn、1→info、≥2→debug，构造 EnvFilter=codesleuth=<level>
   - src/lib.rs:27-37（审计 #2）
3. 日志双轨输出：stderr 人读 + ~/.codesleuth/logs/<session>.log 文件归档，同一过滤器共用；--json 模式下 stdout 只出报告 JSON
   - src/lib.rs:24-26, 39-65（审计 #2）
4. main 解析 Cli 后调用 init_tracing(cli.verbose, &session_id)，session_id 由 audit::new_session_id 生成，审计与日志共用身份
   - src/main.rs:5-10（审计 #14）

## 死胡同
无

## 置信度
high

## 统计
turns=4 · tool_calls=6 · duration=17058ms · tokens=15176


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
-v 日志详细度功能的完整链路已核实：main.rs 入口 Cli::parse → audit::new_session_id → init_tracing(cli.verbose, &session_id)（src/main.rs:5-12）→ init_tracing 将计数映射为过滤级别（0→warn、1→info、≥2→debug）构造 EnvFilter="codesleuth=<level>"（src/lib.rs:31-37），再挂 stderr 层 + ~/.codesleuth/logs/<session_id>.log 文件归档层到同一 registry 过滤器（src/lib.rs:39-65）。文件轨由 logs::SessionLog 实现：超 64MB 单代轮转到 .log.old，打开失败不阻断只降级 stderr（src/logs.rs:11,21-41；src/lib.rs:43-58）。main 之后 cli.run 在 session span 内执行全程，所有 tracing 事件自动携带 session_id（src/cli.rs:96-98）。初稿四条结论全部属实，无需更正；补充：verbose 对 config/index 子命令同样生效、无 RUST_LOG 环境变量支持、HOME 缺失时文件轨静默消失。

## 证据列表
1. 调用链第 1 步：main 中 Cli::parse() 解析 -v 计数旗标（#[arg(short='v', action=Count)] verbose: u8），测试断言 -vv → verbose=2
   - src/cli.rs:50-52（审计 #4）
   - src/cli.rs:756-762（审计 #4）
2. 调用链第 2 步：main 生成 session_id（audit::new_session_id）后调用 init_tracing(cli.verbose, &session_id)，再 cli.run(&session_id) 进入 session span，全程日志事件携带 session_id
   - src/main.rs:5-12（审计 #6）
   - src/cli.rs:96-98（审计 #4）
3. 级别映射：init_tracing 内 0→"warn"、1→"info"、_→"debug"，构造 EnvFilter::try_new(format!("codesleuth={level}"))，解析失败回退 EnvFilter::new("warn")
   - src/lib.rs:27-37（审计 #2）
4. 双轨输出：stderr 层（with_writer(io::stderr)）与文件层共用同一 registry 过滤器；文件层带 with_ansi(false) 去色码
   - src/lib.rs:39-65（审计 #2）
5. 文件轨落点：global_state_dir()=dirs::home_dir()/.codesleuth，日志路径 <dir>/logs/<session_id>.log；HOME 缺失时 and_then 返回 None，文件轨静默关闭（警告仅打印在 open 出错分支 lib.rs:53-55）
   - src/lib.rs:43-58（审计 #2）
   - src/config.rs:148-150（审计 #14）
6. SessionLog：单代轮转（超 LOG_ROTATE_BYTES=64MB → rename 为 .log.old，轮转失败退化为继续追加），append 模式创建，Arc<Mutex<File>> 线程安全并实现 MakeWriter 供 tracing 层使用
   - src/logs.rs:10-41（审计 #9）
   - src/logs.rs:64-70（审计 #9）
7. verbose 全局生效：init_tracing 在子命令分发前执行，config/index 子命令也受 -v 影响。下游日志消费者含 run_task_inner 的『配置就绪』info、harness 的 tool_exec debug（-vv 可观测逐工具调用与 audit_seq）
   - src/cli.rs:103-125（审计 #4）
   - src/cli.rs:218-223（审计 #4）
   - src/harness.rs:350-355（审计 #40）
8. stdout 纪律：--json 模式下 stdout 仅报告 JSON，日志一律走 stderr + 文件两个过程通道（代码注释多处强调，属跨功能契约）
   - src/lib.rs:26（审计 #2）
   - src/logs.rs:1-3（审计 #9）
   - src/cli.rs:29-31（审计 #4）
9. 配置与开关边界：日志级别只受 -v 计数影响，无 RUST_LOG / CODESLEUTH_* 环境变量参与（grep 零命中）；config.toml 也不含日志键（推断：grep 'verbose' 仅命中 cli.rs/lib.rs/docs）
   - src/config.rs:142-159（审计 #14）

## 死胡同
- grep CODESLEUTH_* 环境变量：0 命中（无该前缀环境变量）
- grep RUST_LOG/env::var：0 命中（过滤级别不受任何环境变量影响）
- grep tracing::(warn|info|debug)：宏展开跨行导致 regex 不中，改用 tracing:: 宽匹配补齐

## 置信度
high

## 统计
turns=11 · tool_calls=15 · duration=64363ms · tokens=144830


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
-v 旗标是 CLI 的日志详细度计数器（-v=info，-vv=debug，默认 warn），入口解析后在 main 中传给 init_tracing 建立 stderr + 会话日志文件的双轨 tracing 过滤，让用户/运维按需增减日志噪音并保留归档。

## 证据列表
1. -v 是 clap 计数旗标 verbose: u8，注释写明「-v info / -vv debug」
   - src/cli.rs:50-52（审计 #12）
2. init_tracing 将计数映射为过滤级别：0→warn、1→info、≥2→debug，构造 EnvFilter=codesleuth=<level>
   - src/lib.rs:27-37（审计 #2）
3. 日志双轨输出：stderr 人读 + ~/.codesleuth/logs/<session>.log 文件归档，同一过滤器共用；--json 模式下 stdout 只出报告 JSON
   - src/lib.rs:24-26, 39-65（审计 #2）
4. main 解析 Cli 后调用 init_tracing(cli.verbose, &session_id)，session_id 由 audit::new_session_id 生成，审计与日志共用身份
   - src/main.rs:5-10（审计 #14）

## 死胡同
无

## 置信度
high

## 统计
turns=4 · tool_calls=6 · duration=17058ms · tokens=15176
