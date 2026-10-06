# repo map预算注入（向量检索）

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
# repo map预算注入（向量检索）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
repo map 预算注入：从 codegraph SQLite 读符号表与边表度数，按默认 24_000 字符预算做度数中心度贪心装填，生成结构化导航文本并以 `[repo map]` 段包裹，作为首条 User 消息后缀一次性注入（harness.rs:83-88），让 agent 进门即可参照图结构、避免对图工具的盲查。实际实现按 D005 分层（harness.rs:67-68）走 user 消息而非 system——这是与 repomap.rs:1-4 头部注释/任务书措辞的差异点，以代码为准。

## 证据列表
1. 默认字符预算为 24_000（≈6k token），并为单符号行设置 200 字符上限以防长名吃光预算。
   - src/vector/repomap.rs:9-10（审计 #2）
   - src/vector/repomap.rs:11-12（审计 #2）
2. repo_map_inputs 从 codegraph SQLite 只读打开，SELECT 限定 kind 为 function/method/struct/class/interface/impl/trait 七类节点，并从 edges 表累加 source/target 两侧度数。
   - src/vector/repomap.rs:22-65（审计 #2）
3. build_task_map 与 build_repo_map 都是预算内贪心装填；build_repo_map 排序键为「度数降序 → 名称升序 → 文件路径升序」，build_task_map 优先召回命中种子、再邻居、最后按度数填充。
   - src/vector/repomap.rs:137-178（审计 #2）
   - src/vector/repomap.rs:67-135（审计 #2）
   - src/vector/repomap.rs:200-229（审计 #2）
4. wrap_repo_section 把地图包成 "[repo map]\n{map}\n结构详情勿凭此图推断，用 explore / callers / callees 查证。" 段，固定前缀与查证后缀由 unit test 验证。
   - src/vector/repomap.rs:180-183（审计 #2）
   - src/vector/repomap.rs:290-294（审计 #2）
5. 单元测试覆盖了三个核心不变量：度数降序排序、预算封顶后追加「其余未列入」溢出提示、小仓全图无溢出提示。
   - src/vector/repomap.rs:246-262（审计 #2）
   - src/vector/repomap.rs:264-279（审计 #2）
   - src/vector/repomap.rs:281-287（审计 #2）
6. 调用方在 src/cli.rs 的两条路径使用 repomap：向量层任务相关路径（setup_vector_layer 内部，625-653 行）走 build_task_map + wrap_repo_section；--repo-map 无向量路径（340-346 行）走 build_repo_map + wrap_repo_section。
   - src/cli.rs:340-346（审计 #15）
   - src/cli.rs:619-658（审计 #15）
   - src/cli.rs:540-545（审计 #15）
7. 主流程在 src/cli.rs:306 进入 setup_vector_layer，最终通过 src/cli.rs:368 的 with_first_user_suffix 把段交给 Harness。
   - src/cli.rs:304-314（审计 #15）
   - src/cli.rs:367-371（审计 #15）
8. 实际注入位置是首条 User 消息后缀而非 system：harness.rs:74-88 在 messages 构造完后取 last_mut 把 "\n\n" + suffix 追加到 User 消息 content；system 消息保持 crate::prompt::SYSTEM_PROMPT 恒定。
   - src/harness.rs:67-72（审计 #39）
   - src/harness.rs:74-88（审计 #39）
   - src/harness.rs:42-43（审计 #39）
9. 预算由 config.vector.repomap_budget 提供，默认 24_000，可被配置文件（FileConfig merge）覆写，并暴露在 config key 表中支持 CLI get/set；README 文档值也是 24000。
   - src/config.rs:99（审计 #51）
   - src/config.rs:137（审计 #51）
   - src/config.rs:274-275（审计 #51）
   - src/config.rs:437-444（审计 #51）
   - README.md:70（审计 #59）
10. 任务路径下 seeds 由召回 hits 转 symbols 组成，neighbors 由 vector::chunk::relations_for_symbol 返回的 callers/callees 合并去重得到（fallback/leftover 命中跳过）。
   - src/cli.rs:627-646（审计 #15）
11. 导航图构建失败走非致命降级：tracing::warn 记录后向 audit 写 "degraded" 事件并跳过注入，不阻断主流程。
   - src/cli.rs:348-356（审计 #15）
   - src/cli.rs:655-657（审计 #15）
12. 无符号索引的诚实工具面：cli.rs:282-285 在 codegraph 无符号时不注册图工具并把 first_suffix 留给地形提示，避免 agent 反复空查询图工具。
   - src/cli.rs:279-294（审计 #15）

## 死胡同
- grep 模式 "build_repo_map|build_task_map|repo_map_inputs|DEFAULT_BUDGET_CHARS" 用 plain 模式命中 0 条，换 regex 才命中——plain 模式的 | 不是逻辑或
- 未直接读到 build_task_map 在 setup_vector_layer 中被调用的完整 cfg/registry 装载上下文，但调用串联已通过 src/cli.rs:625-653 read 直接证实
- src/vector/chunk.rs 中 relations_for_symbol 的实现未读，凭 src/cli.rs:638 的调用点推断其返回 (callers, callees)

## 置信度
high

## 统计
turns=13 · tool_calls=23 · duration=74554ms · tokens=158678


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
repo map 预算注入：从 codegraph SQLite 读符号表与边表度数，按默认 24_000 字符预算做度数中心度贪心装填，生成结构化导航文本并以 `[repo map]` 段包裹，作为首条 User 消息后缀一次性注入（harness.rs:83-88），让 agent 进门即可参照图结构、避免对图工具的盲查。实际实现按 D005 分层（harness.rs:67-68）走 user 消息而非 system——这是与 repomap.rs:1-4 头部注释/任务书措辞的差异点，以代码为准。

初稿说 repomap.rs:180 注释写「system prompt 注入段」,实际是 harness.rs:83-88 走 user 消息后缀（repomap.rs:1-4 头部三行注释同样写「system prompt 尾部 [repo map] 段」,与代码不符）。

## 证据列表
1. 默认字符预算为 24_000（≈6k token），并为单符号行设置 200 字符上限以防长名吃光预算。
   - src/vector/repomap.rs:9-10（审计 #2）
   - src/vector/repomap.rs:11-12（审计 #2）
2. repo_map_inputs 从 codegraph SQLite 只读打开，SELECT 限定 kind 为 function/method/struct/class/interface/impl/trait 七类节点，并从 edges 表累加 source/target 两侧度数。
   - src/vector/repomap.rs:22-65（审计 #2）
3. build_task_map 排序键为「seeds 索引升序 → 邻居包含性降序 → 度数降序 → 名称升序 → 文件路径升序」；build_repo_map 排序键为「度数降序 → 名称升序 → 文件路径升序」；两者都是预算内贪心装填，超出封顶时追加「…共 N 符号，其余未列入」溢出提示；空输入时 build_repo_map 兜底输出「（仓库无符号索引）」。
   - src/vector/repomap.rs:67-135（审计 #2）
   - src/vector/repomap.rs:137-178（审计 #2）
4. wrap_repo_section 把地图包成固定 "[repo map]\n{map}\n结构详情勿凭此图推断，用 explore / callers / callees 查证。" 段；模块内函数注释 180 行仍写「system prompt 注入段」与实际 user-后缀注入位置不符。
   - src/vector/repomap.rs:180-183（审计 #2）
   - src/vector/repomap.rs:1-4（审计 #2）
5. 单元测试覆盖：seeds/neighbors/度数三段排序、build_task_map 空 seeds 等价于 build_repo_map、度数降序排序、预算封顶后追加「其余未列入」、小仓无溢出、wrap 段含 explore 提示、对真实 fixture-rs codegraph 端到端读。
   - src/vector/repomap.rs:200-244（审计 #2）
   - src/vector/repomap.rs:246-262（审计 #2）
   - src/vector/repomap.rs:264-287（审计 #2）
   - src/vector/repomap.rs:289-294（审计 #2）
   - src/vector/repomap.rs:296-307（审计 #2）
6. setup_vector_layer 在 repo_map 旗标为真时串行组装：recall(task,10) 失败 best-effort 写 audit "degraded" 继续；hits 非空先 format_recall_block 当 part[0]；再读 codegraph 拿 (symbols, degrees) → 遍历 hits（跳过 fallback/leftover）→ relations_for_symbol 拼 neighbors、c.symbol 去重入 seeds → build_task_map 拼接 wrap_repo_section 当 part[1]；parts.join("\n\n") 整体作为 Some(suffix) 返回。
   - src/cli.rs:619-665（审计 #4）
   - src/cli.rs:627-646（审计 #4）
7. 无 --vector 但有 --repo-map 时，setup_vector_layer 不被调用，由 run_task_inner 兜底：直接 repo_map_inputs → build_repo_map → wrap_repo_section → first_suffix；codegraph 读失败 warn 并 best-effort 写 audit "degraded" 跳过注入。
   - src/cli.rs:340-357（审计 #4）
8. codegraph 节点数 = 0 时（QA FINDING-009 诚实工具面）：不注册 explore/callers/callees/impact/files 五件图工具；first_suffix 留空给「〔地形提示〕…explore/callers/callees/impact 不可用」片段。
   - src/cli.rs:281-302（审计 #4）
9. 实际注入位置是首条 User 消息后缀而非 system：run 入口处构造 messages = [System{crate::prompt::SYSTEM_PROMPT}, User{task}]，随后 last_mut 把 "\n\n" + suffix 拼到 User.content（harness.rs:75-88）。with_first_user_suffix 是 builder 入口（harness.rs:69-72）。
   - src/harness.rs:67-72（审计 #12）
   - src/harness.rs:74-88（审计 #12）
10. 系统提示固定为 crate::prompt::SYSTEM_PROMPT，不受本功能影响；user 后缀是任务派生数据，按 D005 分层。
   - src/harness.rs:75-82（审计 #12）
11. VectorConfig.repomap_budget 字段默认 24_000，配置文件以 FileVector.repomap_budget: Option<usize> 形态覆盖（merge_file 仅在 Some 时覆写）。
   - src/config.rs:36（审计 #14）
   - src/config.rs:99（审计 #14）
   - src/config.rs:133-140（审计 #14）
   - src/config.rs:274-275（审计 #14）
12. 该键已注册到 config_key_table 供 codesleuth config get/set 操作；CLI 旗标层未单独支持（merge_cli 不读 vector.*）。
   - src/config.rs:437-444（审计 #14）
   - src/config.rs:303-310（审计 #14）
13. README 文档值也是 repomap_budget = 24000（与默认值一致，无独立示例）。
   - README.md:70（审计 #25）
14. relations_for_symbol 真实实现（src/vector/chunk.rs:84-133）：以 (file_path, name) 唯一确定节点 id；callers = "WHERE target=?1 JOIN source→name DISTINCT LIMIT 8"；callees = "WHERE source=?1 JOIN target→name DISTINCT LIMIT 8"；找不到节点返空 Vec；出错返 CsError(INDEX_NOT_AVAILABLE)。调用方 cli.rs:638 用 .unwrap_or_default() 兜底为 (Vec::new(), Vec::new())。
   - src/vector/chunk.rs:83-133（审计 #4）
   - src/cli.rs:637-639（审计 #4）
15. format_recall_block 渲染为「[语义召回 · 起步线索]…(召回内容未经验证…)」段，固定头/固定尾；与 wrap_repo_section 的「[repo map]…查证。」段用 "\n\n" 拼接为完整 user 后缀。
   - src/vector/recall.rs:141-159（审计 #44）
   - src/cli.rs:660-664（审计 #4）

## 死胡同
- grep 模式 "build_repo_map|build_task_map|repo_map_inputs|DEFAULT_BUDGET_CHARS" 用 plain 模式命中 0 条，换 regex 才命中——plain 模式的 | 不是逻辑或
- relations_for_symbol 实现已读清（src/vector/chunk.rs:84-133），确认返回 (Vec<String>, Vec<String>)= (callers, callees)，与初稿推断一致；不再列为死胡同。

## 置信度
high

## 统计
turns=8 · tool_calls=19 · duration=56659ms · tokens=207095


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
repo map 预算注入：从 codegraph SQLite 读符号表与边表度数，按默认 24_000 字符预算做度数中心度贪心装填，生成结构化导航文本并以 `[repo map]` 段包裹，作为首条 User 消息后缀一次性注入（harness.rs:83-88），让 agent 进门即可参照图结构、避免对图工具的盲查。实际实现按 D005 分层（harness.rs:67-68）走 user 消息而非 system——这是与 repomap.rs:1-4 头部注释/任务书措辞的差异点，以代码为准。

## 证据列表
1. 默认字符预算为 24_000（≈6k token），并为单符号行设置 200 字符上限以防长名吃光预算。
   - src/vector/repomap.rs:9-10（审计 #2）
   - src/vector/repomap.rs:11-12（审计 #2）
2. repo_map_inputs 从 codegraph SQLite 只读打开，SELECT 限定 kind 为 function/method/struct/class/interface/impl/trait 七类节点，并从 edges 表累加 source/target 两侧度数。
   - src/vector/repomap.rs:22-65（审计 #2）
3. build_task_map 与 build_repo_map 都是预算内贪心装填；build_repo_map 排序键为「度数降序 → 名称升序 → 文件路径升序」，build_task_map 优先召回命中种子、再邻居、最后按度数填充。
   - src/vector/repomap.rs:137-178（审计 #2）
   - src/vector/repomap.rs:67-135（审计 #2）
   - src/vector/repomap.rs:200-229（审计 #2）
4. wrap_repo_section 把地图包成 "[repo map]\n{map}\n结构详情勿凭此图推断，用 explore / callers / callees 查证。" 段，固定前缀与查证后缀由 unit test 验证。
   - src/vector/repomap.rs:180-183（审计 #2）
   - src/vector/repomap.rs:290-294（审计 #2）
5. 单元测试覆盖了三个核心不变量：度数降序排序、预算封顶后追加「其余未列入」溢出提示、小仓全图无溢出提示。
   - src/vector/repomap.rs:246-262（审计 #2）
   - src/vector/repomap.rs:264-279（审计 #2）
   - src/vector/repomap.rs:281-287（审计 #2）
6. 调用方在 src/cli.rs 的两条路径使用 repomap：向量层任务相关路径（setup_vector_layer 内部，625-653 行）走 build_task_map + wrap_repo_section；--repo-map 无向量路径（340-346 行）走 build_repo_map + wrap_repo_section。
   - src/cli.rs:340-346（审计 #15）
   - src/cli.rs:619-658（审计 #15）
   - src/cli.rs:540-545（审计 #15）
7. 主流程在 src/cli.rs:306 进入 setup_vector_layer，最终通过 src/cli.rs:368 的 with_first_user_suffix 把段交给 Harness。
   - src/cli.rs:304-314（审计 #15）
   - src/cli.rs:367-371（审计 #15）
8. 实际注入位置是首条 User 消息后缀而非 system：harness.rs:74-88 在 messages 构造完后取 last_mut 把 "\n\n" + suffix 追加到 User 消息 content；system 消息保持 crate::prompt::SYSTEM_PROMPT 恒定。
   - src/harness.rs:67-72（审计 #39）
   - src/harness.rs:74-88（审计 #39）
   - src/harness.rs:42-43（审计 #39）
9. 预算由 config.vector.repomap_budget 提供，默认 24_000，可被配置文件（FileConfig merge）覆写，并暴露在 config key 表中支持 CLI get/set；README 文档值也是 24000。
   - src/config.rs:99（审计 #51）
   - src/config.rs:137（审计 #51）
   - src/config.rs:274-275（审计 #51）
   - src/config.rs:437-444（审计 #51）
   - README.md:70（审计 #59）
10. 任务路径下 seeds 由召回 hits 转 symbols 组成，neighbors 由 vector::chunk::relations_for_symbol 返回的 callers/callees 合并去重得到（fallback/leftover 命中跳过）。
   - src/cli.rs:627-646（审计 #15）
11. 导航图构建失败走非致命降级：tracing::warn 记录后向 audit 写 "degraded" 事件并跳过注入，不阻断主流程。
   - src/cli.rs:348-356（审计 #15）
   - src/cli.rs:655-657（审计 #15）
12. 无符号索引的诚实工具面：cli.rs:282-285 在 codegraph 无符号时不注册图工具并把 first_suffix 留给地形提示，避免 agent 反复空查询图工具。
   - src/cli.rs:279-294（审计 #15）

## 死胡同
- grep 模式 "build_repo_map|build_task_map|repo_map_inputs|DEFAULT_BUDGET_CHARS" 用 plain 模式命中 0 条，换 regex 才命中——plain 模式的 | 不是逻辑或
- 未直接读到 build_task_map 在 setup_vector_layer 中被调用的完整 cfg/registry 装载上下文，但调用串联已通过 src/cli.rs:625-653 read 直接证实
- src/vector/chunk.rs 中 relations_for_symbol 的实现未读，凭 src/cli.rs:638 的调用点推断其返回 (callers, callees)

## 置信度
high

## 统计
turns=13 · tool_calls=23 · duration=74554ms · tokens=158678
