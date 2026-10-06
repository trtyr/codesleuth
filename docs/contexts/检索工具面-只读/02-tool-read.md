# read 强读工具（检索工具面（只读））

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`02-tool-read-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# read 强读工具（检索工具面（只读））

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
read 强读工具是只读检索工具面中"唯一事实来源"的文件读取器：给调用方（LLM Agent）返回带行号+行哈希锚点的文件内容，支持 offset/limit 分页，并对二进制、空文件、越界、非 UTF-8 等情况做诚实反馈，防止读错内容。

## 证据列表
1. 功能定位：强读工具，MVP 六特性（锚点/分页/二进制探测/编码消毒/诚实反馈/结构化输出头），args 为 path + offset(1-based) + limit(默认200，上限2000)。
   - src/tools/read.rs:1-2, 50-52, 54-64（审计 #2）
   - src/tools/read.rs:25-29, 12-13（审计 #2）
2. 入口与关键文件：src/tools/read.rs（ReadTool 结构体、Tool trait 实现、execute 入口），由 src/cli.rs 启动时注册进 ToolRegistry；tests/adversarial.rs、tests/replay.rs 等测试也注册它。
   - src/tools/read.rs:15-22, 44-48, 66（审计 #2）
   - src/cli.rs:238-244（审计 #8）
   - tests/adversarial.rs:32-36（审计 #13）
   - tests/replay.rs:64-68（审计 #15）
3. 运作方式：CLI/Agent 调用注册表中的 read 工具 → ReadTool::execute 解析参数 → 经 Fence resolve 路径并读文件 → 逐行输出「行号:3位hex行哈希|内容」，截断时提示续读 offset；测试验证锚点+分页行为。
   - src/tools/read.rs:66-146（审计 #2）
   - src/tools/read.rs:86, 93（审计 #2）
   - src/tools/read.rs:158-172（审计 #2）
4. 交互模块：上游依赖 crate::errors（USER_INPUT/REPO_NOT_READABLE 错误码）与 crate::fence::Fence（路径安全解析）；自身实现 tools::Tool trait，与 fuzzy 层 FileFinderTool/GrepTool 同注册表构成检索工具面；内建 sha2 行哈希、二进制启发探测、UTF-8 lossy 消毒。
   - src/tools/read.rs:4-6, 16, 86（审计 #2）
   - src/tools/read.rs:32-42, 96-107（审计 #2）
   - src/cli.rs:240-244（审计 #8）

## 死胡同
- 无；仅做概览，未展开 read.rs 测试细节（201-228 行）

## 置信度
high

## 统计
turns=6 · tool_calls=5 · duration=24463ms · tokens=34394

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
read 强读工具是 codesleuth 只读检索工具面中"唯一事实来源"的文件读取器（src/tools/read.rs，228 行）：LLM Agent 通过 Harness 主循环分发调用，ReadTool::execute 解析 path/offset/limit → 经 Fence 双层路径围栏（词汇预检 + canonicalize 解 symlink）→ std::fs::read 读文件 → 二进制探测（8KB 头部 NUL/控制字符>10% 拒绝倾倒）→ UTF-8 lossy 消毒 → 输出「[read] 头 + 行号:3位hex行哈希|内容」+ 截断续读提示。无任何配置项/环境变量，恒注册、恒开启。初稿结论全部与代码一致、无事实错误；本报告补全完整调用链（cli.rs 装配 → harness.rs 分发 → fence.rs → read.rs execute）、下游记账交互与边界细节，并补充初稿未提的 harness 对 read 的 read_exact_path 特殊分支。另注意 fence 文档指出：越界 FENCE_DENIED 与不存在 REPO_NOT_FOUND 退出码同为 4，仅凭 exit code 不可区分。

## 证据列表
1. 功能定位：MVP 六特性（锚点/分页/二进制探测/编码消毒/诚实反馈/结构化输出头）；参数契约 path(必填)/offset(1-based 默认1)/limit(默认200 上限2000)，常量 DEFAULT_LIMIT=200、MAX_LIMIT=2000
   - src/tools/read.rs:1-2（审计 #2）
   - src/tools/read.rs:12-13（审计 #2）
   - src/tools/read.rs:54-64（审计 #2）
2. 统一抽象：ReadTool 实现 tools::Tool trait（name/description/parameters/只读 execute），trait 定义于 src/tools/mod.rs:14-22
   - src/tools/mod.rs:14-22（审计 #20）
   - src/tools/read.rs:44-52（审计 #2）
3. 装配链：cli.rs run_task_inner 以 canonicalize 后的 repo_abs 建 Fence（cli.rs:239），ReadTool::new(fence.clone()) 恒注册为第一个工具（cli.rs:241）；FileFinderTool/GrepTool 紧随其后（cli.rs:242-244）；测试同构装配于 tests/adversarial.rs:34、tests/replay.rs:66、tests/layered_live.rs:30
   - src/cli.rs:238-244（审计 #4）
   - tests/adversarial.rs:31-38（审计 #36）
   - tests/replay.rs:64-69（审计 #41）
   - tests/layered_live.rs:28-33（审计 #43）
4. 运行期分发：Harness 每回合把 registry.schemas() 注入 ChatRequest 工具面（harness.rs:126-134），LLM 回 tool_calls 后经 tools.get(&call.name) 线性查找（harness.rs:331，mod.rs:38-43），先写审计 tool_call（harness.rs:350-353）再 tool.execute(args)（harness.rs:362）
   - src/harness.rs:126-136（审计 #31）
   - src/tools/mod.rs:38-54（审计 #20）
   - src/harness.rs:331（审计 #31）
   - src/harness.rs:350-362（审计 #31）
5. execute 数据流①参数解析：缺 path 报 USER_INPUT + hint 示例（read.rs:70-73）；offset unwrap_or(1).max(1)；limit unwrap_or(200).clamp(1,2000)（read.rs:75-84）
   - src/tools/read.rs:66-84（审计 #2）
6. execute 数据流②路径围栏：read.rs:86 调 fence.resolve；双层防御——词汇预检（normalize 折叠 ./.. + starts_with_root，fence.rs:37-42；Windows 忽略大小写 fence.rs:73-80）→ canonicalize 解 symlink 复检（fence.rs:43-53）；越界 FENCE_DENIED，不存在 REPO_NOT_FOUND
   - src/tools/read.rs:86（审计 #2）
   - src/fence.rs:30-55（审计 #15）
   - src/fence.rs:58-71（审计 #15）
   - src/fence.rs:73-80（审计 #15）
7. execute 数据流③读取：非普通文件（目录）报 REPO_NOT_READABLE（read.rs:87-92）；std::fs::read 同步读，失败包 REPO_NOT_READABLE（read.rs:93-94）
   - src/tools/read.rs:87-94（审计 #2）
8. execute 数据流④二进制探测：looks_binary 检查 8KB 头部（read.rs:97 min(8192)），NUL 字节或控制字符占比>10% 即拒绝倾倒，只返回字节数提示（read.rs:96-103；启发式 read.rs:31-42）
   - src/tools/read.rs:31-42（审计 #2）
   - src/tools/read.rs:96-103（审计 #2）
9. execute 数据流⑤消毒与诚实反馈：String::from_utf8_lossy，含 U+FFFD 时在输出头声明；空文件（0 行）与 offset>total 越界均返回诚实提示而非报错（read.rs:105-118）
   - src/tools/read.rs:105-118（审计 #2）
10. execute 数据流⑥输出格式：头部「[read] path · 共 N 行 · 显示 X-Y 行[· 含非 UTF-8 字节…]」，逐行「行号:3位hex行哈希|内容」——line_hash 为 SHA-256 取 12-bit（read.rs:25-29）；截断时追加「…未完：还有 N 行。续读 offset=X」（read.rs:138-144）
   - src/tools/read.rs:120-146（审计 #2）
   - src/tools/read.rs:25-29（审计 #2）
11. 特殊分支：harness 在 execute 前对 call.name=="read" 摘出精确 path（read_exact_path），成功后 evidence.observe_exact 存证（FINDING-010）；所有工具输出走 evidence.observe + info_keys 信息增量记账，零增益计 no_progress/zero_gain_streak 熔断（harness.rs:379-388）
   - src/harness.rs:356-378（审计 #31）
   - src/harness.rs:379-388（审计 #31）
12. 测试覆盖五分支：锚点+分页+续读 offset（read.rs:158-172）、二进制拒倒（175-184）、lossy UTF-8 声明（186-198）、空文件与越界诚实反馈（200-217）、缺 path 报 USER_INPUT（219-227）
   - src/tools/read.rs:149-228（审计 #2）
   - src/tools/read.rs:158-172（审计 #2）
13. 配置与开关：read 工具与 Fence 围栏均恒开启，零配置项、零环境变量（path-fence.md:246）；config::load 的 base_url/model/thinking_disabled 仅影响 LLM 层（cli.rs:212-236），不触及 read。已知坑：越界 FENCE_DENIED 与不存在 REPO_NOT_FOUND 退出码同为 4，仅凭 exit code 不可区分（path-fence.md:246）
   - docs/contexts/只读边界与安全/01-path-fence.md:246（审计 #45）
   - src/cli.rs:216-235（审计 #4）

## 死胡同
- grep "read::ReadTool|tools::read" 用 plain 模式 0 命中（正则语义被 plain 吃掉），改 regex 后命中
- 本任务未使用 explore：调用链已有 docs/contexts 文档 + 源码 grep 直达，read 原文验证更高效

## 置信度
high

## 统计
turns=12 · tool_calls=17 · duration=69764ms · tokens=191802


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
read 强读工具是只读检索工具面中"唯一事实来源"的文件读取器：给调用方（LLM Agent）返回带行号+行哈希锚点的文件内容，支持 offset/limit 分页，并对二进制、空文件、越界、非 UTF-8 等情况做诚实反馈，防止读错内容。

## 证据列表
1. 功能定位：强读工具，MVP 六特性（锚点/分页/二进制探测/编码消毒/诚实反馈/结构化输出头），args 为 path + offset(1-based) + limit(默认200，上限2000)。
   - src/tools/read.rs:1-2, 50-52, 54-64（审计 #2）
   - src/tools/read.rs:25-29, 12-13（审计 #2）
2. 入口与关键文件：src/tools/read.rs（ReadTool 结构体、Tool trait 实现、execute 入口），由 src/cli.rs 启动时注册进 ToolRegistry；tests/adversarial.rs、tests/replay.rs 等测试也注册它。
   - src/tools/read.rs:15-22, 44-48, 66（审计 #2）
   - src/cli.rs:238-244（审计 #8）
   - tests/adversarial.rs:32-36（审计 #13）
   - tests/replay.rs:64-68（审计 #15）
3. 运作方式：CLI/Agent 调用注册表中的 read 工具 → ReadTool::execute 解析参数 → 经 Fence resolve 路径并读文件 → 逐行输出「行号:3位hex行哈希|内容」，截断时提示续读 offset；测试验证锚点+分页行为。
   - src/tools/read.rs:66-146（审计 #2）
   - src/tools/read.rs:86, 93（审计 #2）
   - src/tools/read.rs:158-172（审计 #2）
4. 交互模块：上游依赖 crate::errors（USER_INPUT/REPO_NOT_READABLE 错误码）与 crate::fence::Fence（路径安全解析）；自身实现 tools::Tool trait，与 fuzzy 层 FileFinderTool/GrepTool 同注册表构成检索工具面；内建 sha2 行哈希、二进制启发探测、UTF-8 lossy 消毒。
   - src/tools/read.rs:4-6, 16, 86（审计 #2）
   - src/tools/read.rs:32-42, 96-107（审计 #2）
   - src/cli.rs:240-244（审计 #8）

## 死胡同
- 无；仅做概览，未展开 read.rs 测试细节（201-228 行）

## 置信度
high

## 统计
turns=6 · tool_calls=5 · duration=24463ms · tokens=34394
