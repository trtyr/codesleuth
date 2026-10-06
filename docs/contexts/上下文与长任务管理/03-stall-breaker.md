# 空转熔断（上下文与长任务管理）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`03-stall-breaker-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 空转熔断（上下文与长任务管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
空转熔断是 Agent 主循环的安全阀：连续 5 步无进展（重复调用、非法调用、零增量）即以会话级错误 LLM_FUSE（CS2099）终止 run()，防止模型无限空转烧钱。核心全在 src/harness.rs：阈值常量、fuse_if_hit 判定函数、run() 主循环多处检查点；新信息增量将 no_progress 归零。

## 证据列表
1. 阈值常量 MAX_NO_PROGRESS_STREAK=5；fuse_if_hit 在 streak≥5 时记录审计 "fuse" 并返回 LLM_FUSE 错误（CS2099，故障域非成本限制）
   - src/harness.rs:18-19（审计 #2）
   - src/harness.rs:616-625（审计 #2）
2. run() 主循环维护 no_progress 计数（第91行），在重复调用、参数非法 JSON、LLM 失败、工具错误等多处检查点 no_progress+=1 并调用 fuse_if_hit，命中即 return Err(fuse)
   - src/harness.rs:91（审计 #2）
   - src/harness.rs:304-307（审计 #2）
   - src/harness.rs:319-322（审计 #2）
   - src/harness.rs:226-227,262-263（审计 #2）
3. recall 结果产生新信息增量（info_keys 命中 seen_keys）时 no_progress 与 zero_gain_streak 归零
   - src/harness.rs:288-294（审计 #2）
4. 测试覆盖：5 步无进展触发 LLM_FUSE（exit_code=3）、工具错误调用也计入熔断
   - src/harness.rs:806-816（审计 #2）
   - src/harness.rs:818-824（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=7 · duration=12075ms · tokens=23991

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
空转熔断是 Agent 主循环（Harness::run）的安全阀：连续 5 个「无进展步」（重复同参调用、参数非法 JSON、未知工具、零增量执行、工具错误、报告校验被拒）即经 fuse_if_hit 记审计 "fuse" 并返回 LLM_FUSE（CS2099，按码段映射 exit 3）终止 run()；工具执行/recall 返回产生新信息增量（info_keys 新键入 seen_keys）时 no_progress 与 zero_gain_streak 双双归零。阈值 MAX_NO_PROGRESS_STREAK=5 与 ZERO_GAIN_STEER_THRESHOLD=2 均为硬编码常量，不受任何配置项/环境变量影响。初稿有一处错误：LLM 失败不计入熔断计数，而是直接 return Err 上抛。另发现一处初稿未提的不对称：recall 参数非法 JSON 不计 no_progress。

## 证据列表
1. 常量与判定函数：MAX_NO_PROGRESS_STREAK=5（pub const）与 ZERO_GAIN_STEER_THRESHOLD=2 定义于 src/harness.rs:19/21；fuse_if_hit 在 streak>=5 时记录审计 kind="fuse"（含 no_progress_streak 字段）、tracing::error 打日志（不受 -v 静默）并返回 Some(CsError(LLM_FUSE, ...))，否则 None
   - src/harness.rs:18-21（审计 #2）
   - src/harness.rs:616-629（审计 #2）
   - src/errors.rs:40（审计 #20）
   - src/errors.rs:13-22（审计 #20）
2. 计数检查点全集（no_progress 于 harness.rs:91 初始化为 0，每处 +1 后立即调 fuse_if_hit，命中即 return Err）：① submit_report 参数非 JSON(:226)；② submit_report 报告校验被拒(:262)；③ 同 canonical 调用重复(:304)；④ 普通工具参数非 JSON(:319)；⑤ 未知工具(:332)；⑥ 工具执行 Err(:407)；⑦ 零增量执行(:386)。重复判定用 canonical_call（name+重序列化 JSON 键序归一，harness.rs:638-643），seen_calls HashSet 去重
   - src/harness.rs:89-92（审计 #2）
   - src/harness.rs:226-229（审计 #2）
   - src/harness.rs:262-265（审计 #2）
   - src/harness.rs:302-307（审计 #2）
   - src/harness.rs:318-322（审计 #2）
   - src/harness.rs:331-335（审计 #2）
   - src/harness.rs:404-414（审计 #2）
3. 归零路径：工具成功输出经 evidence.rs info_keys 提取「路径形 token + 内容 SHA256 前 8 字节指纹」键集，新键插入 seen_keys 即 no_progress=0、zero_gain_streak=0（harness.rs:379-388）；recall 返回内容同样过 info_keys，有新键时归零（harness.rs:288-294）
   - src/harness.rs:379-388（审计 #2）
   - src/harness.rs:288-294（审计 #2）
   - src/evidence.rs:8-24（审计 #36）
4. 与打转转向的耦合：零增量执行同时累加 zero_gain_streak，达到阈值 2 时注入「无新信息」User 转向消息并在同一点位调 fuse_if_hit（此时 no_progress 已因该步 +1）——转向是软引导、熔断是硬熔断，二者共用 no_progress 计数；测试 zero_gain_streak_injects_steering 验证第 4 次请求含转向指令且不熔断（harness.rs:781-804）
   - src/harness.rs:393-402（审计 #2）
   - src/harness.rs:21（审计 #2）
   - src/harness.rs:781-804（审计 #2）
5. 不对称点（初稿未提）：recall 参数非 JSON 时只 reject_call 并 continue，不累加 no_progress（harness.rs:275-278）——坏模型理论上可无限以非法参数调 recall 不触熔断（推断：设计者视 recall 为幂等元操作，但严格说是防御缺口；最简验证：脚本化 provider 连发非法 recall 参数确认不熔断）
   - src/harness.rs:274-278（审计 #2）
6. 不计入熔断的路径（边界）：① LLM 调用失败：记审计 llm_error 后直接 return Err(e) 上抛，不走 fuse（harness.rs:138-147）——初稿说 LLM 失败计入熔断，实际不计入；② submit_report 与 recall 是直通分支，不走去重（harness.rs:223,273）；③ prose（无工具调用）走 prose_steer→降级路径，完全不涉及 no_progress（harness.rs:176-212）；④ 有工具调用的轮次将 prose_streak 归零(:220)
   - src/harness.rs:138-147（审计 #2）
   - src/harness.rs:223（审计 #2）
   - src/harness.rs:273（审计 #2）
   - src/harness.rs:176-212（审计 #2）
   - src/harness.rs:220（审计 #2）
7. 调用链：cli.rs run_task(:180) catch CsError 后 report_error + e.exit_code()；run_task_inner(:190) 装配 config/provider/registry/audit 后构造 Harness::new(:353-360) 并 rt.block_on(agent.run(&task))(:365)；熔断错误经 CsCode::exit_code 段位映射 2000..=2999→3（errors.rs:17），测试断言 exit_code()==3（harness.rs:815）
   - src/cli.rs:180-188（审计 #25）
   - src/cli.rs:353-365（审计 #25）
   - src/harness.rs:812-816（审计 #2）
   - src/errors.rs:17（审计 #20）
   - src/errors.rs:101-103（审计 #20）
8. 配置与开关：无。两个阈值均为编译期硬编码 pub const，grep 全仓无对应配置键；邻近的 context 配置（model_context_tokens=1_000_000、compact_at_percent=60，config.rs:115-118 及覆盖点 240-242）只影响压缩时机，与熔断无关；熔断不可通过配置关闭或调整
   - src/harness.rs:19（审计 #2）
   - src/harness.rs:21（审计 #2）
   - src/config.rs:115-118（审计 #48）
   - src/config.rs:240-242（审计 #48）
9. 测试覆盖：fuse_after_five_no_progress_steps 用 5 次未知工具调用（"nope"）断言 code==LLM_FUSE 且 exit_code==3（harness.rs:806-816）；tool_errors_count_toward_no_progress_fuse 用 AlwaysErr 工具连发千变参数断言熔断，注释标明这是 P004 T2.1 回归——旧实现工具错误不计数可无限空转（harness.rs:818-830）
   - src/harness.rs:806-816（审计 #2）
   - src/harness.rs:818-830（审计 #2）
   - src/harness.rs:820（审计 #2）
10. 交互契约/数据流下游：熔断落点一为审计账本（Audit::record "fuse" 行，JSONL 审计文件供 recall 钻取与归因），二为 stderr 的 tracing::error，三为进程 exit code 3 供脚本判别；上游输入是 LLM 返回的 tool_calls 数组与工具输出文本（info_keys 指纹），共享内存态为 seen_calls/seen_keys 两个 HashSet 与 EvidenceStore（paths→首观察 seq，供 submit_report 证据校验，evidence.rs:26-30）；recall 报告确认「返回内容经 info_keys 计入 seen_keys 可重置 no_progress/zero_gain_streak」
   - src/harness.rs:619（审计 #2）
   - src/harness.rs:604-607（审计 #2）
   - docs/contexts/上下文与长任务管理/02-tool-recall.md:243（审计 #52）
   - src/evidence.rs:26-30（审计 #36）

## 死胡同
- grep MAX_NO_PROGRESS|ZERO_GAIN（无下划线完整前缀写法）0 命中，需用完整常量名才能命中
- grep LLM_FUSE|CS2099|fuse plain 模式 0 命中（| 被当字面量），换分开的 plain grep 后命中——初稿已预判此坑

## 置信度
high

## 统计
turns=11 · tool_calls=21 · duration=87488ms · tokens=246129


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
空转熔断是 Agent 主循环的安全阀：连续 5 步无进展（重复调用、非法调用、零增量）即以会话级错误 LLM_FUSE（CS2099）终止 run()，防止模型无限空转烧钱。核心全在 src/harness.rs：阈值常量、fuse_if_hit 判定函数、run() 主循环多处检查点；新信息增量将 no_progress 归零。

## 证据列表
1. 阈值常量 MAX_NO_PROGRESS_STREAK=5；fuse_if_hit 在 streak≥5 时记录审计 "fuse" 并返回 LLM_FUSE 错误（CS2099，故障域非成本限制）
   - src/harness.rs:18-19（审计 #2）
   - src/harness.rs:616-625（审计 #2）
2. run() 主循环维护 no_progress 计数（第91行），在重复调用、参数非法 JSON、LLM 失败、工具错误等多处检查点 no_progress+=1 并调用 fuse_if_hit，命中即 return Err(fuse)
   - src/harness.rs:91（审计 #2）
   - src/harness.rs:304-307（审计 #2）
   - src/harness.rs:319-322（审计 #2）
   - src/harness.rs:226-227,262-263（审计 #2）
3. recall 结果产生新信息增量（info_keys 命中 seen_keys）时 no_progress 与 zero_gain_streak 归零
   - src/harness.rs:288-294（审计 #2）
4. 测试覆盖：5 步无进展触发 LLM_FUSE（exit_code=3）、工具错误调用也计入熔断
   - src/harness.rs:806-816（审计 #2）
   - src/harness.rs:818-824（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=7 · duration=12075ms · tokens=23991
