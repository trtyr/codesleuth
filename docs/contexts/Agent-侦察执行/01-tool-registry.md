# 工具注册表（Agent 侦察执行）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`01-tool-registry-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 工具注册表（Agent 侦察执行）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

工具注册表（ToolRegistry）是 Agent 侦察执行域的只读工具中枢：它把九个只读检索/结构工具以统一 trait 集中注册、按名查找，并向 LLM 输出 JSON Schema 工具面，是「接口上就不存在写能力」的 D009 只读边界防线。入口为 src/tools/mod.rs 的 Tool trait 与 ToolRegistry；src/cli.rs 装配时按层注册 read/fuzzy/graph/vector_search 共九个工具，Harness 运行期经 schemas() 注入工具列表、经 get() 分发执行；下游与 LLM schema、fence 只读围栏、FuzzyEngine、codegraph MCP、RecallEngine 及审计模块交互。

## 证据列表

1. ToolRegistry 定位为只读边界防线，注释明确「接口上就不存在写能力」
    - src/tools/mod.rs:1-2（审计 #2）
2. 统一抽象：Tool trait（name/description/parameters/只读 execute）+ ToolRegistry（Vec<Box<dyn Tool>>）
    - src/tools/mod.rs:14-27（审计 #2）
3. 核心 API：register 注册、get 按名查找、schemas 输出 Vec<ToolSchema> 给 LLM
    - src/tools/mod.rs:34-54（审计 #2）
4. 装配：cli.rs 注册 read/fuzzy 两层 3 个工具 + codegraph 就绪且有符号时注册 5 个图工具 + 向量层注册 vector_search，共九个
    - src/cli.rs:240-244（审计 #18）
    - src/cli.rs:282-288（审计 #18）
    - src/cli.rs:597-599（审计 #18）
5. 诚实工具面：codegraph 无符号时图工具不注册，改为注入地形提示
    - src/cli.rs:273-292（审计 #18）
6. 运行期消费：Harness::new 持有 ToolRegistry，tool_names 用 schemas() 生成工具名列表注入提示
    - src/harness.rs:46-57（审计 #16）
    - src/harness.rs:631-635（审计 #16）
7. 上游依赖：工具实现依赖 Fence 只读围栏、FuzzyEngine、CodegraphEngine（codegraph MCP）
    - src/cli.rs:239（审计 #18）
    - src/cli.rs:242（审计 #18）
    - src/cli.rs:252-253（审计 #18）
8. vector_search 工具依赖 vector::RecallEngine（嵌入+召回存储）
    - src/cli.rs:596-599（审计 #18）

## 死胡同

无

## 置信度

high

## 统计

turns=12 · tool_calls=14 · duration=30948ms · tokens=67065

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
ToolRegistry 是只读工具中枢：cli.rs 装配期按条件注册最多九个工具（read/find_files/grep 恒注册；explore/callers/callees/impact/files 需 codegraph 就绪且索引非空；vector_search 需 --vector 且向量索引可用），所有权移交 Harness 后每回合经 schemas() 注入 LLM 工具面、经 get(name) 分发 execute；submit_report/recall 是 harness 内置工具不进注册表。运行期有 canonical 去重、5 步无进展熔断、证据校验拒绝；配置经「CLI>项目>全局>默认」四层链（env 层已移除），--vector/--fresh-index/graph.bin/向量配置直接影响工具面构成。发现初稿遗漏：内置工具与注册表的边界、adversarial 测试只读清单缺 vector_search。

## 证据列表
1. 统一抽象：Tool trait（name/description/parameters/只读 execute）+ ToolRegistry（Vec 线性查找，get 取先注册者，无重名保护）
   - src/tools/mod.rs:14-59（审计 #2）
2. 装配链：Fence→ReadTool、FuzzyEngine→FileFinder/Grep 恒注册；图工具 5 个条件注册；vector_search 由 setup_vector_layer 注册
   - src/cli.rs:239-244（审计 #4）
   - src/cli.rs:282-288（审计 #4）
   - src/cli.rs:596-599（审计 #4）
3. 诚实工具面：codegraph 无符号→图工具不注册+地形提示；启动失败降级留痕，INDEX_LOCKED 判负退出
   - src/cli.rs:273-295（审计 #4）
   - src/cli.rs:259-271（审计 #4）
4. 运行期分发：schemas()+builtin_schemas() 注入工具面；submit_report/recall 为内置不进注册表；get→execute→审计+EvidenceStore
   - src/harness.rs:126-134（审计 #12）
   - src/harness.rs:222-417（审计 #12）
   - src/harness.rs:423-458（审计 #12）
5. 熔断/去重：canonical 同参去重、5 步无进展熔断（CS2099）、2 回合零增量转向、工具错误也计熔断步
   - src/harness.rs:18-21（审计 #12）
   - src/harness.rs:617-629（审计 #12）
   - src/harness.rs:638-653（审计 #12）
6. 证据校验拒绝：引用未读文件、零 findings 有读取无 dead_ends、finding 无证据均打回
   - src/harness.rs:504-578（审计 #12）
7. 配置链：CLI>项目(.codesleuth/config.toml)>全局(~/.codesleuth/config.toml)>默认，env 层已移除；--vector/--repo-map/--fresh-index/graph.bin 影响工具面
   - src/config.rs:1-3（审计 #30）
   - src/config.rs:173-200（审计 #30）
   - src/cli.rs:44-49（审计 #4）
8. read 工具默认 limit=200 上限 2000；二进制探测不倾倒、空/越界诚实反馈
   - src/tools/read.rs:12-13（审计 #20）
   - src/tools/read.rs:66-118（审计 #20）
9. Fence 双层路径校验（词汇预检+canonicalize 解 symlink 复检），越界 CS3003 且不泄露围栏外存在性
   - src/fence.rs:28-55（审计 #42）
10. 初稿未提及：adversarial 只读允许清单仅 8 工具不含 vector_search（推断：该测试装配只建 3 工具故未覆盖；验证：跑断言①）
   - tests/adversarial.rs:20-29（审计 #47）
   - tests/adversarial.rs:31-39（审计 #47）
11. 零写入自证：writeguard 考前快照+考后 diff，不可归因变更=完整性违规 ERROR
   - src/cli.rs:369-407（审计 #4）

## 死胡同
无

## 置信度
high

## 统计
turns=11 · tool_calls=19 · duration=134444ms · tokens=228001


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

工具注册表（ToolRegistry）是 Agent 侦察执行域的只读工具中枢：它把九个只读检索/结构工具以统一 trait 集中注册、按名查找，并向 LLM 输出 JSON Schema 工具面，是「接口上就不存在写能力」的 D009 只读边界防线。入口为 src/tools/mod.rs 的 Tool trait 与 ToolRegistry；src/cli.rs 装配时按层注册 read/fuzzy/graph/vector_search 共九个工具，Harness 运行期经 schemas() 注入工具列表、经 get() 分发执行；下游与 LLM schema、fence 只读围栏、FuzzyEngine、codegraph MCP、RecallEngine 及审计模块交互。

## 证据列表

1. ToolRegistry 定位为只读边界防线，注释明确「接口上就不存在写能力」
    - src/tools/mod.rs:1-2（审计 #2）
2. 统一抽象：Tool trait（name/description/parameters/只读 execute）+ ToolRegistry（Vec<Box<dyn Tool>>）
    - src/tools/mod.rs:14-27（审计 #2）
3. 核心 API：register 注册、get 按名查找、schemas 输出 Vec<ToolSchema> 给 LLM
    - src/tools/mod.rs:34-54（审计 #2）
4. 装配：cli.rs 注册 read/fuzzy 两层 3 个工具 + codegraph 就绪且有符号时注册 5 个图工具 + 向量层注册 vector_search，共九个
    - src/cli.rs:240-244（审计 #18）
    - src/cli.rs:282-288（审计 #18）
    - src/cli.rs:597-599（审计 #18）
5. 诚实工具面：codegraph 无符号时图工具不注册，改为注入地形提示
    - src/cli.rs:273-292（审计 #18）
6. 运行期消费：Harness::new 持有 ToolRegistry，tool_names 用 schemas() 生成工具名列表注入提示
    - src/harness.rs:46-57（审计 #16）
    - src/harness.rs:631-635（审计 #16）
7. 上游依赖：工具实现依赖 Fence 只读围栏、FuzzyEngine、CodegraphEngine（codegraph MCP）
    - src/cli.rs:239（审计 #18）
    - src/cli.rs:242（审计 #18）
    - src/cli.rs:252-253（审计 #18）
8. vector_search 工具依赖 vector::RecallEngine（嵌入+召回存储）
    - src/cli.rs:596-599（审计 #18）

## 死胡同

无

## 置信度

high

## 统计

turns=12 · tool_calls=14 · duration=30948ms · tokens=67065
