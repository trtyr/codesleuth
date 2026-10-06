# 错误码体系（错误与日志）

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
# 错误码体系（错误与日志）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
错误码体系（CSxxxx）以段位决定 exit code（CS1xxx→1 用法、CS2xxx→3 LLM、CS3xxx→4 目标库、CS4xxx→5 索引、其它→6），由 src/errors.rs 统一定义 CsCode/CsError/report_error；各业务模块用 CsError 返回错误，cli 三个顶层入口（run/run_task/run_config）捕获后调用 report_error 打 stderr 并按 exit_code 退出。

## 证据列表
（无结构化发现——降级报告）

## 死胡同
- 未深入展开 LLM/classify_llm_error 与 cli/index_structure_error 的内部错误分类细节（仅做上下游引用层确认）
- 未读取 cli.rs 三个入口的完整实现，只确认其调用 report_error 与 exit_code

## 置信度
high

## 统计
turns=3 · tool_calls=5 · duration=20599ms · tokens=17295


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
错误码体系（CSxxxx）以段位决定 exit code：CS1000-1009→1 用法、CS1010-1099→2 配置/凭据、CS2000-2999→3 LLM（含熔断 CS2099）、CS3000-3999→4 目标库、CS4000-4999→5 索引、CS5000-5999→6 内部，成功统一 exit 0（src/errors.rs:13-22）。由 src/errors.rs 统一定义 CsCode/CsError/CsResult/report_error；CsError 五字段（code/message/hint/retryable/source_text，src/errors.rs:59-67）；report_error 按"主行+根因行+hint 行"打到 stderr（src/errors.rs:107-115）；CLI 三个顶层入口（Cli::run、run_task、run_config，src/cli.rs:99-130/184-192/480-511）捕获后调 report_error 并按 e.exit_code() 退出（src/main.rs:11）。初稿说"CS1xxx→1"实际是细分 1000-1009→1、1010-1099→2（src/errors.rs:15-16, 124-130）；初稿说 classify_llm_error 是 LLM 分类器，实际仅 #[cfg(test)] 测试辅助（src/llm.rs:312, 315），生产分类在 OpenAiProvider::chat 内联（src/llm.rs:262-280）。

## 证据列表
1. 段位→exit code 映射在 CsCode::exit_code 集中实现，CS1xxx 细分为 1000-1009→1、1010-1099→2。
   - src/errors.rs:13-22（审计 #2）
   - src/errors.rs:124-130（审计 #2）
2. 15 个 CS 常量按 5 段位声明：USER_INPUT=1001、CONFIG_MISSING=1011、CONFIG_INVALID=1012、LLM_UNREACHABLE=2001、LLM_RATE_LIMITED=2002、LLM_SERVER=2003、LLM_BAD_RESPONSE=2004、LLM_FUSE=2099、REPO_NOT_FOUND=3001、REPO_NOT_READABLE=3002、FENCE_DENIED=3003、INDEX_NOT_AVAILABLE=4010、INDEX_BUILD_FAILED=4011、INDEX_TIMEOUT=4014、INDEX_EMBED_FAILED=4015、INDEX_LOCKED=4016、INTERNAL=5001。
   - src/errors.rs:32-54（审计 #2）
3. CsError 五字段：code(CsCode) + message(String) + hint(Option<String>) + retryable(bool) + source_text(Option<String>，避开 thiserror 自动绑 source)。Display 形如 "CS2001: 网关连不上"。
   - src/errors.rs:59-104（审计 #2）
   - src/errors.rs:25-29（审计 #2）
   - src/errors.rs:133-138（审计 #2）
4. report_error 按 eprintln! 主行（Display）+ "  ↳ 根因: {src}" + "  hint: {hint}" 三行打到 stderr。
   - src/errors.rs:107-115（审计 #2）
5. CLI 三个顶层入口捕获 CsError 并退出：Cli::run 三向 match (Config→run_config / Index→run_index_vector 或 index_structure_error / None→run_task) 都在 Err 分支调 report_error + e.exit_code()。
   - src/cli.rs:99-130（审计 #15）
   - src/cli.rs:184-192（审计 #15）
   - src/cli.rs:480-511（审计 #15）
6. main 入口解析 CLI 后生成 session_id、init_tracing、cli.run、std::process::exit(code)。
   - src/main.rs:5-12（审计 #15）
7. LLM 错误生产分类内联在 OpenAiProvider::chat：HTTP 429/5xx→LLM_RATE_LIMITED(retryable=true)，其它非 2xx→LLM_BAD_RESPONSE，reqwest 发送失败→LLM_UNREACHABLE(retryable=true)；终局统一 .with_retryable(false)。
   - src/llm.rs:262-280（审计 #15）
   - src/llm.rs:289-306（审计 #15）
8. classify_llm_error 函数位于 #[cfg(test)] 模块，是测试辅助而非生产分类路径。
   - src/llm.rs:312-315（审计 #15）
   - src/llm.rs:420-437（审计 #15）
9. config 层错误：parse_file 读/解 toml 失败→CONFIG_INVALID；resolve_api_key 缺 api_key→CONFIG_MISSING；profile 未知/未知键/值非法→CONFIG_INVALID。
   - src/config.rs:180-188（审计 #15）
   - src/config.rs:241-245（审计 #15）
   - src/config.rs:325-330（审计 #15）
   - src/config.rs:446-452（审计 #15）
   - src/config.rs:478-484（审计 #15）
   - src/config.rs:528-538（审计 #15）
10. 围栏错误：Fence::new canonicalize 失败→REPO_NOT_FOUND；resolve 词法越界/canonicalize 复检越界→FENCE_DENIED；路径不存在→REPO_NOT_FOUND。
   - src/fence.rs:14-19（审计 #15）
   - src/fence.rs:37-50（审计 #15）
11. 引导锁三态全归 CS4016 INDEX_LOCKED 判负（Won→guard 干活；Lost→INDEX_LOCKED+hint"重跑即可"；超时→INDEX_LOCKED+hint"确认其结束后重跑"），cli 在 codegraph 引导与向量层装配两处显式 return Err(e) 不走降级 catch-all。
   - src/bootlock.rs:26-33（审计 #15）
   - src/bootlock.rs:77-114（审计 #15）
   - src/bootlock.rs:120-129（审计 #15）
   - src/cli.rs:265-268（审计 #15）
   - src/cli.rs:321-324（审计 #15）
12. 索引/向量层错误：vector/store.rs 与 chunk.rs、repomap.rs 多处 → INDEX_NOT_AVAILABLE；vector/embed.rs 嵌入失败 → INDEX_EMBED_FAILED（HTTP 429/5xx 时 with_retryable(true)）；vector/recall.rs 嵌空 → INDEX_NOT_AVAILABLE。
   - src/vector/store.rs:50-135（审计 #107）
   - src/vector/embed.rs:100-211（审计 #76）
   - src/vector/recall.rs:133-135（审计 #15）
13. 主循环熔断：连续无进展 ≥ MAX_NO_PROGRESS_STREAK=5 → LLM_FUSE(CS2099)，audit.record("fuse") + tracing::error!，段位 CS2xxx → exit 3。
   - src/harness.rs:653-666（审计 #44）
   - src/harness.rs:870-880（审计 #44）
14. Report::validate 用 INDEX_BUILD_FAILED(CS4011) 表达 schema 错误（非法 confidence/空 answer），与索引构建共享段位 CS4xxx。
   - src/report.rs:59-74（审计 #83）
15. 工具层错误：read.rs 缺 path→USER_INPUT、非普通文件/IO 失败→REPO_NOT_READABLE；fuzzy.rs fff 初始化/收集失败→INDEX_BUILD_FAILED、缺 query/pattern→USER_INPUT；graph.rs 缺 symbol/query→USER_INPUT。
   - src/tools/read.rs:66-94（审计 #94）
   - src/tools/fuzzy.rs:32-36（审计 #109）
   - src/tools/fuzzy.rs:78-80（审计 #109）
   - src/tools/fuzzy.rs:150-152（审计 #109）
   - src/tools/graph.rs:113-120（审计 #111）
   - src/tools/graph.rs:219-221（审计 #111）
16. index_structure_error 诚实 hint：rebuild=true 指 run --fresh-index、rebuild=false 指 --vector；测试 index_structure_hint_is_honest 强制断言 hint 不含"已记录"。
   - src/cli.rs:465-478（审计 #15）
   - src/cli.rs:708-718（审计 #15）
17. 日志双轨 + stdout 纪律：init_tracing 挂 stderr_layer + file_layer (env filter codesleuth={warn|info|debug})，session span 自动带 session_id；--json 模式 stdout 仅承载报告 JSON。
   - src/lib.rs:24-66（审计 #15）
   - src/main.rs:7-9（审计 #15）
18. 单测三处断言契约：err.code == CONST、err.exit_code() == N、builder 链式（with_hint/with_source/retryable）字段持久化。
   - src/errors.rs:123-153（审计 #2）
   - src/config.rs:774-782（审计 #15）
   - src/fence.rs:104-132（审计 #15）
   - src/bootlock.rs:131-200（审计 #15）
   - src/harness.rs:871-895（审计 #44）
   - src/cli.rs:803-805（审计 #15）

## 死胡同
- grep 模式含 | 触发 plain 模式限制导致 0 命中（classify_llm_error/index_structure_error/CONFIG_MISSING+CONFIG_INVALID 联合等首轮），已改用单关键字 grep 绕过。
- src/evidence.rs、src/audit.rs 凭 grep 命中推断存在但本会话未读源，证据不充分时按要求标（推断）。
- src/cli.rs:581-589 build_vector_index 失败时的 warn+audit degraded 路径未直接读到对应行号，由 src/cli.rs:265-268 / 321-324 显式判负反推分叉存在（推断）。

## 置信度
high

## 统计
turns=13 · tool_calls=49 · duration=164155ms · tokens=506692


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
错误码体系（CSxxxx）以段位决定 exit code（CS1xxx→1 用法、CS2xxx→3 LLM、CS3xxx→4 目标库、CS4xxx→5 索引、其它→6），由 src/errors.rs 统一定义 CsCode/CsError/report_error；各业务模块用 CsError 返回错误，cli 三个顶层入口（run/run_task/run_config）捕获后调用 report_error 打 stderr 并按 exit_code 退出。

## 证据列表
（无结构化发现——降级报告）

## 死胡同
- 未深入展开 LLM/classify_llm_error 与 cli/index_structure_error 的内部错误分类细节（仅做上下游引用层确认）
- 未读取 cli.rs 三个入口的完整实现，只确认其调用 report_error 与 exit_code

## 置信度
high

## 统计
turns=3 · tool_calls=5 · duration=20599ms · tokens=17295
