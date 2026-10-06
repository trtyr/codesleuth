# 零写入自证（只读边界与安全）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`02-write-guard-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# 零写入自证（只读边界与安全）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

零写入自证：任务前后对目标仓库做 per-file manifest 快照比对，证明 codesleuth 全程只读；凡差异（除 .codegraph/.codesleuth 等工具元数据豁免目录）一律如实归因为「不可归因变更」，写入审计并打印 stderr 摘要，给调用方可审计的零写入保证。

## 证据列表

1. 功能价值：证明 codesleuth 在目标仓内的可写面为空集，任何差异如实上报（不可归因变更），不冒领不瞒报
    - src/writeguard.rs:1-4（审计 #2）
2. 关键文件 src/writeguard.rs：snapshot 生成 BTreeMap<路径,sha256> manifest，diff 检测增/删/改三类变更
    - src/writeguard.rs:26-39（审计 #2）
    - src/writeguard.rs:117-142（审计 #2）
3. 调用链：cli.rs 任务前 snapshot 考前快照 → agent.run 任务运行 → 任务后再 snapshot + diff，写审计 write_check 行并输出 stderr 摘要
    - src/cli.rs:246-247（审计 #7）
    - src/cli.rs:369-389（审计 #7）
4. 交互：快照豁免 SKIP_DIRS（.git/.codegraph/.codesleuth/target 等），与 vector::store 的项目索引目录 .codesleuth、bootlock 锁文件共用同一豁免约定（D011 工具元数据）；errors 模块提供 REPO_NOT_READABLE
    - src/writeguard.rs:11-20（审计 #2）
    - src/vector/store.rs:305-307（审计 #12）
    - src/bootlock.rs:12-13（审计 #17）

## 死胡同

无

## 置信度

high

## 统计

turns=7 · tool_calls=5 · duration=26444ms · tokens=55536

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
零写入自证（D1）由 src/writeguard.rs 实现核心（snapshot/diff），由 src/cli.rs 的 run_task_inner 在任务前后各做一次 per-file manifest 快照并 diff：考前快照 src/cli.rs:247（.ok() 失败即放弃自证），任务运行 src/cli.rs:365（agent.run），考后 src/cli.rs:370-421 生成 write_check 审计行（含 files_snapshotted / unattributed_changes / samples≤5）写 ~/.codesleuth/audit/{sid}.jsonl，stderr 打印摘要；有变更=完整性违规 ERROR。快照遍历时豁免 SKIP_DIRS（.git/.codegraph/.codesleuth/target/node_modules/dist/__pycache__/.venv，src/writeguard.rs:11-20）、跳过符号链接（src/writeguard.rs:53-61）、>1MB 大文件用「大小+首64KB」指纹（src/writeguard.rs:80-93）；无任何配置项/环境变量开关，行为完全硬编码。

## 证据列表
1. 完整调用链：CLI 入口 run_cli → run_task(:180) → run_task_inner(:190)：canonicalize repo(:203)、config::load(:216)、Audit::create(:229)、考前 writeguard::snapshot(:247)、注册工具+CodegraphEngine 启动(:252)、向量层(:300)、Harness::new(:353)、rt.block_on(agent.run)(:365)、drop cg(:367)、考后 snapshot+diff+append_line+stderr(:370-421)、报告持久化(:428-441)。
   - src/cli.rs:190-247（审计 #4）
   - src/cli.rs:353-421（审计 #4）
2. snapshot 实现：入口校验 root.is_dir() 否则 REPO_NOT_READABLE（src/writeguard.rs:30-35），walk 递归读目录（:41-102）：read_dir 失败→REPO_NOT_READABLE 错误（:42-47），entries 按文件名排序保证确定性（:49），symlink_metadata 不跟随、symlink 一律跳过（:53-61），目录命中 SKIP_DIRS 剪枝（:62-65），文件 strip_prefix 后统一 '/' 分隔作 key（:68-73），Sha256 全量哈希（:74-98）。Manifest = BTreeMap<String,[u8;32]>（:26-27）。
   - src/writeguard.rs:26-102（审计 #2）
3. 大文件优化：>1MB（SNAPSHOT_FULL_HASH_MAX=1024*1024，src/writeguard.rs:22-24）只摘「size.to_le_bytes + 首64KB」指纹（:80-93）；首 64KB 读失败降级按 0 字节计并 tracing::warn 可见（:85-92）。
   - src/writeguard.rs:22-24（审计 #2）
   - src/writeguard.rs:80-93（审计 #2）
4. diff：after 中 before 没有的=Added、哈希不同的=Modified（:119-131）；before 有 after 没有=Removed（:132-139）；结果 sort 后返回（:140-141）。三类 ChangeKind（:104-109）。
   - src/writeguard.rs:104-142（审计 #2）
5. 数据流落点：write_check 事件经 audit::append_line（src/audit.rs:21-51）追加到 <state_dir>/audit/{session_id}.jsonl，seq 取全文件最大 seq+1 保持单调，payload 先并入、顶层 seq/ts_ms/kind 后写防键冲突；文件落点由 Audit::create 决定 state_dir.join("audit")（src/audit.rs:70-76）。
   - src/audit.rs:21-51（审计 #32）
   - src/cli.rs:379-387（审计 #4）
6. 归因上报分支：changes 为空 → stderr「零写入自证：N 文件快照，无变更 ✓」（src/cli.rs:388-389）；非空 → tracing::error 完整性违规 + stderr 列前 5 条 + 「其余见审计」（:391-406）；考后 snapshot 本身失败 → 写 available:false 的 write_check 行（:409-419），不冒充成功也不中断主流程。考前快照失败（guard_before=None）则整个自证静默跳过（src/cli.rs:247 的 .ok()）。
   - src/cli.rs:247（审计 #4）
   - src/cli.rs:388-419（审计 #4）
7. 配置与开关：writeguard 无任何 CLI flag / 配置项 / 环境变量输入，SKIP_DIRS 与 1MB 阈值均为硬编码常量；grep 证实调用点仅 cli.rs 两处 + lib.rs 模块声明，run_task_inner 不读 self.vector/self.repo_map 之外的开关来控制自证——始终执行（除非考前快照失败）。
   - src/writeguard.rs:11-24（审计 #2）
   - src/cli.rs:246-247（审计 #4）
8. 豁免面契约：SKIP_DIRS 含 .git/.codegraph/.codesleuth/target/node_modules/dist/__pycache__/.venv（src/writeguard.rs:11-20）；vector::store::project_index_dir 把索引放 <repo>/.codesleuth 并注释与 SKIP_DIRS 同待遇（src/vector/store.rs:303-308）；bootlock 锁文件常驻 .codesleuth/boot.lock 同豁免（src/bootlock.rs:12-13）；vector/chunk.rs:37 也在分块时跳过 .codesleuth 对齐。测试 dot_codegraph_is_excluded_from_manifest 验证 .codegraph 增长不产生 diff（src/writeguard.rs:212-224）。
   - src/vector/store.rs:303-308（审计 #12）
   - src/bootlock.rs:12-13（审计 #14）
   - src/vector/chunk.rs:34-43（审计 #37）
   - src/writeguard.rs:212-224（审计 #2）
9. 错误码：快照所有 IO 失败统一 REPO_NOT_READABLE=CS3002（src/errors.rs:43），段位 CS3xxx→exit code 4（src/errors.rs:18）。审计 write_check 留痕二次失败仅 tracing::warn 不升级（src/cli.rs:416-417）。
   - src/errors.rs:13-22（审计 #9）
   - src/errors.rs:41-44（审计 #9）
   - src/cli.rs:409-419（审计 #4）
10. 测试覆盖：no_change 空增删改（:148-156）、modify/add/remove 三类（:158-193）、unix symlink 跳过防成环（:195-210）、.codegraph 豁免（:212-224）、write_check append seq 单调不重且 payload seq 键不覆盖顶层（:226-256）。
   - src/writeguard.rs:144-256（审计 #2）
11. 初稿勘误：初稿称「diff 检测增/删/改三类变更 src/writeguard.rs:26-39」——实际 26-39 只有 Manifest 类型与 snapshot 入口，三类变更逻辑在 :104-142；其余结论与代码一致。
   - src/writeguard.rs:26-39（审计 #2）
   - src/writeguard.rs:104-142（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=10 · tool_calls=14 · duration=65072ms · tokens=173460


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论

零写入自证：任务前后对目标仓库做 per-file manifest 快照比对，证明 codesleuth 全程只读；凡差异（除 .codegraph/.codesleuth 等工具元数据豁免目录）一律如实归因为「不可归因变更」，写入审计并打印 stderr 摘要，给调用方可审计的零写入保证。

## 证据列表

1. 功能价值：证明 codesleuth 在目标仓内的可写面为空集，任何差异如实上报（不可归因变更），不冒领不瞒报
    - src/writeguard.rs:1-4（审计 #2）
2. 关键文件 src/writeguard.rs：snapshot 生成 BTreeMap<路径,sha256> manifest，diff 检测增/删/改三类变更
    - src/writeguard.rs:26-39（审计 #2）
    - src/writeguard.rs:117-142（审计 #2）
3. 调用链：cli.rs 任务前 snapshot 考前快照 → agent.run 任务运行 → 任务后再 snapshot + diff，写审计 write_check 行并输出 stderr 摘要
    - src/cli.rs:246-247（审计 #7）
    - src/cli.rs:369-389（审计 #7）
4. 交互：快照豁免 SKIP_DIRS（.git/.codegraph/.codesleuth/target 等），与 vector::store 的项目索引目录 .codesleuth、bootlock 锁文件共用同一豁免约定（D011 工具元数据）；errors 模块提供 REPO_NOT_READABLE
    - src/writeguard.rs:11-20（审计 #2）
    - src/vector/store.rs:305-307（审计 #12）
    - src/bootlock.rs:12-13（审计 #17）

## 死胡同

无

## 置信度

high

## 统计

turns=7 · tool_calls=5 · duration=26444ms · tokens=55536
