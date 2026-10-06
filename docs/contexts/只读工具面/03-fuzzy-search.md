# 模糊搜索工具（只读工具面）

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
# 模糊搜索工具（只读工具面）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
对外暴露 `find_files`（frecency 模糊路径搜索）与 `grep`（plain/regex/fuzzy 三态内容检索）两个只读工具，共用 `FuzzyEngine` 持有的 `FilePicker` 会话级索引，由 `src/cli.rs` 装配进 `ToolRegistry` 后供 LLM 通过 tool_calls 调用。

## 证据列表
1. 模块顶部声明 fff-search 封装与 zlob 禁用策略
   - src/tools/fuzzy.rs:1-3（审计 #2）
2. FuzzyEngine 持有 FilePicker，new 阶段用 Fence 取根 + 同步 collect_files
   - src/tools/fuzzy.rs:18-38（审计 #2）
3. FileFinderTool 实现 Tool trait，name="find_files"，执行走 QueryParser + FuzzySearchOptions + picker.fuzzy_search
   - src/tools/fuzzy.rs:41-110（审计 #2）
4. GrepTool 实现 Tool trait，name="grep"，mode 映射 GrepMode::PlainText/Regex/Fuzzy，执行走 picker.grep
   - src/tools/fuzzy.rs:113-208（审计 #2）
5. fff-search crate 提供 FilePicker / QueryParser / FuzzySearchOptions / GrepSearchOptions / GrepMode
   - src/tools/fuzzy.rs:9-12（审计 #2）
6. cli.rs 在层进 v0 阶段构建 Arc<FuzzyEngine> 并把两个工具注册到 ToolRegistry
   - src/cli.rs:244-250（审计 #23）
7. system prompt 把 find_files / grep 列为契约锚点
   - src/prompt.rs:42-43（审计 #36）
8. Tool trait 定义于 tools/mod.rs，子模块 fuzzy 导出两个工具与引擎
   - src/tools/mod.rs:1-22（审计 #8）
9. replay 测试与 fixture 证明 find_files / grep 被 LLM 实际作为工具调用
   - tests/replay.rs:91（审计 #45）
   - tests/fixtures/replay/fixture-rs-replay.json:6-11（审计 #47）
10. 单测覆盖模糊命中、grep 内容命中、缺参报错三条路径
   - src/tools/fuzzy.rs:222-258（审计 #2）

## 死胡同
- grep "FuzzyEngine::new|FileFinderTool::new|GrepTool::new" 复合正则 0 命中（符号名带括号时索引切分失败）
- grep "fff_search" 仅 1 命中，依赖项声明位置已在 fuzzy.rs:9 直接定位，无需扩展
- explore 返回的 vector_search/recall.rs 内容与本任务（路径+内容检索）非同一关注面，未纳入引用

## 置信度
high

## 统计
turns=13 · tool_calls=17 · duration=45899ms · tokens=159561


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
模糊搜索工具对外暴露 find_files（frecency 模糊路径搜索）与 grep（plain/regex/fuzzy 三态内容检索）两个只读工具，由 src/tools/fuzzy.rs 实现的 FuzzyEngine（封装 fff-search 0.11 的 FilePicker 同步 collect_files）持有会话级索引，FileFinderTool/GrepTool 实现 Tool trait 后被 src/cli.rs 在「层进 v0」阶段连同 ReadTool、CodegraphEngine 等一起塞进同一个 ToolRegistry；Harness 主循环在每轮 LLM 响应里按 name 查表分发执行；system prompt 与 replay fixture 把这两件工具固化为契约锚点。

## 证据列表
1. FuzzyEngine 在 new 阶段用 Fence 取规范化根目录定界 picker（fence.rs 提供 canonicalize + symlink/越界防御），并同步 collect_files；picker 持有 watch=false + FFFMode::Ai，错误映射 CS4011 INDEX_BUILD_FAILED。
   - src/tools/fuzzy.rs:22-38（审计 #2）
   - src/fence.rs:11-25（审计 #16）
   - src/errors.rs:46-48（审计 #52）
2. FileFinderTool 用 QueryParser::default().parse(&query_str) + FuzzySearchOptions{max_threads:0, pagination: PaginationArgs{offset:0,limit}} 走 picker.fuzzy_search；输出形如 [find_files] "q" · 命中 matched/total，每行 i+1 + 相对路径，超出 limit 时追加「…（已截断，可提高 limit）」。
   - src/tools/fuzzy.rs:73-110（审计 #2）
3. GrepTool 三态映射：mode 字符串 → GrepMode::Regex/Fuzzy/PlainText（默认 plain，未知值亦走 plain）；GrepSearchOptions 内置 max_file_size=1_048_576、max_matches_per_file=10、smart_case=true、time_budget_ms=5000、enforce_time_budget=false、classify_definitions=true；输出形如 [grep] "pat" (mode) · files_matched/filtered/total_scanned，每行 file:line: content，超出 page_limit 时追加「…还有更多文件未扫（file_offset=N）」。
   - src/tools/fuzzy.rs:145-208（审计 #2）
4. find_files 默认 limit=20 clamp(1,100)；grep 默认 limit=50 clamp(1,200)；两者均要求 query/pattern 非空字符串（trim 后），缺失 → CsError::new(USER_INPUT, "…缺少 … 参数") 走 CS1001。
   - src/tools/fuzzy.rs:73-84（审计 #2）
   - src/tools/fuzzy.rs:145-162（审计 #2）
   - src/errors.rs:32-32（审计 #52）
5. Tool trait（D009 接口层防线）：name/description/parameters/execute 四个方法；ToolRegistry 持有 Vec<Box<dyn Tool>>，register/get/schemas/is_empty 五个方法，schemas() 转 Vec<ToolSchema> 供 ChatRequest.tools 使用。
   - src/tools/mod.rs:14-58（审计 #4）
   - src/llm.rs:45-72（审计 #30）
6. cli.rs「层进 v0」装配顺序：先建 Arc<Fence> 给 ReadTool，再建 Arc<FuzzyEngine> 复用同一份 picker 给 FileFinderTool + GrepTool，4 个工具全部塞进同一个 ToolRegistry；CodegraphEngine 与向量层在 v0 之后才可选挂载。
   - src/cli.rs:244-250（审计 #9）
   - src/cli.rs:359-371（审计 #9）
7. Harness 主循环把 schema 喂给 LLM、按 ToolCallSpec.name 在 registry 中查表执行；submit_report/recall 是 Harness 自带内置 schema（在 builtin_schemas 中追加，与外部 ToolRegistry 并列）；去重 + 零增量熔断通过 EvidenceStore.observe + info_keys 判定，find_files/grep 输出里的 path-like token 自动入证据库供 submit_report 校验。
   - src/harness.rs:126-135（审计 #30）
   - src/harness.rs:331-417（审计 #30）
   - src/evidence.rs:7-55（审计 #49）
8. system prompt 把 find_files/grep 与只读/拒绝/explore/callers/read/file:line/无新信息/submit_report/置信度/死胡同 一起列为契约锚点；prompt_contains_contract_anchors 测试断言每个锚点都必须出现在 SYSTEM_PROMPT 字符串中。
   - src/prompt.rs:37-56（审计 #14）
9. 初稿说 src/prompt.rs:42-43 是契约锚点处，实际是 src/prompt.rs:37-56 的测试体（SYSTEM_PROMPT 字符串本体在 prompt.rs:4 起）。初稿行号偏内，准确锚点应改为 prompt.rs:42-43 对应测试数组的 find_files/grep 元素、或 src/prompt.rs:4 起的 SYSTEM_PROMPT 本体。
   - src/prompt.rs:37-56（审计 #14）
   - src/prompt.rs:4-30（审计 #14）
10. replay fixture 录制 find_files(query=retry) → grep(pattern=retry_with_backoff) → read(path=src/retry.rs) → submit_report 的真实 LLM 响应序列；tests/replay.rs 断言工具调用顺序为 [find_files, grep, read, submit_report] 且 outcome.tool_calls=3、证据指向 src/retry.rs。
   - tests/fixtures/replay/fixture-rs-replay.json:1-29（审计 #21）
   - tests/replay.rs:85-106（审计 #19）
11. 只读对抗测试（tests/adversarial.rs）把 find_files/grep 列入 READ_ONLY_TOOLS 白名单，复用 build_registry 装配 ReadTool + FileFinderTool + GrepTool 三件套；fuzz 测试覆盖三个路径：模糊命中路径名、grep 命中具体行号、缺参报错。
   - tests/adversarial.rs:19-39（审计 #30）
   - src/tools/fuzzy.rs:222-258（审计 #2）
12. fff-search 0.11.0 是外部 crate 依赖（依赖项声明在 Cargo.toml），禁用 zlob feature（需要 zig）改走纯 Rust fallback，保证 cargo install 可移植；fff_search::FilePicker/QueryParser/FuzzySearchOptions/GrepSearchOptions/GrepMode/FFFMode/PaginationArgs/FilePickerOptions 由 src/tools/fuzzy.rs:9-12 引入。
   - Cargo.toml:20-20（审计 #47）
   - src/tools/fuzzy.rs:1-12（审计 #2）

## 死胡同
- grep 复合正则「FuzzyEngine::new|FileFinderTool::new|GrepTool::new」索引返回 0 命中（符号名带括号或 :: 时被切分失败），已改用 explore/读源码确认；真实引用全部位于 src/tools/fuzzy.rs:23/46/118 与 src/cli.rs:248-250、tests/adversarial.rs:35-37。
- grep 「fff_search|fff-search」0 命中——fff-search 仅作外部 crate 依赖（Cargo.toml:20），无源码字符串引用。
- vector_search/recall.rs explore 内容与本任务非同一关注面，未纳入引用。

## 置信度
high

## 统计
turns=11 · tool_calls=25 · duration=64885ms · tokens=253900


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
对外暴露 `find_files`（frecency 模糊路径搜索）与 `grep`（plain/regex/fuzzy 三态内容检索）两个只读工具，共用 `FuzzyEngine` 持有的 `FilePicker` 会话级索引，由 `src/cli.rs` 装配进 `ToolRegistry` 后供 LLM 通过 tool_calls 调用。

## 证据列表
1. 模块顶部声明 fff-search 封装与 zlob 禁用策略
   - src/tools/fuzzy.rs:1-3（审计 #2）
2. FuzzyEngine 持有 FilePicker，new 阶段用 Fence 取根 + 同步 collect_files
   - src/tools/fuzzy.rs:18-38（审计 #2）
3. FileFinderTool 实现 Tool trait，name="find_files"，执行走 QueryParser + FuzzySearchOptions + picker.fuzzy_search
   - src/tools/fuzzy.rs:41-110（审计 #2）
4. GrepTool 实现 Tool trait，name="grep"，mode 映射 GrepMode::PlainText/Regex/Fuzzy，执行走 picker.grep
   - src/tools/fuzzy.rs:113-208（审计 #2）
5. fff-search crate 提供 FilePicker / QueryParser / FuzzySearchOptions / GrepSearchOptions / GrepMode
   - src/tools/fuzzy.rs:9-12（审计 #2）
6. cli.rs 在层进 v0 阶段构建 Arc<FuzzyEngine> 并把两个工具注册到 ToolRegistry
   - src/cli.rs:244-250（审计 #23）
7. system prompt 把 find_files / grep 列为契约锚点
   - src/prompt.rs:42-43（审计 #36）
8. Tool trait 定义于 tools/mod.rs，子模块 fuzzy 导出两个工具与引擎
   - src/tools/mod.rs:1-22（审计 #8）
9. replay 测试与 fixture 证明 find_files / grep 被 LLM 实际作为工具调用
   - tests/replay.rs:91（审计 #45）
   - tests/fixtures/replay/fixture-rs-replay.json:6-11（审计 #47）
10. 单测覆盖模糊命中、grep 内容命中、缺参报错三条路径
   - src/tools/fuzzy.rs:222-258（审计 #2）

## 死胡同
- grep "FuzzyEngine::new|FileFinderTool::new|GrepTool::new" 复合正则 0 命中（符号名带括号时索引切分失败）
- grep "fff_search" 仅 1 命中，依赖项声明位置已在 fuzzy.rs:9 直接定位，无需扩展
- explore 返回的 vector_search/recall.rs 内容与本任务（路径+内容检索）非同一关注面，未纳入引用

## 置信度
high

## 统计
turns=13 · tool_calls=17 · duration=45899ms · tokens=159561
