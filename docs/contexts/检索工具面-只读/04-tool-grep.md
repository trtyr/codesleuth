# grep 内容检索（检索工具面（只读））

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`04-tool-grep-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# grep 内容检索（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
grep 是仓库内内容检索工具，支持 plain（默认）/regex/fuzzy 三种匹配模式，供调用方以 {pattern, mode?, limit?} 在仓库范围内拿到 "file:line: content" 形式的命中结果，是层进检索协议中「顺藤摸点」的定位手段。

## 证据列表
1. 功能价值：grep 在仓库内做内容检索，支持 plain/regex/fuzzy 三态模式，返回 file:line:content 命中，是层进检索协议的定位手段（fuzzy.rs 模块注释明确 find_files/grep 的分工）。
   - src/tools/fuzzy.rs:112-131（审计 #2）
2. 入口与关键文件：GrepTool 定义于 src/tools/fuzzy.rs:113（impl Tool name="grep"，execute 在 145-207）；CLI 启动时在 src/cli.rs:242-244 创建 FuzzyEngine 并注册 GrepTool；单元测试 grep_plain_finds_lines 位于 src/tools/fuzzy.rs:238-248。
   - src/tools/fuzzy.rs:124-143（审计 #2）
   - src/tools/fuzzy.rs:145-181（审计 #2）
   - src/cli.rs:242-244（审计 #10）
   - src/tools/fuzzy.rs:17-37（审计 #2）
   - src/tools/fuzzy.rs:153-157（审计 #2）
   - src/tools/fuzzy.rs:164-181（审计 #2）
   - src/tools/fuzzy.rs:238-248（审计 #2）
3. 运作方式：调用方传 {pattern, mode?, limit?} → GrepTool::execute 将 mode 字符串映射为 fff_search 的 GrepMode（fuzzy.rs:153-157），构建 GrepSearchOptions（smart_case、limit clamp 1-200 等）后调用共享 FuzzyEngine 内 FilePicker.grep（fuzzy.rs:181），再格式化为 "file:line: content" 输出（fuzzy.rs:183-206）。
   - src/tools/fuzzy.rs:145-207（审计 #2）
   - src/cli.rs:242-244（审计 #10）
4. 模块交互：上游依赖 fff_search crate（FilePicker/GrepMode/GrepSearchOptions/QueryParser，fuzzy.rs:9-12）、crate::errors 错误码与 crate::fence::Fence（fuzzy.rs:5-6，FuzzyEngine::new 用 Fence 取规范化根目录，fuzzy.rs:23-27）；与 ReadTool、FileFinderTool 共享同一 Arc<FuzzyEngine> 注册进 ToolRegistry（cli.rs:241-244）；下游被 tests/adversarial.rs:37、tests/replay.rs:69、tests/layered_live.rs:33 注册消费，零命中降级由 fff 处理（fuzzy.rs:112）。
   - src/tools/fuzzy.rs:9-15（审计 #2）
   - src/tools/fuzzy.rs:5-7（审计 #2）
   - src/tools/fuzzy.rs:114（审计 #2）
   - src/cli.rs:238-244（审计 #10）
   - src/tools/fuzzy.rs:187-193（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=6 · duration=16744ms · tokens=28399

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
grep 工具的完整链路：LLM Agent 在 Harness 主循环发出工具调用 → Harness 从 ToolRegistry 按名取 GrepTool → GrepTool::execute 解析 {pattern, mode?, limit?}，mode 字符串映射为 fff_search::GrepMode，构建 GrepSearchOptions（smart_case、limit clamp 1-200、单文件上限 1MB/10 命中、5s time_budget 但未启用强制）→ 调共享 FuzzyEngine 内 fff-search FilePicker.grep 同步检索 → 格式化为 "file:line: content" 文本返回，经 audit 记账与 evidence 观察后回灌 LLM。初稿结论基本正确，本次补充了 Harness 分发层、审计/证据记账、硬编码参数细节与误用边界（mode 非法值静默落 plain、grep 不经 fence.resolve 只受 picker 根目录定界）。

## 证据列表
1. 调用链第 1 段（分发）：Harness::run 主循环收到 LLM tool_calls，先 parse_args 校验 JSON（harness.rs:318），再 self.tools.get(&call.name) 按名在 ToolRegistry（Vec 线性查找，取先注册者）中查找 GrepTool；未知工具则 reject_call 并提示可用工具列表（harness.rs:331-347）。
   - src/harness.rs:331-347（审计 #29）
   - src/tools/mod.rs:38-43（审计 #15）
2. 调用链第 2 段（记账+执行）：执行前 audit.record('tool_call', {turn,name,args}) 取 seq（harness.rs:350-353），随后 tool.execute(args).await（harness.rs:362），成功后 record('tool_result', {tool_call_seq, name, output}）并 evidence.observe(output, seq) 记账（harness.rs:365-378）；输出还经 info_keys 做零增量检测驱动熔断/转向（harness.rs:379-388）。
   - src/harness.rs:349-392（审计 #29）
3. 装配：CLI 启动时（层进 v0 读层+模糊层注释）Fence::new(&repo_abs) → FuzzyEngine::new(&repo_abs)（内部经 Fence 取规范化根目录定界 picker，watch=false 同步 collect_files，索引失败报 INDEX_BUILD_FAILED）→ GrepTool::new(engine) 注册进 ToolRegistry（cli.rs:238-244）；GrepTool 仅持 Arc<FuzzyEngine>（fuzzy.rs:112-121）。
   - src/cli.rs:238-244（审计 #4）
   - src/tools/fuzzy.rs:22-37（审计 #2）
   - src/tools/fuzzy.rs:112-121（审计 #2）
4. 输入解析：execute 取 pattern（缺失/空白报 USER_INPUT 错），mode 缺省 'plain'，'regex'→GrepMode::Regex、'fuzzy'→GrepMode::Fuzzy、其余一律静默落 GrepMode::PlainText（非法 mode 不报错）；limit 缺省 50，clamp(1,200)。
   - src/tools/fuzzy.rs:145-157（审计 #2）
5. 检索参数（全部硬编码，无配置项）：QueryParser::default().parse(pattern) 后构建 GrepSearchOptions：max_file_size=1_048_576、max_matches_per_file=10、smart_case=true、page_limit=limit、time_budget_ms=5000 但 enforce_time_budget=false、before/after_context=0、classify_definitions=true，调 picker.grep（fuzzy.rs:164-181）。
   - src/tools/fuzzy.rs:164-181（审计 #2）
6. 输出落点：格式化头行 '[grep] pattern (mode) · 命中文件数/过滤文件数 · 已扫描文件数'，逐条 'rel_path:line_number: line_content'（relative_path 相对 picker 根），next_file_offset>0 时附 '…还有更多文件未扫（file_offset=N）' 分页提示（fuzzy.rs:183-206）。
   - src/tools/fuzzy.rs:183-206（审计 #2）
   - src/tools/fuzzy.rs:200-204（审计 #2）
7. 上游依赖与数据源：fff-search crate 0.11.0（Cargo.toml:20，其子 crate fff-grep/fff-query-parser/fff-notify-debouncer-full，Cargo.lock:811-853）；数据流输入来自 LLM 工具调用 JSON args，扫描对象为 repo_abs 根下 fff 收集的文件集，输出纯文本回灌 LLM 上下文并落审计 JSONL（~/.codesleuth/audit/{session_id}.jsonl，cli.rs:225-229）。grep 结果不落任何仓库内文件（只读）。
   - Cargo.toml:14-31（审计 #35）
   - src/tools/fuzzy.rs:9-15（审计 #2）
   - src/cli.rs:225-236（审计 #4）
8. 配置与开关：grep 本身零配置项、零环境变量，行为全由硬编码 GrepSearchOptions 决定；间接受配置影响的是 repo 根（--repo canonicalize 后供 Fence 定界，无开关）与 fff-search 版本锁定；与 ReadTool 不同，grep/fuzzy 层不经 fence.resolve 路径校验，仅由 picker base_path 定界（docs/contexts/只读边界与安全/01-path-fence.md:268）。
   - src/tools/fuzzy.rs:145-206（审计 #2）
   - src/tools/fuzzy.rs:5-7（审计 #2）
   - docs/contexts/只读边界与安全/01-path-fence.md:268（审计 #43）
9. 边界与坑：①mode 传非法值（如 'Plain' 或带空格）静默按 plain 处理（fuzzy.rs:153-157）；②enforce_time_budget=false，5s time_budget 实际不强制，大仓可能慢（fuzzy.rs:173-174）；③单文件 >1MB 跳过、单文件最多 10 命中，limit 只是全局截断（fuzzy.rs:166-167）；④before/after_context=0 拿不到上下文行，需跟 read；⑤zero-gain 输出会累积 no_progress 触发熔断/转向，同 pattern 重复 grep 有代价（harness.rs:384-388）；⑥ToolRegistry.get 取先注册者且无重名保护（tools/mod.rs:38-43）；⑦缺 pattern 是工具错误（测试 missing_args_are_tool_errors，fuzzy.rs:251-258）；⑧正确用例测试 grep_plain_finds_lines（fuzzy.rs:238-248）。
   - src/tools/fuzzy.rs:153-157（审计 #2）
   - src/tools/fuzzy.rs:166-176（审计 #2）
   - src/tools/fuzzy.rs:251-258（审计 #2）
   - src/tools/fuzzy.rs:238-248（审计 #2）
   - src/harness.rs:384-388（审计 #29）
   - src/tools/mod.rs:38-43（审计 #15）
10. 交互契约：与 FileFinderTool 共享同一 Arc<FuzzyEngine>（cli.rs:242-244），FilePicker 集合只建一次；工具面经 ToolRegistry::schemas() 注入 LLM（tools/mod.rs:45-54），find_files 定位文件、grep 定位内容、read 强读的三层递进契约；输出经 evidence.observe 记 path→seq 账，供 submit_report 证据校验回溯（harness.rs:369-378）。
   - src/cli.rs:238-244（审计 #4）
   - src/harness.rs:350-378（审计 #29）
   - src/tools/mod.rs:45-54（审计 #15）
   - src/tools/fuzzy.rs:40-54（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=14 · tool_calls=15 · duration=72606ms · tokens=248189


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
grep 是仓库内内容检索工具，支持 plain（默认）/regex/fuzzy 三种匹配模式，供调用方以 {pattern, mode?, limit?} 在仓库范围内拿到 "file:line: content" 形式的命中结果，是层进检索协议中「顺藤摸点」的定位手段。

## 证据列表
1. 功能价值：grep 在仓库内做内容检索，支持 plain/regex/fuzzy 三态模式，返回 file:line:content 命中，是层进检索协议的定位手段（fuzzy.rs 模块注释明确 find_files/grep 的分工）。
   - src/tools/fuzzy.rs:112-131（审计 #2）
2. 入口与关键文件：GrepTool 定义于 src/tools/fuzzy.rs:113（impl Tool name="grep"，execute 在 145-207）；CLI 启动时在 src/cli.rs:242-244 创建 FuzzyEngine 并注册 GrepTool；单元测试 grep_plain_finds_lines 位于 src/tools/fuzzy.rs:238-248。
   - src/tools/fuzzy.rs:124-143（审计 #2）
   - src/tools/fuzzy.rs:145-181（审计 #2）
   - src/cli.rs:242-244（审计 #10）
   - src/tools/fuzzy.rs:17-37（审计 #2）
   - src/tools/fuzzy.rs:153-157（审计 #2）
   - src/tools/fuzzy.rs:164-181（审计 #2）
   - src/tools/fuzzy.rs:238-248（审计 #2）
3. 运作方式：调用方传 {pattern, mode?, limit?} → GrepTool::execute 将 mode 字符串映射为 fff_search 的 GrepMode（fuzzy.rs:153-157），构建 GrepSearchOptions（smart_case、limit clamp 1-200 等）后调用共享 FuzzyEngine 内 FilePicker.grep（fuzzy.rs:181），再格式化为 "file:line: content" 输出（fuzzy.rs:183-206）。
   - src/tools/fuzzy.rs:145-207（审计 #2）
   - src/cli.rs:242-244（审计 #10）
4. 模块交互：上游依赖 fff_search crate（FilePicker/GrepMode/GrepSearchOptions/QueryParser，fuzzy.rs:9-12）、crate::errors 错误码与 crate::fence::Fence（fuzzy.rs:5-6，FuzzyEngine::new 用 Fence 取规范化根目录，fuzzy.rs:23-27）；与 ReadTool、FileFinderTool 共享同一 Arc<FuzzyEngine> 注册进 ToolRegistry（cli.rs:241-244）；下游被 tests/adversarial.rs:37、tests/replay.rs:69、tests/layered_live.rs:33 注册消费，零命中降级由 fff 处理（fuzzy.rs:112）。
   - src/tools/fuzzy.rs:9-15（审计 #2）
   - src/tools/fuzzy.rs:5-7（审计 #2）
   - src/tools/fuzzy.rs:114（审计 #2）
   - src/cli.rs:238-244（审计 #10）
   - src/tools/fuzzy.rs:187-193（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=6 · duration=16744ms · tokens=28399
