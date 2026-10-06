# find_files 文件查找（检索工具面（只读））

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`03-tool-find-files-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# find_files 文件查找（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
find_files 是一个基于 fff-search 的模糊文件名查找工具（frecency 排序），输入 query（文件名/路径片段，支持约束语法）和可选 limit（默认 20，上限 100），帮助调用方按名字快速定位仓库内文件。入口是 FileFinderTool（src/tools/fuzzy.rs:41-110），CLI 启动时创建会话级 FuzzyEngine 并注册该工具（src/cli.rs:242-244）；执行时经 QueryParser 解析 query 后调用共享 FuzzyEngine 中 FilePicker.fuzzy_search 完成检索（src/tools/fuzzy.rs:86-92）。它与 GrepTool（内容检索）共享同一引擎、同属「读层+模糊层」检索工具面，上游依赖 crate::errors、crate::fence::Fence 与 fff-search crate；集成测试（tests/adversarial.rs、tests/replay.rs、tests/layered_live.rs）也将它与 GrepTool 注册在同一 ToolRegistry。

## 证据列表
1. FileFinderTool 是名为 find_files 的 Tool，按名字模糊找文件（frecency 排序），参数为 query（必填）与 limit（默认20，上限100）
   - src/tools/fuzzy.rs:40-71（审计 #2）
2. execute 解析 query/limit，QueryParser 解析后调用 engine.picker.fuzzy_search，输出编号路径列表，超限时提示可提高 limit
   - src/tools/fuzzy.rs:73-108（审计 #2）
3. CLI 启动时创建 FuzzyEngine 并把 FileFinderTool 与 GrepTool 注册进 ToolRegistry（层进 v0：读层+模糊层）
   - src/cli.rs:238-244（审计 #10）
4. FuzzyEngine 为会话级引擎，watch=false 同步收集文件，封装 fff-search FilePicker；FileFinderTool 与 GrepTool 共享同一 Arc<FuzzyEngine>（41-43、113-115）
   - src/tools/fuzzy.rs:17-37（审计 #2）
5. 上游依赖：crate::errors 错误码、crate::fence::Fence（取规范化根目录定界 picker）、fff-search crate（fuzzy.rs:5-14），单测 find_files_fuzzy_matches_name 覆盖模糊命中
   - src/tools/fuzzy.rs:222-236（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=4 · tool_calls=4 · duration=13219ms · tokens=22692

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
find_files（FileFinderTool, src/tools/fuzzy.rs:41-110）是恒注册的模糊文件名查找工具：CLI 装配期创建会话级 FuzzyEngine（封装 fff-search FilePicker，watch=false 同步收集索引，fuzzy.rs:23-37）并注册进 ToolRegistry（cli.rs:242-243），与 GrepTool 共享同一 Arc；运行期由 Harness 主循环经去重/熔断门控后分发 execute（harness.rs:331-362），内部 QueryParser 解析 query、limit clamp(1,100) 后调 picker.fuzzy_search（fuzzy.rs:86-92），输出 frecency 排序编号相对路径。无专属配置项/环境变量；query 缺失→USER_INPUT，引擎构建失败→INDEX_BUILD_FAILED 且致命（cli.rs:242 ? 传播）；工具错误与同参重复均计无进展熔断；fence 仅用于取 root 定界，不经 resolve；与 read/grep/vector_search/图工具存在显式降级与引导契约。

## 证据列表
1. 调用链装配期：run_task_inner 建 Fence→FuzzyEngine（致命 ?）→注册 FileFinderTool/GrepTool 共享同一 Arc<FuzzyEngine>
   - src/cli.rs:239-244（审计 #4）
   - src/tools/fuzzy.rs:17-37（审计 #2）
   - src/tools/mod.rs:24-59（审计 #12）
2. 运行期分发链：canonical 去重→参数校验→tools.get→审计 tool_call→tool.execute→tool_result 审计+evidence.observe+info_keys 增量计数；工具错误计无进展熔断
   - src/harness.rs:301-315（审计 #12）
   - src/harness.rs:331-362（审计 #12）
   - src/harness.rs:365-388（审计 #12）
   - src/harness.rs:404-407（审计 #12）
3. execute 数据流：query 必填非空（USER_INPUT）、limit unwrap_or(20).clamp(1,100)、QueryParser.parse、fuzzy_search(max_threads:0, pagination offset=0)、输出编号 relative_path + 截断提示；无翻页参数
   - src/tools/fuzzy.rs:73-108（审计 #2）
   - src/tools/fuzzy.rs:62-71（审计 #2）
4. 无专属配置/环境变量：FuzzySearchOptions 与 FilePickerOptions 全部硬编码（FFFMode::Ai, watch=false, max_threads=0）；依赖 fff-search 0.11.0（禁 zlob feature）
   - src/tools/fuzzy.rs:87-91（审计 #2）
   - src/tools/fuzzy.rs:23-37（审计 #2）
   - Cargo.toml:20（审计 #30）
   - src/tools/fuzzy.rs:3（审计 #2）
5. 错误语义：索引构建失败 INDEX_BUILD_FAILED 在装配期 ? 传播属致命（对比 codegraph 启动失败非致命降级）
   - src/cli.rs:242（审计 #4）
   - src/tools/fuzzy.rs:32-35（审计 #2）
   - src/cli.rs:263-271（审计 #4）
6. 围栏语义：fuzzy 层仅消费 Fence::root() 定界 picker，不经 Fence::resolve；resolve 失败 hint 官方推荐「用 find_files 先定位真实路径」
   - src/tools/fuzzy.rs:24-27（审计 #2）
   - src/fence.rs:30-47（审计 #14）
   - src/fence.rs:43-45（审计 #14）
7. 交互契约：图工具无符号索引时地形提示降级引导用 find_files（cli.rs:290-294）；vector_search 空召回引导文案「换关键词用 grep，或用 find_files」；审计落 ~/.codesleuth/
   - src/cli.rs:282-295（审计 #4）
   - docs/contexts/向量语义检索/02-vector-search.md:253-262（审计 #44）
   - src/cli.rs:227-229（审计 #4）
8. 测试覆盖：单测 find_files_fuzzy_matches_name（模糊命中）与 missing_args_are_tool_errors（空 args 报错）；无空白字符串 query 单测
   - src/tools/fuzzy.rs:222-258（审计 #2）

## 死胡同
- grep "fff-search|fff=" 无 Cargo.toml 直接命中（后改 read Cargo.toml 直接确认依赖），fff-search crate 内部 frecency/QueryParser 约束语法语义未读（外部 crate，标注推断）
- config.rs 未直接 read；「配置零触及 fuzzy」基于全仓 grep 'fff' 仅 4 文件命中且无一在 config.rs 的负结果（推断）

## 置信度
high

## 统计
turns=13 · tool_calls=16 · duration=142503ms · tokens=271094


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
find_files 是一个基于 fff-search 的模糊文件名查找工具（frecency 排序），输入 query（文件名/路径片段，支持约束语法）和可选 limit（默认 20，上限 100），帮助调用方按名字快速定位仓库内文件。入口是 FileFinderTool（src/tools/fuzzy.rs:41-110），CLI 启动时创建会话级 FuzzyEngine 并注册该工具（src/cli.rs:242-244）；执行时经 QueryParser 解析 query 后调用共享 FuzzyEngine 中 FilePicker.fuzzy_search 完成检索（src/tools/fuzzy.rs:86-92）。它与 GrepTool（内容检索）共享同一引擎、同属「读层+模糊层」检索工具面，上游依赖 crate::errors、crate::fence::Fence 与 fff-search crate；集成测试（tests/adversarial.rs、tests/replay.rs、tests/layered_live.rs）也将它与 GrepTool 注册在同一 ToolRegistry。

## 证据列表
1. FileFinderTool 是名为 find_files 的 Tool，按名字模糊找文件（frecency 排序），参数为 query（必填）与 limit（默认20，上限100）
   - src/tools/fuzzy.rs:40-71（审计 #2）
2. execute 解析 query/limit，QueryParser 解析后调用 engine.picker.fuzzy_search，输出编号路径列表，超限时提示可提高 limit
   - src/tools/fuzzy.rs:73-108（审计 #2）
3. CLI 启动时创建 FuzzyEngine 并把 FileFinderTool 与 GrepTool 注册进 ToolRegistry（层进 v0：读层+模糊层）
   - src/cli.rs:238-244（审计 #10）
4. FuzzyEngine 为会话级引擎，watch=false 同步收集文件，封装 fff-search FilePicker；FileFinderTool 与 GrepTool 共享同一 Arc<FuzzyEngine>（41-43、113-115）
   - src/tools/fuzzy.rs:17-37（审计 #2）
5. 上游依赖：crate::errors 错误码、crate::fence::Fence（取规范化根目录定界 picker）、fff-search crate（fuzzy.rs:5-14），单测 find_files_fuzzy_matches_name 覆盖模糊命中
   - src/tools/fuzzy.rs:222-236（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=4 · tool_calls=4 · duration=13219ms · tokens=22692
