# 结构化错误与退出码（LLM 接入与运维）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`02-error-codes-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 结构化错误与退出码（LLM 接入与运维）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
结构化错误与退出码功能为 CLI 调用方提供可编程判别的错误体系：每个错误是 CsError（CS 编码 + 人话消息 + 修复建议 + 可重试标记 + 根因链），退出码由错误码段位决定（1000-1009→1 用法、1010-1099→2 配置/凭据、CS2xxx→3 上游 LLM、CS3xxx→4 目标库、CS4xxx→5 索引、其他→6 内部），供脚本按退出码分类处理。核心定义在 src/errors.rs（CsCode::exit_code、CsError、report_error），CLI 各命令入口统一 catch 后调用 report_error 打印并返回 e.exit_code() 作为进程退出码；上游 LLM 模块将 HTTP 状态分类映射到 CS 码（如 429→LLM_RATE_LIMITED、401→CONFIG_MISSING）。下游 config、fence、harness 等模块的测试分别断言配置错误 exit_code=2、围栏错误=4、LLM 熔断错误=3，验证分段契约。

## 证据列表
1. CsCode::exit_code 按码段映射退出码 1-6：1000-1009→1 用法、1010-1099→2 配置、2000-2999→3 上游 LLM、3000-3999→4 目标库、4000-4999→5 索引、其余→6 内部
   - src/errors.rs:13-22（审计 #2）
2. CsError 结构含 code、message、hint、retryable、source_text（根因链）；report_error 将其以主行 + 根因 + hint 行输出到 stderr
   - src/errors.rs:56-67（审计 #2）
   - src/errors.rs:106-115（审计 #2）
3. 错误码常量按段位定义：CS1xxx 用户/配置、CS2xxx 上游 LLM（含 LLM_FUSE 熔断 2099）、CS3xxx 目标库/围栏、CS4xxx 索引、CS5xxx 内部
   - src/errors.rs:32-53（审计 #2）
4. CLI 命令入口（如 run_index_vector 与 run_task）统一 catch CsError：report_error 打印后返回 e.exit_code() 作为进程退出码
   - src/cli.rs:110-122（审计 #13）
   - src/cli.rs:180-188（审计 #13）
   - src/cli.rs:7-7（审计 #13）
5. 上游 LLM 错误分类：classify_llm_error 按 HTTP 状态映射 CS 码——429→LLM_RATE_LIMITED（可重试）、401/403→CONFIG_MISSING、5xx→LLM_SERVER、其他→LLM_BAD_RESPONSE、无状态→LLM_UNREACHABLE
   - src/llm.rs:304-321（审计 #4）
6. 下游模块测试消费该契约：config.rs 断言配置错误 exit_code=2，fence.rs 断言围栏拒绝 exit_code=4，harness.rs 断言 LLM 熔断 exit_code=3
   - src/config.rs:606-610（审计 #18）
   - src/fence.rs:112-116（审计 #20）
   - src/harness.rs:812-816（审计 #22）

## 死胡同
无

## 置信度
high

## 统计
turns=7 · tool_calls=8 · duration=31946ms · tokens=49039


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
结构化错误与退出码功能是全仓唯一错误通道：CsError（CS 码+消息+hint+retryable+source_text 根因链，src/errors.rs:59-67）由 config/llm/fence/harness/vector 等模块构造，沿 CsResult 上抛，CLI 各命令入口（src/cli.rs:96-126,180-188,474-505）统一 catch 后 report_error 打印到 stderr（src/errors.rs:107-115），e.exit_code() 经 CsCode::exit_code 段位映射（src/errors.rs:13-22：1xxx→1、101x→2、2xxx→3、3xxx→4、4xxx→5、其余→6）成为进程退出码（src/main.rs:11）。重要勘误：初稿引用的 classify_llm_error（src/llm.rs:304-321）是 #[cfg(test)] 测试专用函数（llm.rs:301-303），生产路径真实映射在 OpenAiProvider::chat（llm.rs:249-260）：429 或 5xx→LLM_RATE_LIMITED（可重试）、其他状态→LLM_BAD_RESPONSE、网络错误→LLM_UNREACHABLE——初稿所说「401→CONFIG_MISSING、5xx→LLM_SERVER」仅为测试预期，生产行为不符。retryable 仅供 provider 内部指数退避重试决策（llm.rs:230-236,261-265），终局失败被 with_retryable(false) 抹平，脚本只能靠退出码盲分。降级类错误（codegraph/vector 装配失败）不产生非零退出码。下游测试锁定分段契约：config→2、fence→4、harness 熔断→3、cli 用法错→1。

## 证据列表
1. 退出码由 CsCode::exit_code 按码段映射：1000-1009→1、1010-1099→2、2000-2999→3、3000-3999→4、4000-4999→5、其余→6；e.exit_code() 委托此映射，main.rs 以 std::process::exit(code) 落地
   - src/errors.rs:13-22（审计 #2）
   - src/errors.rs:101-103（审计 #2）
   - src/main.rs:5-12（审计 #45）
2. CsError 含 code/message/hint/retryable/source_text 五字段；report_error 以主行+↳根因+hint 三行打 stderr；source_text 刻意避开 thiserror 的 source 字段名；Display 输出 CS{:04}: message 格式
   - src/errors.rs:56-67（审计 #2）
   - src/errors.rs:107-115（审计 #2）
   - src/errors.rs:80-84（审计 #2）
   - src/errors.rs:25-29（审计 #2）
3. CLI 全部命令入口统一 catch CsError：Cli::run 分发 index/run_task/run_config 三路径，每处 Err 都 report_error + e.exit_code()；index 非 --vector 走 index_structure_error 直接构造 INDEX_NOT_AVAILABLE
   - src/cli.rs:96-126（审计 #4）
   - src/cli.rs:180-188（审计 #4）
   - src/cli.rs:489-502（审计 #4）
   - src/cli.rs:462-472（审计 #4）
4. 初稿勘误：classify_llm_error（llm.rs:304-321）是 #[cfg(test)] 函数（llm.rs:301-303），非生产路径；生产映射在 OpenAiProvider::chat：429 或 5xx→LLM_RATE_LIMITED(retryable)、其他状态→LLM_BAD_RESPONSE、网络错误→LLM_UNREACHABLE，与初稿所述 401→CONFIG_MISSING、5xx→LLM_SERVER 不符
   - src/llm.rs:301-321（审计 #14）
   - src/llm.rs:249-260（审计 #14）
   - src/llm.rs:230-236（审计 #14）
   - src/llm.rs:280-288（审计 #14）
5. retryable 仅驱动 provider 内部指数退避重试（500ms 起步封顶），终局失败统一 with_retryable(false) 抹平——重试语义对脚本不可见，退出码只编码段位
   - src/llm.rs:261-265（审计 #14）
   - src/llm.rs:267-275（审计 #14）
   - src/llm.rs:292-294（审计 #14）
6. 主要生产构造点：resolve_api_key 缺 key→CONFIG_MISSING；缺 task/--repo→USER_INPUT；canonicalize 失败→REPO_NOT_FOUND+with_source；TOML 非法→CONFIG_INVALID(exit 2)；fence 越界→FENCE_DENIED(exit 4)；熔断 LLM_FUSE(exit 3)；cli 测试锁 USER_INPUT(exit 1)
   - src/config.rs:456-468（审计 #32）
   - src/cli.rs:228（审计 #4）
   - src/cli.rs:191-210（审计 #4）
   - src/config.rs:603-610（审计 #32）
   - src/fence.rs:113-115（审计 #34）
   - src/harness.rs:812-816（审计 #25）
   - src/cli.rs:790-795（审计 #4）
7. 降级错误不产生非零退出码：codegraph/vector 层装配失败仅 warn + 审计 degraded 行，进程仍 exit 0；熔断经 fuse_if_hit 记审计 fuse 行后返回 LLM_FUSE；成功路径报告落 ~/.codesleuth/reports/{session_id}.md/.json
   - src/harness.rs:617-629（审计 #25）
   - src/cli.rs:263-271（审计 #4）
   - src/cli.rs:319-326（审计 #4）
   - src/cli.rs:428-436（审计 #4）
8. 边界/语义混用坑：Report::validate 复用 INDEX_BUILD_FAILED 报告校验错（非法置信度/空 answer），脚本会看到 exit 5；CsCode::_ 兜底使任何未映射码（含未来新段位）静默归 exit 6
   - src/report.rs:59-74（审计 #4）
   - src/errors.rs:46-47（审计 #2）

## 死胡同
- grep "LLM_FUSE|with_retryable..." plain 模式 0 命中（语法含 | 被当字面量），换 regex 模式后命中
- grep process::exit：唯一命中 src/main.rs:11，确认退出码单点收敛，无其他分支

## 置信度
high

## 统计
turns=12 · tool_calls=18 · duration=147944ms · tokens=314912


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
结构化错误与退出码功能为 CLI 调用方提供可编程判别的错误体系：每个错误是 CsError（CS 编码 + 人话消息 + 修复建议 + 可重试标记 + 根因链），退出码由错误码段位决定（1000-1009→1 用法、1010-1099→2 配置/凭据、CS2xxx→3 上游 LLM、CS3xxx→4 目标库、CS4xxx→5 索引、其他→6 内部），供脚本按退出码分类处理。核心定义在 src/errors.rs（CsCode::exit_code、CsError、report_error），CLI 各命令入口统一 catch 后调用 report_error 打印并返回 e.exit_code() 作为进程退出码；上游 LLM 模块将 HTTP 状态分类映射到 CS 码（如 429→LLM_RATE_LIMITED、401→CONFIG_MISSING）。下游 config、fence、harness 等模块的测试分别断言配置错误 exit_code=2、围栏错误=4、LLM 熔断错误=3，验证分段契约。

## 证据列表
1. CsCode::exit_code 按码段映射退出码 1-6：1000-1009→1 用法、1010-1099→2 配置、2000-2999→3 上游 LLM、3000-3999→4 目标库、4000-4999→5 索引、其余→6 内部
   - src/errors.rs:13-22（审计 #2）
2. CsError 结构含 code、message、hint、retryable、source_text（根因链）；report_error 将其以主行 + 根因 + hint 行输出到 stderr
   - src/errors.rs:56-67（审计 #2）
   - src/errors.rs:106-115（审计 #2）
3. 错误码常量按段位定义：CS1xxx 用户/配置、CS2xxx 上游 LLM（含 LLM_FUSE 熔断 2099）、CS3xxx 目标库/围栏、CS4xxx 索引、CS5xxx 内部
   - src/errors.rs:32-53（审计 #2）
4. CLI 命令入口（如 run_index_vector 与 run_task）统一 catch CsError：report_error 打印后返回 e.exit_code() 作为进程退出码
   - src/cli.rs:110-122（审计 #13）
   - src/cli.rs:180-188（审计 #13）
   - src/cli.rs:7-7（审计 #13）
5. 上游 LLM 错误分类：classify_llm_error 按 HTTP 状态映射 CS 码——429→LLM_RATE_LIMITED（可重试）、401/403→CONFIG_MISSING、5xx→LLM_SERVER、其他→LLM_BAD_RESPONSE、无状态→LLM_UNREACHABLE
   - src/llm.rs:304-321（审计 #4）
6. 下游模块测试消费该契约：config.rs 断言配置错误 exit_code=2，fence.rs 断言围栏拒绝 exit_code=4，harness.rs 断言 LLM 熔断 exit_code=3
   - src/config.rs:606-610（审计 #18）
   - src/fence.rs:112-116（审计 #20）
   - src/harness.rs:812-816（审计 #22）

## 死胡同
无

## 置信度
high

## 统计
turns=7 · tool_calls=8 · duration=31946ms · tokens=49039
