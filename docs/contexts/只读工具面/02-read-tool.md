# read工具（只读工具面）

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
# read工具（只读工具面）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
`read` 是仓库内的只读强读工具，作为侦察链路唯一的事实来源：按 offset/limit 分页读取文件，输出「行号:行哈希锚点|内容」格式，并对二进制、非 UTF-8、空文件、offset 越界等情形给出诚实声明。已阅读 src/tools/read.rs（228 行全文）、src/tools/mod.rs、src/cli.rs:240-254 及工具调用图，可确认它在 ToolRegistry 中作为 `read` 名称注册的只读工具之一，由 CLI 启动时注入。

## 证据列表
1. `ReadTool` 是只读工具面 `Tool` trait 的实现，结构体定义在 src/tools/read.rs:15-23，通过 `new(fence: Arc<Fence>)` 注入沙箱。
   - src/tools/read.rs:15-23（审计 #2）
2. `Tool` trait 定义在 src/tools/mod.rs:14-22，要求实现 `name`/`description`/`parameters`/`execute`；`ReadTool` 注册名为 `"read"`（src/tools/read.rs:46-48），并声明参数 `{path, offset?, limit?}`（src/tools/read.rs:54-64）。
   - src/tools/mod.rs:14-43（审计 #7）
   - src/tools/read.rs:46-64（审计 #2）
3. `execute` 实现核心六特性：行号+12-bit 行哈希锚点（src/tools/read.rs:135）、offset/limit 分页与续读 offset（src/tools/read.rs:120, 138-143）、二进制探测拒倾倒（src/tools/read.rs:96-103, 32-42）、lossy UTF-8 编码消毒并声明 U+FFFD（src/tools/read.rs:105-107, 121-128）、空文件与 offset 越界的诚实反馈（src/tools/read.rs:110-118）。
   - src/tools/read.rs:66-146（审计 #2）
   - src/tools/read.rs:25-42（审计 #2）
4. 路径访问受 `Fence` 沙箱约束：execute 通过 `self.fence.resolve(&path_arg)`（src/tools/read.rs:86）将相对路径锁定在仓库根内，越界或非普通文件返回 `CsError(REPO_NOT_READABLE)`（src/tools/read.rs:87-92）。`DEFAULT_LIMIT=200`、`MAX_LIMIT=2000`（src/tools/read.rs:12-13）。
   - src/tools/read.rs:12-13（审计 #2）
   - src/tools/read.rs:86-94（审计 #2）
5. `ReadTool` 在 CLI 启动时由 src/cli.rs:247 通过 `registry.register(Box::new(tools::read::ReadTool::new(fence.clone())))` 注入 `ToolRegistry`，与 `FileFinderTool`、`GrepTool` 并列（src/cli.rs:248-250），作为只读工具面的首个注册项。
   - src/cli.rs:244-251（审计 #7）
6. `ReadTool` 同样在三处集成测试中被注册使用：tests/adversarial.rs:34、tests/replay.rs:66、tests/layered_live.rs:30，且 src/tools/read.rs:149-228 自身有 5 个单元测试覆盖锚点/分页/二进制/lossy/空文件/越界/缺参等路径。
   - src/tools/read.rs:149-228（审计 #2）
7. 依赖项：`CsError`/`CsResult`/`REPO_NOT_READABLE`/`USER_INPUT`（src/tools/read.rs:4, 71, 88-94, 220-226）、`Fence`（src/tools/read.rs:5, 86）、`Tool` trait（src/tools/read.rs:6, 45-146）、`async_trait`（src/tools/read.rs:7, 44, 66）、`serde_json::Value`（src/tools/read.rs:8, 54-64, 66-84）、`sha2::Sha256`（src/tools/read.rs:9, 27-28）。
   - src/tools/read.rs:1-30（审计 #2）
   - src/tools/read.rs:66-94（审计 #2）
   - src/tools/read.rs:220-226（审计 #2）

## 死胡同
- grep 检索 `tools::read::ReadTool` / `use.*tools::read` 命中 0 文件——src/cli.rs 使用的是 `tools::read::ReadTool::new(...)` 限定路径全称而非 use 别名（src/cli.rs:247）。
- grep 复合 `register.*read` 模式命中 0 文件，因 src/cli.rs:247 是多 token 构造 `registry.register(Box::new(tools::read::ReadTool::new(fence.clone())))`，单条 plain 模式被空格拆分。改用 `ReadTool::new` 检索后正常命中。
- explore 报告 `ReadTool` 仅 1 个 caller in `src/tools/read.rs`、⚠️ no covering tests——这是 codegraph 仅扫到单文件直接 new 调用的盲点，实际在 src/cli.rs:247 与三处集成测试（tests/adversarial.rs:34, tests/replay.rs:66, tests/layered_live.rs:30）均有 register 链路。

## 置信度
high

## 统计
turns=6 · tool_calls=8 · duration=34437ms · tokens=50845


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
read 是 codesleuth 侦察链路的唯一事实来源，只读强读工具。它在 src/tools/read.rs:15-23 定义 ReadTool 结构体，实现 src/tools/mod.rs:14-22 的 Tool trait（name="read"），按 offset/limit 分页读取 Fence 围栏内文件并输出「行号:12-bit hex 锚点|内容」格式；同时对二进制探测、lossy UTF-8 消毒、空文件、offset 越界、路径越界五类情形给出诚实声明而非静默倾倒。由 src/cli.rs:246-247 在 run_task_inner 中首位注册进 ToolRegistry，src/harness.rs:331-362 在主循环里通过 tools.get("read") 取到 trait 对象后调 execute(args).await 触发；输出回到 Harness 走 audit + Evidence::observe + 模型下一轮。共 5 个 #[tokio::test] 单测覆盖锚点/分页/二进制/lossy/空&越界/缺参，3 处集成测试（tests/adversarial.rs:34、tests/replay.rs:66、tests/layered_live.rs:30）同样注册 ReadTool。配置面无 CLI 旗标/环境变量/配置键直接控制 read 行为；DEFAULT_LIMIT=200、MAX_LIMIT=2000 为硬编码常量，limit 走 clamp(1,2000) 静默限幅。错误码段位 USER_INPUT=1001→exit 1、REPO_NOT_READABLE=3002→exit 4、FENCE_DENIED=3003 由 Fence 层抛，read 直接吃下不重映射。初稿总体准确，本报告在其上补全了 Fence 三检细节、Harness 旁路 observe_exact、二进制探测只看前 8192 字节等限制、以及与 CS error 码段位的对应关系。

## 证据列表
1. ReadTool 结构体定义在 src/tools/read.rs:15-23，通过 new(fence: Arc<Fence>) 注入沙箱；fn name() 返 "read"（src/tools/read.rs:46-48）；fn parameters() 声明 JSON Schema 必填 path、可选 offset(默认 1) 与 limit(默认 200 上限 2000)（src/tools/read.rs:54-64）。
   - src/tools/read.rs:15-23（审计 #2）
   - src/tools/read.rs:46-64（审计 #2）
2. execute 核心六特性：①行号+12-bit 行哈希锚点（src/tools/read.rs:25-29, 135）；②offset/limit 分页与续读 offset 提示（src/tools/read.rs:120, 138-143）；③二进制探测只看前 8192 字节（src/tools/read.rs:32-42, 96-103）；④lossy UTF-8 编码消毒并声明 U+FFFD（src/tools/read.rs:105-107, 121-128）；⑤空文件与 offset 越界诚实反馈（src/tools/read.rs:110-118）；⑥path 缺失返 USER_INPUT=1001 带 hint（src/tools/read.rs:67-73）。
   - src/tools/read.rs:66-146（审计 #2）
   - src/tools/read.rs:25-42（审计 #2）
3. Tool trait 在 src/tools/mod.rs:14-22，要求 name/description/parameters/execute 四方法 + Send + Sync；ToolRegistry 内部 Vec<Box<dyn Tool>>，按 name 线性查找（src/tools/mod.rs:24-43）；Fence 走「词汇层 normalize + canonicalize + 围栏前缀判定」三检，越界返 FENCE_DENIED=3003（src/fence.rs:30-55, 75-88）。
   - src/tools/mod.rs:14-43（审计 #4）
   - src/fence.rs:30-88（审计 #27）
4. 错误码段位定义：USER_INPUT=1001→exit 1、REPO_NOT_READABLE=3002、FENCE_DENIED=3003→exit 4、INTERNAL=5001（src/errors.rs:32-44, 101-104, 125）；read 工具 path 缺失用 USER_INPUT（src/tools/read.rs:71），文件非普通文件/IO 失败用 REPO_NOT_READABLE（src/tools/read.rs:88-94），越界由 Fence 直接抛 FENCE_DENIED。
   - src/errors.rs:30-104（审计 #45）
   - src/errors.rs:125（审计 #45）
   - src/tools/read.rs:67-94（审计 #2）
5. CLI 装配链路：main → Cli::run → run_task → run_task_inner（src/cli.rs:99, 184, 194）；dunce::canonicalize 解析 --repo 到绝对路径（src/cli.rs:207-214）；构造 Fence（src/cli.rs:245）→ 新建 ToolRegistry 并首位注册 ReadTool（src/cli.rs:246-247）→ 注册 FileFinderTool/GrepTool（src/cli.rs:248-250）→ 条件注册图工具（src/cli.rs:288-301）→ 可选向量层（src/cli.rs:304-334）→ Harness::new 接 registry（src/cli.rs:359-371）。
   - src/cli.rs:99-371（审计 #9）
6. Harness 主循环 dispatch：解析 tool_call → self.tools.get(&call.name)（src/harness.rs:331）→ 审计写 tool_call（src/harness.rs:350-353）→ 对 read 工具专门从 args 摘出 path 走 Evidence::observe_exact 旁路存证（src/harness.rs:357-378）→ tool.execute(args).await 触发 ReadTool（src/harness.rs:362）。
   - src/harness.rs:320-378（审计 #27）
7. 三处集成测试同样以 ReadTool::new(fence.clone()) 注册进 ToolRegistry 后交给 Harness：tests/adversarial.rs:34 含只读工具白名单 ["read","find_files","grep","explore","callers","callees","impact","files"]（tests/adversarial.rs:19-29）；tests/replay.rs:66 走录制回放断言；tests/layered_live.rs:30 默认 #[ignore] 真网关冒烟。
   - tests/adversarial.rs:19-39（审计 #11）
   - tests/replay.rs:10-89（审计 #13）
   - tests/layered_live.rs:13-43（审计 #13）
8. ReadTool 自身 5 个 #[tokio::test] 单测覆盖：①anchors_and_pagination_with_next_offset 锚点+分页+续读（src/tools/read.rs:158-172）；②binary_refuses_to_dump 二进制拒倾倒（src/tools/read.rs:174-184）；③lossy_utf8_declared 非 UTF-8 声明（src/tools/read.rs:186-198）；④empty_and_out_of_range_are_honest 空文件与越界（src/tools/read.rs:200-217）；⑤missing_path_arg_is_tool_error 缺 path 返 USER_INPUT（src/tools/read.rs:219-227）。
   - src/tools/read.rs:149-228（审计 #2）
9. read 工具无 CLI 旗标 / 环境变量 / config.toml 键直接控制（推断：grep 全文配置项零结果于 read.rs 内部，read 行为完全由 tool_call 参数与硬编码常量 DEFAULT_LIMIT=200 / MAX_LIMIT=2000 决定；最简验证途径：grep -n "DEFAULT_LIMIT\|MAX_LIMIT" src/tools/read.rs 应仅命中 12-13、83-84）。
   - src/tools/read.rs:12-13（审计 #2）
   - src/tools/read.rs:80-84（审计 #2）
10. Fence 测试覆盖越界/绝对路径/symlink 逃逸/不存在四类错误：src/fence.rs:104-116 symlink_escape_denied、118-124 absolute_outside_denied、126-132 nonexistent_is_repo_not_found，分别对应 FENCE_DENIED×2 与 REPO_NOT_FOUND，read 工具直接吃下这些 CsError 不重映射。
   - src/fence.rs:90-132（审计 #27）
   - src/errors.rs:42-44（审计 #45）

## 死胡同
- grep `USER_INPUT|REPO_NOT_READABLE` 复合模式首轮命中 0（管道分隔符被 plain 模式拆开），改为单模式后正常命中 src/errors.rs:32,43 与 src/tools/read.rs:4,71,88-94。
- grep `tools\.get|ToolRegistry|registry\.get|info_keys|observe_exact` 命中 0（应为字符 . 被 plain 模式视为字面量），改在 src/harness.rs 内直接定位 read 路径 331-378。
- explore 报告 ReadTool 仅 1 个 caller in src/tools/read.rs 是单文件盲点，实际注册链路在 src/cli.rs:247 与三处集成测试 tests/adversarial.rs:34 / tests/replay.rs:66 / tests/layered_live.rs:30。
- ToolRegistry 在 Harness 中是否走 Arc 包装未在 harness.rs 头部 200 行内直接出现，本报告相应位置标注为「推断」并以 path:line 复核作最简验证途径。

## 置信度
high

## 统计
turns=9 · tool_calls=23 · duration=111701ms · tokens=205105


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
`read` 是仓库内的只读强读工具，作为侦察链路唯一的事实来源：按 offset/limit 分页读取文件，输出「行号:行哈希锚点|内容」格式，并对二进制、非 UTF-8、空文件、offset 越界等情形给出诚实声明。已阅读 src/tools/read.rs（228 行全文）、src/tools/mod.rs、src/cli.rs:240-254 及工具调用图，可确认它在 ToolRegistry 中作为 `read` 名称注册的只读工具之一，由 CLI 启动时注入。

## 证据列表
1. `ReadTool` 是只读工具面 `Tool` trait 的实现，结构体定义在 src/tools/read.rs:15-23，通过 `new(fence: Arc<Fence>)` 注入沙箱。
   - src/tools/read.rs:15-23（审计 #2）
2. `Tool` trait 定义在 src/tools/mod.rs:14-22，要求实现 `name`/`description`/`parameters`/`execute`；`ReadTool` 注册名为 `"read"`（src/tools/read.rs:46-48），并声明参数 `{path, offset?, limit?}`（src/tools/read.rs:54-64）。
   - src/tools/mod.rs:14-43（审计 #7）
   - src/tools/read.rs:46-64（审计 #2）
3. `execute` 实现核心六特性：行号+12-bit 行哈希锚点（src/tools/read.rs:135）、offset/limit 分页与续读 offset（src/tools/read.rs:120, 138-143）、二进制探测拒倾倒（src/tools/read.rs:96-103, 32-42）、lossy UTF-8 编码消毒并声明 U+FFFD（src/tools/read.rs:105-107, 121-128）、空文件与 offset 越界的诚实反馈（src/tools/read.rs:110-118）。
   - src/tools/read.rs:66-146（审计 #2）
   - src/tools/read.rs:25-42（审计 #2）
4. 路径访问受 `Fence` 沙箱约束：execute 通过 `self.fence.resolve(&path_arg)`（src/tools/read.rs:86）将相对路径锁定在仓库根内，越界或非普通文件返回 `CsError(REPO_NOT_READABLE)`（src/tools/read.rs:87-92）。`DEFAULT_LIMIT=200`、`MAX_LIMIT=2000`（src/tools/read.rs:12-13）。
   - src/tools/read.rs:12-13（审计 #2）
   - src/tools/read.rs:86-94（审计 #2）
5. `ReadTool` 在 CLI 启动时由 src/cli.rs:247 通过 `registry.register(Box::new(tools::read::ReadTool::new(fence.clone())))` 注入 `ToolRegistry`，与 `FileFinderTool`、`GrepTool` 并列（src/cli.rs:248-250），作为只读工具面的首个注册项。
   - src/cli.rs:244-251（审计 #7）
6. `ReadTool` 同样在三处集成测试中被注册使用：tests/adversarial.rs:34、tests/replay.rs:66、tests/layered_live.rs:30，且 src/tools/read.rs:149-228 自身有 5 个单元测试覆盖锚点/分页/二进制/lossy/空文件/越界/缺参等路径。
   - src/tools/read.rs:149-228（审计 #2）
7. 依赖项：`CsError`/`CsResult`/`REPO_NOT_READABLE`/`USER_INPUT`（src/tools/read.rs:4, 71, 88-94, 220-226）、`Fence`（src/tools/read.rs:5, 86）、`Tool` trait（src/tools/read.rs:6, 45-146）、`async_trait`（src/tools/read.rs:7, 44, 66）、`serde_json::Value`（src/tools/read.rs:8, 54-64, 66-84）、`sha2::Sha256`（src/tools/read.rs:9, 27-28）。
   - src/tools/read.rs:1-30（审计 #2）
   - src/tools/read.rs:66-94（审计 #2）
   - src/tools/read.rs:220-226（审计 #2）

## 死胡同
- grep 检索 `tools::read::ReadTool` / `use.*tools::read` 命中 0 文件——src/cli.rs 使用的是 `tools::read::ReadTool::new(...)` 限定路径全称而非 use 别名（src/cli.rs:247）。
- grep 复合 `register.*read` 模式命中 0 文件，因 src/cli.rs:247 是多 token 构造 `registry.register(Box::new(tools::read::ReadTool::new(fence.clone())))`，单条 plain 模式被空格拆分。改用 `ReadTool::new` 检索后正常命中。
- explore 报告 `ReadTool` 仅 1 个 caller in `src/tools/read.rs`、⚠️ no covering tests——这是 codegraph 仅扫到单文件直接 new 调用的盲点，实际在 src/cli.rs:247 与三处集成测试（tests/adversarial.rs:34, tests/replay.rs:66, tests/layered_live.rs:30）均有 register 链路。

## 置信度
high

## 统计
turns=6 · tool_calls=8 · duration=34437ms · tokens=50845
