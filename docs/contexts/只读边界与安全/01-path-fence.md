# 路径围栏（只读边界与安全）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`01-path-fence-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 路径围栏（只读边界与安全）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

路径围栏（Fence）是只读边界安全层：保证所有工具目标路径解析后落在目标仓库根目录内，词汇越界（..、绝对路径）与 symlink 逃逸一律拒绝并返回 FENCE_DENIED 结构化错误，为调用方提供「读不越出仓库」的安全保证。入口 src/fence.rs:30（resolve），Fence 定义于 src/fence.rs:7，由 src/cli.rs:239 在启动时以仓库根创建并注入工具注册表。运作链：cli.rs 创建 Fence → ReadTool 持有 Arc<Fence> → read.rs:86 每次读文件前调 fence.resolve → resolve 先词汇预检（normalize 折叠 .. 后 starts_with_root 检查，fence.rs:37/59/75），再 dunce::canonicalize 解 symlink 后复检（fence.rs:43-48），两道都不过即 FENCE_DENIED。上游依赖 errors 模块的 CsError/FENCE_DENIED/REPO_NOT_FOUND 与 dunce::canonicalize；下游消费 ReadTool；FuzzyEngine 仅用 Fence::new 取规范化根目录定界 picker（fuzzy.rs:24-27），不消费 resolve；tests/adversarial.rs 覆盖 symlink/绝对/相对越界攻击面。

## 证据列表

1. 路径围栏确保所有工具目标路径解析后落在仓库根内，symlink 逃逸拒绝，越界返回 FENCE_DENIED 结构化错误
    - src/fence.rs:1-2（审计 #2）
    - src/fence.rs:27-55（审计 #2）
2. 入口 resolve 于 fence.rs:30，Fence 在 cli.rs:239 创建并注入 ReadTool
    - src/fence.rs:7-30（审计 #2）
    - src/cli.rs:239-241（审计 #7）
3. 调用链：ReadTool 调 fence.resolve；resolve 先词汇预检（normalize+starts_with_root）再 canonicalize 解 symlink 复检
    - src/tools/read.rs:86（审计 #18）
    - src/fence.rs:37-48（审计 #2）
    - src/fence.rs:58-88（审计 #2）
4. 交互：依赖 errors 模块错误码与 dunce::canonicalize；FuzzyEngine 仅用 Fence 取规范化根目录；tests/adversarial.rs 覆盖 symlink/绝对/相对越界
    - src/fence.rs:4（审计 #2）
    - src/tools/fuzzy.rs:24-27（审计 #23）
    - tests/adversarial.rs:152-158（审计 #25）

## 死胡同

无

## 置信度

high

## 统计

turns=8 · tool_calls=9 · duration=26288ms · tokens=46455

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
路径围栏是恒开启的只读边界安全层，无任何配置项/环境变量控制。完整链路：cli.rs:203 canonicalize --repo 得 repo_abs → cli.rs:239 Fence::new（内部再 canonicalize，失败 REPO_NOT_FOUND CS3001）→ cli.rs:241 注入 ReadTool（Arc<Fence>，read.rs:15-17）→ Harness 经 ToolRegistry 分发 → ReadTool::execute（read.rs:66-84 解析 path/offset/limit）→ read.rs:86 调 fence.resolve：①词汇预检 root.join(rel)→current_dir 拼接→normalize 折叠 ./.（fence.rs:59-71）→starts_with_root（fence.rs:75-88，Windows 忽略 ASCII 大小写、其他平台严格，fence.rs:77-86）失败→FENCE_DENIED（CS3003，fence.rs:37-42）；②dunce::canonicalize 解 symlink 后复检（fence.rs:43-53），不存在→REPO_NOT_FOUND。落点：read.rs:87-94 is_file 校验 + std::fs::read → 二进制探测/UTF-8 lossy/分页行哈希输出。坑：越界 FENCE_DENIED 与不存在 REPO_NOT_FOUND 退出码同为 4（errors.rs:18,42-44,128），仅凭 exit code 不可区分；fence.rs:35 current_dir unwrap_or_default 静默兜底（实践中因 root 恒绝对不可达，推断）；围栏只保护 read 工具，fuzzy 层仅用 Fence::root() 定界 fff picker（fuzzy.rs:25-27），graph/vector 工具路径边界在各自引擎内部（推断）。tests/adversarial.rs:141-161 覆盖 symlink/绝对/相对三类敌对路径。初稿结论全部与代码一致，无错误。

## 证据列表
1. Fence 创建与注入：cli.rs:239 以 canonicalize 后的 repo_abs 建 Fence，241 注入 ReadTool；Fence::new 内部 canonicalize 失败→REPO_NOT_FOUND（fence.rs:12-20）
   - src/fence.rs:30-55（审计 #2）
   - src/fence.rs:7-25（审计 #2）
   - src/cli.rs:239-241（审计 #4）
2. 运行期调用链：ReadTool::execute 解析 JSON 参数后于 read.rs:86 调 fence.resolve；resolve 先 normalize（折叠 ./..）+starts_with_root 词汇预检，再 canonicalize 解 symlink 复检，两道都不过才 FENCE_DENIED
   - src/tools/read.rs:66-94（审计 #7）
   - src/fence.rs:59-88（审计 #2）
3. 错误契约：FENCE_DENIED=CS3003、REPO_NOT_FOUND=CS3001、REPO_NOT_READABLE=CS3002，同属 3000-3999 段→exit code 4，调用方不可仅凭退出码区分被拒与不存在
   - src/errors.rs:13-44（审计 #17）
   - src/errors.rs:119-131（审计 #17）
   - src/fence.rs:127-132（审计 #2）
4. 平台适配与配置面：starts_with_root 在 Windows 用 eq_ignore_ascii_case 逐组件比较，其他平台严格比较；全仓无任何配置项/环境变量控制 Fence，恒开启
   - src/fence.rs:73-88（审计 #2）
   - src/tools/fuzzy.rs:22-37（审计 #9）
   - src/cli.rs:239-244（审计 #4）
5. 数据流与测试契约：resolve 结果供 read.rs is_file 校验+std::fs::read+行哈希分页输出；fence.rs 内建单测含 exit_code=4 断言（fence.rs:115），tests/adversarial.rs:153 覆盖 innocent.txt//etc/passwd/../../etc/passwd 三类敌对路径
   - src/tools/read.rs:86-145（审计 #7）
   - src/fence.rs:90-133（审计 #2）
   - tests/adversarial.rs:141-161（审计 #14）
6. 边界与推断项：fence.rs:35 current_dir().unwrap_or_default() 静默兜底（因 root 恒为 canonicalize 后绝对路径，实践中不可达，推断）；fuzzy 层仅消费 Fence::root() 定界 fff picker，不经 resolve；graph/vector 工具路径边界在各自引擎内部（未读，推断）
   - src/fence.rs:31-36（审计 #2）
   - src/tools/fuzzy.rs:24-27（审计 #9）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=8 · duration=59888ms · tokens=62877


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

路径围栏（Fence）是只读边界安全层：保证所有工具目标路径解析后落在目标仓库根目录内，词汇越界（..、绝对路径）与 symlink 逃逸一律拒绝并返回 FENCE_DENIED 结构化错误，为调用方提供「读不越出仓库」的安全保证。入口 src/fence.rs:30（resolve），Fence 定义于 src/fence.rs:7，由 src/cli.rs:239 在启动时以仓库根创建并注入工具注册表。运作链：cli.rs 创建 Fence → ReadTool 持有 Arc<Fence> → read.rs:86 每次读文件前调 fence.resolve → resolve 先词汇预检（normalize 折叠 .. 后 starts_with_root 检查，fence.rs:37/59/75），再 dunce::canonicalize 解 symlink 后复检（fence.rs:43-48），两道都不过即 FENCE_DENIED。上游依赖 errors 模块的 CsError/FENCE_DENIED/REPO_NOT_FOUND 与 dunce::canonicalize；下游消费 ReadTool；FuzzyEngine 仅用 Fence::new 取规范化根目录定界 picker（fuzzy.rs:24-27），不消费 resolve；tests/adversarial.rs 覆盖 symlink/绝对/相对越界攻击面。

## 证据列表

1. 路径围栏确保所有工具目标路径解析后落在仓库根内，symlink 逃逸拒绝，越界返回 FENCE_DENIED 结构化错误
    - src/fence.rs:1-2（审计 #2）
    - src/fence.rs:27-55（审计 #2）
2. 入口 resolve 于 fence.rs:30，Fence 在 cli.rs:239 创建并注入 ReadTool
    - src/fence.rs:7-30（审计 #2）
    - src/cli.rs:239-241（审计 #7）
3. 调用链：ReadTool 调 fence.resolve；resolve 先词汇预检（normalize+starts_with_root）再 canonicalize 解 symlink 复检
    - src/tools/read.rs:86（审计 #18）
    - src/fence.rs:37-48（审计 #2）
    - src/fence.rs:58-88（审计 #2）
4. 交互：依赖 errors 模块错误码与 dunce::canonicalize；FuzzyEngine 仅用 Fence 取规范化根目录；tests/adversarial.rs 覆盖 symlink/绝对/相对越界
    - src/fence.rs:4（审计 #2）
    - src/tools/fuzzy.rs:24-27（审计 #23）
    - tests/adversarial.rs:152-158（审计 #25）

## 死胡同

无

## 置信度

high

## 统计

turns=8 · tool_calls=9 · duration=26288ms · tokens=46455
