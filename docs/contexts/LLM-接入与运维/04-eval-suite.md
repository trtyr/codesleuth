# 多语言 fixture 评测（LLM 接入与运维）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`04-eval-suite-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 多语言 fixture 评测（LLM 接入与运维）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
多语言 fixture 评测是 scripts/eval 下的正式评测系统：以 Rust/TS/Python 三个 fixture 仓为被测对象，对 codesleuth 二进制的侦察报告做「硬门规则校验 + LLM judge 三维打分」，产出 scorecard 并支持 baseline 回归比对，作为质量门禁。入口为 run_eval.py（--suite 可重复指定 GOLDEN=FIXTURE_DIR），每题子进程调用被测二进制（run_one_question），经 hardgate 硬门、judge（三维 1-5 分、temperature=0）判分，最后 scorecard 汇总并和 baseline 对比、支持退出码供 CI 使用。上游依赖 golden 题库与 fixtures、~/.codesleuth/config.toml 的 LLM 配置；下游消费为 scorecard/baseline 机制与 README/AGENTS 文档中的 CI 工作流。

## 证据列表
1. 评测以多语言 fixture 仓为对象，验证 codesleuth 报告质量：题库含 fixture-rs/ts/py（AGENTS.md 示例一次跑三个 suite；fixture-rs.json 有 16 题，考察文件定位、事实核对与死代码诚实回答）
   - scripts/eval/run_eval.py:1-14（审计 #2）
   - tests/golden/fixture-rs.json:1-20（审计 #7）
   - AGENTS.md:30-35（审计 #15）
2. 入口是 scripts/eval/run_eval.py 的 main，--suite 重复传入 GOLDEN=FIXTURE_DIR 对；每题通过 run_one_question 子进程运行被测二进制 codesleuth-bin 并解析 JSON 报告与审计路径
   - scripts/eval/run_eval.py:121-133（审计 #2）
   - scripts/eval/run_eval.py:34-52（审计 #2）
3. 调用链：main → eval_suite（run_eval.py:55）→ 每题先 hardgate.check_hard_gate 硬门（run_eval.py:75），通过后调 judge.judge_report 三维打分（run_eval.py:83-85；judge 维度为 evidence_grounded/depth/honesty，1-5 分，temperature=0，失败降级为 judge_degraded）
   - scripts/eval/run_eval.py:55-107（审计 #2）
   - scripts/eval/judge.py:1-5（审计 #4）
4. 判分结果汇入 scorecard.build_scorecard，与 load_baseline 对比并保存 scorecard，--set-baseline 可设基线（run_eval.py:176-193）；README.md:130 将其标注为「正式 eval：多语言 fixture × 硬门 + judge 判分」
   - scripts/eval/run_eval.py:176-193（审计 #2）
   - README.md:130（审计 #10）
5. 上游依赖：golden 题库 JSON、tests/fixtures/ 各语言 fixture 目录，以及 ~/.codesleuth/config.toml 的 [llm] api_key/base_url/model（judge 端点配置）
   - scripts/eval/run_eval.py:110-118（审计 #2）
   - scripts/eval/run_eval.py:154-165（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=6 · duration=18769ms · tokens=35703


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
多语言 fixture 评测是 scripts/eval 下的独立 Python 系统：run_eval.py main（--suite 可重复 GOLDEN=FIXTURE_DIR）→ eval_suite 逐题 subprocess 调 codesleuth 二进制（run_one_question，timeout 3600s）→ hardgate 四规则一票否决（G1 gold 命中/G2 audit_seq 可回溯/G3 schema/G4 非 degraded）→ judge 三维 1-5 分（temperature=0，失败静默降级 judge_degraded）→ scorecard 汇总 + baseline 对比（hard_gate_rate 下降或 judge 维降>0.5 即回归）→ 写 ~/.codesleuth/eval/scorecard-*.json 与 baseline.json 指针，退出码 0/1/5。配置只读 ~/.codesleuth/config.toml [llm]（env 层已移除）。重要限定：ci.yml 并未接入 eval（只跑 cargo 三连），初稿「作为质量门禁」实为手动/文档层门禁。坑：subprocess timeout 异常未捕获会整轮崩、baseline 指针失效静默转 auto-first、审计 stderr 格式耦合。

## 证据列表
1. 调用链：main（argparse --suite/--no-judge/--set-baseline/--extra-args 等）→ eval_suite → run_one_question 子进程调被测二进制（--repo fixture --json），timeout=3600s
   - scripts/eval/run_eval.py:121-133（审计 #2）
   - scripts/eval/run_eval.py:34-52（审计 #2）
   - scripts/eval/run_eval.py:31（审计 #2）
2. 硬门四规则一票否决；审计行从被测进程 stderr 的『# 审计:』正则抽取，load_audit_rows 读 JSONL 建 {seq:row}
   - scripts/eval/run_eval.py:30,50-51（审计 #2）
   - scripts/eval/hardgate.py:26-66（审计 #9）
   - scripts/eval/hardgate.py:69-84（审计 #9）
3. judge 三维（evidence_grounded/depth/honesty）1-5 分，temperature=0，POST {base_url}/chat/completions，60s 超时，失败返回 None→judge_degraded 不阻断硬门
   - scripts/eval/judge.py:23,41-70,78,82-102（审计 #11）
   - scripts/eval/run_eval.py:83-91（审计 #2）
4. scorecard 汇总 hard_gate_rate/judge_avg，baseline.json 指针机制，回归判定（hard_gate_rate 下降或 judge 维降>0.5）→ EXIT_REGRESSION=5
   - scripts/eval/scorecard.py:14,31-52,55-74,88-94,102-121（审计 #13）
   - scripts/eval/run_eval.py:184-206（审计 #2）
5. 配置契约：只读 ~/.codesleuth/config.toml [llm]（api_key/base_url 必需否则 EXIT_USAGE，model 默认 gpt-4o-mini），环境变量层已移除；--codesleuth-bin 默认 ./target/debug/codesleuth
   - scripts/eval/run_eval.py:110-118,150-168（审计 #2）
   - scripts/eval/run_eval.py:13（审计 #2）
6. 题库契约：每题 q + expect_files + 可选 expect_facts + fact_ref（强制事实出处，AGENTS.md 约定 gold 只能从 fixture 源码导出）
   - tests/golden/fixture-rs.json:4-18（审计 #18）
   - AGENTS.md:50（审计 #30）
7. 边界：CI 未接入 eval——.github/workflows/ci.yml 只跑 cargo fmt/clippy/test；eval 门禁目前靠 README 徽章与 AGENTS.md 手动流程（初稿说『作为质量门禁』，实际是本地/手动门禁，CI 集成未落地）
   - .github/workflows/ci.yml:14-16（审计 #16）
   - README.md:10,130（审计 #23）
   - AGENTS.md:28-35（审计 #30）
8. 坑：run_one_question 的 subprocess timeout 抛 TimeoutExpired 未捕获（会整轮崩而非记一题失败）；baseline 指针失效静默转 auto-first 回归检测失效；judge 全挂时 judge_avg=null 但跑通；base_url 为空曾致 judge 首题崩（P005 已修）
   - scripts/eval/run_eval.py:37-42（审计 #2）
   - scripts/eval/run_eval.py:184-188（审计 #2）
   - scripts/eval/scorecard.py:109-114（审计 #13）
   - scripts/eval/run_eval.py:160-161（审计 #2）
9. 离线单测面：test_judge/test_hardgate/test_scorecard 全无网络，经 AGENTS.md 的 unittest discover 入口；scorecard 状态目录与 Rust global_state_dir 对齐 ~/.codesleuth
   - scripts/eval/test_judge.py:1-25（审计 #32）
   - AGENTS.md:25（审计 #30）
   - scripts/eval/scorecard.py:22-24（审计 #13）

## 死胡同
- grep "llm|api_key|base_url" plain 模式 0 命中（正则管道被当 plain），改用 grep api_key 命中配置文档
- docs/plantree/ 规划文档被 .gitignore 排除，无法直接读 scoring-spec 原文（AGENTS.md:42 明示不随仓分发）

## 置信度
high

## 统计
turns=6 · tool_calls=14 · duration=125488ms · tokens=95019


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
多语言 fixture 评测是 scripts/eval 下的正式评测系统：以 Rust/TS/Python 三个 fixture 仓为被测对象，对 codesleuth 二进制的侦察报告做「硬门规则校验 + LLM judge 三维打分」，产出 scorecard 并支持 baseline 回归比对，作为质量门禁。入口为 run_eval.py（--suite 可重复指定 GOLDEN=FIXTURE_DIR），每题子进程调用被测二进制（run_one_question），经 hardgate 硬门、judge（三维 1-5 分、temperature=0）判分，最后 scorecard 汇总并和 baseline 对比、支持退出码供 CI 使用。上游依赖 golden 题库与 fixtures、~/.codesleuth/config.toml 的 LLM 配置；下游消费为 scorecard/baseline 机制与 README/AGENTS 文档中的 CI 工作流。

## 证据列表
1. 评测以多语言 fixture 仓为对象，验证 codesleuth 报告质量：题库含 fixture-rs/ts/py（AGENTS.md 示例一次跑三个 suite；fixture-rs.json 有 16 题，考察文件定位、事实核对与死代码诚实回答）
   - scripts/eval/run_eval.py:1-14（审计 #2）
   - tests/golden/fixture-rs.json:1-20（审计 #7）
   - AGENTS.md:30-35（审计 #15）
2. 入口是 scripts/eval/run_eval.py 的 main，--suite 重复传入 GOLDEN=FIXTURE_DIR 对；每题通过 run_one_question 子进程运行被测二进制 codesleuth-bin 并解析 JSON 报告与审计路径
   - scripts/eval/run_eval.py:121-133（审计 #2）
   - scripts/eval/run_eval.py:34-52（审计 #2）
3. 调用链：main → eval_suite（run_eval.py:55）→ 每题先 hardgate.check_hard_gate 硬门（run_eval.py:75），通过后调 judge.judge_report 三维打分（run_eval.py:83-85；judge 维度为 evidence_grounded/depth/honesty，1-5 分，temperature=0，失败降级为 judge_degraded）
   - scripts/eval/run_eval.py:55-107（审计 #2）
   - scripts/eval/judge.py:1-5（审计 #4）
4. 判分结果汇入 scorecard.build_scorecard，与 load_baseline 对比并保存 scorecard，--set-baseline 可设基线（run_eval.py:176-193）；README.md:130 将其标注为「正式 eval：多语言 fixture × 硬门 + judge 判分」
   - scripts/eval/run_eval.py:176-193（审计 #2）
   - README.md:130（审计 #10）
5. 上游依赖：golden 题库 JSON、tests/fixtures/ 各语言 fixture 目录，以及 ~/.codesleuth/config.toml 的 [llm] api_key/base_url/model（judge 端点配置）
   - scripts/eval/run_eval.py:110-118（审计 #2）
   - scripts/eval/run_eval.py:154-165（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=6 · duration=18769ms · tokens=35703
