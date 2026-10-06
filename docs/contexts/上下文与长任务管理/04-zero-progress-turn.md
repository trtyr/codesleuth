# 零增量转向（上下文与长任务管理）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`04-zero-progress-turn-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 零增量转向（上下文与长任务管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
零增量转向是 harness 主循环中的防打转软机制：连续 2 回合工具执行无新信息增量时，向对话注入一条「无新信息」的用户消息，促使模型换工具/换角度或直接收敛，不终止运行。

## 证据列表
1. 功能价值：当模型连续执行但产出无新信息时注入转向提示，促其换策略或收敛，区别于 5 步无进展熔断（不终止运行）
   - src/harness.rs:393-398（审计 #2）
2. 入口：阈值常量 ZERO_GAIN_STEER_THRESHOLD=2 定义于 src/harness.rs:20-21；计数器 zero_gain_streak 在主循环中初始化（src/harness.rs:92）
   - src/harness.rs:20-21（审计 #2）
   - src/harness.rs:92（审计 #2）
3. 运作：主循环工具执行后用 info_keys 对 seen_keys 去重判定增量，无增量则 streak+1（harness.rs:379-388），达到阈值即注入 User 转向消息并归零计数（harness.rs:394-402）
   - src/harness.rs:379-388（审计 #2）
   - src/harness.rs:394-402（审计 #2）
4. 交互：注入消息与 system prompt 的「无空转」条款（src/prompt.rs:19-20）呼应；零增量步同时计入 no_progress 熔断计数并经 fuse_if_hit 检查（harness.rs:399-401）；测试 zero_gain_streak_injects_steering 固化行为（harness.rs:781-804）
   - src/prompt.rs:19-20（审计 #18）
   - src/harness.rs:399-401（审计 #2）
   - src/harness.rs:781-804（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=7 · tool_calls=6 · duration=27597ms · tokens=38645

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
零增量转向是 harness 主循环中的防打转软机制：普通工具成功执行后用 info_keys（路径 token + Sha256 指纹，evidence.rs:8-20）对会话级 seen_keys 去重判定增量，连续 2 次（ZERO_GAIN_STEER_THRESHOLD，编译期常量）零增量即注入「无新信息」User 转向消息并归零计数，不终止运行；同时每个零增量步计入 no_progress，由 fuse_if_hit 在 ≥5 步时以 CS2099 熔断兜底。该机制无任何配置项/环境变量开关；工具错误、重复调用、非法参数不计 zero_gain_streak（只计熔断计数）；recall 返回的新 info_keys 可重置两计数器。初稿行号全部核实无误，但初稿未点明注入点内嵌熔断检查——若模型无视转向，同一代码路径 3 步后即终止运行。

## 证据列表
1. 零增量判定落点：工具成功执行后 info_keys 对 seen_keys 去重，有增量归零两计数器，无增量则 no_progress 与 zero_gain_streak 各 +1
   - src/harness.rs:379-388（审计 #2）
2. 达到阈值 2 即注入「无新信息」User 转向消息、归零 zero_gain_streak，并立刻 fuse_if_hit(no_progress) 检查硬熔断——软机制内嵌硬熔断升级路径
   - src/harness.rs:393-402（审计 #2）
3. 阈值 ZERO_GAIN_STEER_THRESHOLD=2 与 MAX_NO_PROGRESS_STREAK=5 均为编译期 pub const，无配置项/环境变量开关
   - src/harness.rs:19-21（审计 #2）
4. 增量键语义：info_keys 提取路径形 token（含 / 或 .、≥3 字符、排除 ://）加全文 Sha256 前 8 字节指纹，任何输出差异即算增量
   - src/evidence.rs:8-24（审计 #27）
5. 边界：工具错误/重复调用/非法参数/未知工具只计 no_progress 不计 zero_gain_streak，转向只发生在成功执行路径
   - src/harness.rs:404-415（审计 #2）
6. recall 返回内容经 info_keys 计入 seen_keys，新 key 可重置 no_progress 与 zero_gain_streak（读回原文=有进展）
   - src/harness.rs:289-295（审计 #2）
7. 熔断兜底：fuse_if_hit 在 streak≥5 时审计记录 fuse 并返回 LLM_FUSE（CS2099）会话级错误
   - src/harness.rs:617-629（审计 #2）
8. 上游入口：run_task_inner 构造 Harness 并 block_on agent.run(&task)，Harness::new 参数含 model_context_tokens 与 compact_at_percent（间接影响压缩频率从而影响判定）
   - src/cli.rs:353-365（审计 #53）
9. system prompt「无空转」条款与注入的转向消息语义呼应，构成 prompt 级 + 循环级双重防打转
   - src/prompt.rs:19-20（审计 #29）
10. 测试固化：zero_gain_streak_injects_steering 验证第 4 次请求含转向消息且运行不终止
   - src/harness.rs:780-804（审计 #2）

## 死胡同
- grep 'ZERO_GAIN|Harness::new' 因 OR 正则按 plain 处理而 0 命中，拆分单 pattern 后成功
- 转向注入本身无独立 audit.record 事件（harness.rs:393-402 段确认），只能从审计序列间接推断（已标注为推断）

## 置信度
high

## 统计
turns=16 · tool_calls=24 · duration=106135ms · tokens=278777


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
零增量转向是 harness 主循环中的防打转软机制：连续 2 回合工具执行无新信息增量时，向对话注入一条「无新信息」的用户消息，促使模型换工具/换角度或直接收敛，不终止运行。

## 证据列表
1. 功能价值：当模型连续执行但产出无新信息时注入转向提示，促其换策略或收敛，区别于 5 步无进展熔断（不终止运行）
   - src/harness.rs:393-398（审计 #2）
2. 入口：阈值常量 ZERO_GAIN_STEER_THRESHOLD=2 定义于 src/harness.rs:20-21；计数器 zero_gain_streak 在主循环中初始化（src/harness.rs:92）
   - src/harness.rs:20-21（审计 #2）
   - src/harness.rs:92（审计 #2）
3. 运作：主循环工具执行后用 info_keys 对 seen_keys 去重判定增量，无增量则 streak+1（harness.rs:379-388），达到阈值即注入 User 转向消息并归零计数（harness.rs:394-402）
   - src/harness.rs:379-388（审计 #2）
   - src/harness.rs:394-402（审计 #2）
4. 交互：注入消息与 system prompt 的「无空转」条款（src/prompt.rs:19-20）呼应；零增量步同时计入 no_progress 熔断计数并经 fuse_if_hit 检查（harness.rs:399-401）；测试 zero_gain_streak_injects_steering 固化行为（harness.rs:781-804）
   - src/prompt.rs:19-20（审计 #18）
   - src/harness.rs:399-401（审计 #2）
   - src/harness.rs:781-804（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=7 · tool_calls=6 · duration=27597ms · tokens=38645
