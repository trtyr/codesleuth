# Harness主循环（侦察编排）

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
# Harness主循环（侦察编排）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
Harness::run（src/harness.rs:74-419）是 codesleuth Agent 的主循环：以单次任务字符串为输入，循环驱动 LLM 决策 → 只读工具调度 → 增量记账 → 通过 submit_report 收敛为带 evidence 校验的结构化 RunOutcome，或在连续无进展达 MAX_NO_PROGRESS_STREAK=5（src/harness.rs:19）时由 fuse_if_hit 返回 LLM_FUSE 熔断。证据校验在 build_report（src/harness.rs:460-598）里完成：findings 引用的 file 必须本会话真实观察到（cite_seq），否则连同零 findings 错误一并打回。

## 证据列表
1. Harness::run 是主循环入口，在 src/harness.rs:74-419 定义，构造 system+user 首消息并进入 loop。
   - src/harness.rs:74-99（审计 #2）
2. Harness 结构体持有 provider / tools / audit / evidence / model / context_tokens / compact_percent。
   - src/harness.rs:35-44（审计 #2）
3. 熔断阈值常量 MAX_NO_PROGRESS_STREAK = 5 在 src/harness.rs:19 定义。
   - src/harness.rs:18-19（审计 #2）
4. fuse_if_hit 在 streak >= MAX_NO_PROGRESS_STREAK 时返回 LLM_FUSE 错误并 audit.record("fuse")。
   - src/harness.rs:653-666（审计 #2）
5. 主循环每轮先按 context::should_compact 判断压缩，再 provider.chat 调 LLM。
   - src/harness.rs:100-148（审计 #2）
6. 工具调度按 submit_report / recall / 去重 / 工具存在性 / 工具执行分支处理；submit_report 走 build_report 校验 evidence 引用。
   - src/harness.rs:222-417（审计 #2）
7. 重复调用、非法 JSON、未知工具、零增量执行、工具错误均 no_progress += 1 并触发 fuse_if_hit。
   - src/harness.rs:300-416（审计 #2）
8. build_report 校验 evidence 引用的 file 必须 evidence.cite_seq 能查到（cite_seq 返回 None 计入 uncited 并返回 Err）。
   - src/harness.rs:460-598（审计 #2）
9. run_with 测试夹具用 Scripted provider + Harness::new 调用 harness.run，验证主循环语义。
   - src/harness.rs:803-821（审计 #2）
10. CLI 入口 src/cli.rs:371 用 rt.block_on(agent.run(&task)) 触发主循环并消费 RunOutcome。
   - src/cli.rs:359-371（审计 #36）
11. tests/adversarial.rs:197 也调用 agent.run(...) 消费 RunOutcome。
   - tests/adversarial.rs:189-197（审计 #31）
12. 单测 fuse_after_five_no_progress_steps 与 tool_errors_count_toward_no_progress_fuse 验证 5 步熔断与工具错误计入熔断。
   - src/harness.rs:871-895（审计 #2）

## 死胡同
- grep 检索 "\.run\(" / "\brun\(" / "Harness::|Harness\s*\{" 在 Rust 项目里匹配过宽或被分词切碎，未能直接拿到除 cli.rs 与 src/harness.rs:803 之外的外部调用点；callers 图确认 Harness::run 只有 run_with 与 compaction_fires_when_window_exceeds_and_evicts 两个内部调用者，外部真实消费来自 src/cli.rs:371 与 tests/adversarial.rs:197。

## 置信度
high

## 统计
turns=11 · tool_calls=15 · duration=40394ms · tokens=139589


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
Harness 主循环（src/harness.rs:74-419）以单次任务字符串为输入，循环驱动「压缩判断 → LLM 决策 → 内置工具/外部工具/去重/合法性分支 → 增量记账 → submit_report 收敛 / prose 降级 / LLM_FUSE 熔断」三向终止。结构化 Report 与人类渲染 answer 在 RunOutcome 中一并返回（src/harness.rs:26-33, 205-211, 253-259），CLI 在 src/cli.rs:371 同步阻塞消费，审计与持久化落盘在 src/cli.rs:372-460。证据校验在 build_report（src/harness.rs:460-598）里以 EvidenceStore.cite_seq 互查；找不到的引用进入 uncited 并打回，连同零 findings 但会话有真实读取的退化一并回显拒绝原因与结构诊断（src/harness.rs:541-565, 620-651）。

## 证据列表
（无结构化发现——降级报告）

## 死胡同
- 初稿死胡同说「除 cli.rs 与 src/harness.rs:803 之外的外部调用点」未能拿到；本轮 grep ".run(" 实测确认还有 tests/replay.rs:86 与 tests/layered_live.rs:41 两处外部调用方，初稿此处不准确。
- grep 用 "agent\.run" 转义模式 0 命中，改用纯 plain 模式 "agent.run" 才匹配到——纯 grep 在仓库符号跨点时易因 dotall 误判，转义形式会因「agent.run」含 dot 而需 plain 模式。
- explore 返回的 Blast radius 把 src/main.rs:10 的 cli.run 标作「Harness::run 的 1 caller」——实际 main 是 Cli::run 的调用方，并非 Harness::run；这是 explore 把同名符号聚合呈现带来的误读，需以 path:line 区分。
- runtime 实际采用 tokio::runtime::Runtime（src/cli.rs:256-257, 371），非 async-std；Harness 自身是 async fn，CLI 在 run_task_inner 同步阻塞 rt.block_on 触发，跨边界这一点初稿未提，是补遗点。

## 置信度
high

## 统计
turns=13 · tool_calls=30 · duration=71878ms · tokens=495980


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
Harness::run（src/harness.rs:74-419）是 codesleuth Agent 的主循环：以单次任务字符串为输入，循环驱动 LLM 决策 → 只读工具调度 → 增量记账 → 通过 submit_report 收敛为带 evidence 校验的结构化 RunOutcome，或在连续无进展达 MAX_NO_PROGRESS_STREAK=5（src/harness.rs:19）时由 fuse_if_hit 返回 LLM_FUSE 熔断。证据校验在 build_report（src/harness.rs:460-598）里完成：findings 引用的 file 必须本会话真实观察到（cite_seq），否则连同零 findings 错误一并打回。

## 证据列表
1. Harness::run 是主循环入口，在 src/harness.rs:74-419 定义，构造 system+user 首消息并进入 loop。
   - src/harness.rs:74-99（审计 #2）
2. Harness 结构体持有 provider / tools / audit / evidence / model / context_tokens / compact_percent。
   - src/harness.rs:35-44（审计 #2）
3. 熔断阈值常量 MAX_NO_PROGRESS_STREAK = 5 在 src/harness.rs:19 定义。
   - src/harness.rs:18-19（审计 #2）
4. fuse_if_hit 在 streak >= MAX_NO_PROGRESS_STREAK 时返回 LLM_FUSE 错误并 audit.record("fuse")。
   - src/harness.rs:653-666（审计 #2）
5. 主循环每轮先按 context::should_compact 判断压缩，再 provider.chat 调 LLM。
   - src/harness.rs:100-148（审计 #2）
6. 工具调度按 submit_report / recall / 去重 / 工具存在性 / 工具执行分支处理；submit_report 走 build_report 校验 evidence 引用。
   - src/harness.rs:222-417（审计 #2）
7. 重复调用、非法 JSON、未知工具、零增量执行、工具错误均 no_progress += 1 并触发 fuse_if_hit。
   - src/harness.rs:300-416（审计 #2）
8. build_report 校验 evidence 引用的 file 必须 evidence.cite_seq 能查到（cite_seq 返回 None 计入 uncited 并返回 Err）。
   - src/harness.rs:460-598（审计 #2）
9. run_with 测试夹具用 Scripted provider + Harness::new 调用 harness.run，验证主循环语义。
   - src/harness.rs:803-821（审计 #2）
10. CLI 入口 src/cli.rs:371 用 rt.block_on(agent.run(&task)) 触发主循环并消费 RunOutcome。
   - src/cli.rs:359-371（审计 #36）
11. tests/adversarial.rs:197 也调用 agent.run(...) 消费 RunOutcome。
   - tests/adversarial.rs:189-197（审计 #31）
12. 单测 fuse_after_five_no_progress_steps 与 tool_errors_count_toward_no_progress_fuse 验证 5 步熔断与工具错误计入熔断。
   - src/harness.rs:871-895（审计 #2）

## 死胡同
- grep 检索 "\.run\(" / "\brun\(" / "Harness::|Harness\s*\{" 在 Rust 项目里匹配过宽或被分词切碎，未能直接拿到除 cli.rs 与 src/harness.rs:803 之外的外部调用点；callers 图确认 Harness::run 只有 run_with 与 compaction_fires_when_window_exceeds_and_evicts 两个内部调用者，外部真实消费来自 src/cli.rs:371 与 tests/adversarial.rs:197。

## 置信度
high

## 统计
turns=11 · tool_calls=15 · duration=40394ms · tokens=139589
