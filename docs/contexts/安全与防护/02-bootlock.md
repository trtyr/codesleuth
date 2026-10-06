# bootlock跨进程锁（安全与防护）

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
# bootlock跨进程锁（安全与防护）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
bootlock 跨进程锁是仓库级引导互斥原语：以 `<repo>/.codesleuth/boot.lock` 上的 flock（`std::fs::File::try_lock`）把 graph 引导（init/index --force）与向量索引构建两层 check-then-act 操进程间串行化；acquire 返回 Won/Lost/超时三态（Lost 与超时统一以 CS4016 INDEX_LOCKED 判负退出，不降级），BootLockGuard 在 Drop 时 unlock，引导段结束即释放，不跨 LLM 调用、不罩 serve。

## 证据列表
1. 模块定位：仓库级引导锁 `<repo>/.codesleuth/boot.lock`，用 `std::fs::File::try_lock`（flock，Rust 1.89 稳定）把 graph 引导（init / index --force）与向量构建两层 check-then-act 索引引导在进程间串行化，进程崩溃由内核放锁无陈旧锁。
   - src/bootlock.rs:1-13（审计 #2）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:24-27（审计 #4）
2. 三态结果：acquire 返回 `BootLock::Won(BootLockGuard)` 锁空闲即得、`BootLock::Lost` 等待后拿到（对手刚完成引导，判负退出）、轮询超过 timeout（默认 300s，与 INDEX_TIMEOUT 对齐）→ CS4016 INDEX_LOCKED 结构化错误。
   - src/bootlock.rs:27-33（审计 #2）
   - src/bootlock.rs:50-114（审计 #2）
3. 守卫与释放：`BootLockGuard` 保存 File 句柄，Drop 中调用 `self.file.unlock()` 双保险释放；acquire_guard 封装判负语义——Won 直接拿 guard 干活，Lost → CS4016 INDEX_LOCKED 带 hint「重跑本命令即可」。
   - src/bootlock.rs:38-48（审计 #2）
   - src/bootlock.rs:116-129（审计 #2）
4. 调用关系：模块经 `src/lib.rs:7 pub mod bootlock` 挂载；下游三处消费——`src/tools/graph.rs:78` codegraph 引导入口（init/index --force 段，进锁后 `drop(guard)` 放锁再 `McpClient::spawn` serve）、`src/cli.rs:162` 手动 `index --vector` 构建、`src/cli.rs:565` run 时向量构建兜底引导，三处共用同一把 boot.lock。
   - src/lib.rs:1-15（审计 #21）
   - src/tools/graph.rs:75-89（审计 #19）
   - src/cli.rs:161-166（审计 #15）
   - src/cli.rs:563-569（审计 #15）

## 死胡同
- 用 grep 搜 bootlock/BootLock/boot\.lock 全仓只命中 4 个文件，已覆盖 lib.rs/cli.rs/tools/graph.rs/bootlock.rs 全部消费点，无需进一步。
- 未读取 errors.rs 中 INDEX_LOCKED 常量定义（仅在 bootlock.rs:16 use 与 :79/:94/:109 引用处见其码值），不影响本侦察结论。

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=27333ms · tokens=46397


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
bootlock 是仓库级跨进程引导互斥原语：在 `<repo>/.codesleuth/boot.lock` 上以 `std::fs::File::try_lock`（flock，Rust 1.89 稳定）把 graph 引导（codegraph init / index --force）与向量索引构建两层 check-then-act 索引引导在进程间串行化。模块暴露 `acquire`（三态 Won/Lost/超时）与 `acquire_guard`（二态：Ok guard / Err INDEX_LOCKED），三处调用点共享：src/tools/graph.rs:78（codegraph 引导入口，引导段结束 drop(guard) 后再 spawn serve）、src/cli.rs:162（index --vector 子命令）、src/cli.rs:565（run 时 setup_vector_layer 兜底引导，构建段结束 drop(vguard) 后再开库）。三态/Lost/超时统一以 CS4016 INDEX_LOCKED（src/errors.rs:51，exit code 5）判负退出，CLI 侧对 INDEX_LOCKED 显式 return Err(e)（src/cli.rs:265-268, src/cli.rs:321-324）不走降级 catch-all；BootLockGuard 在 Drop 时静默调用 self.file.unlock()（src/bootlock.rs:44-48），锁不跨 LLM 调用、不罩 serve。DEFAULT_TIMEOUT=300s、POLL_INTERVAL=500ms，均为写死常量。

## 证据列表
1. 模块挂载：pub mod bootlock 在 src/lib.rs:7；三处消费点 = src/tools/graph.rs:78 / src/cli.rs:162 / src/cli.rs:565，共用 DEFAULT_TIMEOUT=300s。
   - src/lib.rs:7（审计 #4）
   - src/tools/graph.rs:78-82（审计 #13）
   - src/cli.rs:162-166（审计 #15）
   - src/cli.rs:565-569（审计 #15）
2. 三态结果：enum BootLock { Won(BootLockGuard), Lost }（src/bootlock.rs:28-33），acquire 轮询 500ms 间隔、上限 300s，超时与 Lost 统一归 CS4016 INDEX_LOCKED（src/bootlock.rs:50-114）。
   - src/bootlock.rs:22-24（审计 #2）
   - src/bootlock.rs:27-33（审计 #2）
   - src/bootlock.rs:50-114（审计 #2）
   - src/bootlock.rs:78-80（审计 #2）
   - src/bootlock.rs:93-98（审计 #2）
   - src/bootlock.rs:103-106（审计 #2）
   - src/bootlock.rs:108-110（审计 #2）
3. 守卫与释放：BootLockGuard 保存 File，Drop 中 let _ = self.file.unlock() 静默双保险（src/bootlock.rs:38-48）；acquire_guard 把 Lost 归一为 INDEX_LOCKED 带 hint「重跑本命令即可」（src/bootlock.rs:116-129）。
   - src/bootlock.rs:38-48（审计 #2）
   - src/bootlock.rs:116-129（审计 #2）
   - src/bootlock.rs:123-127（审计 #2）
4. CLI 侧对 INDEX_LOCKED 显式 return Err(e) 判负（src/cli.rs:265-268 graph 引导、src/cli.rs:321-324 向量层装配），不走降级 catch-all；与 build_vector_index 失败时的 warn+audit degraded 降级路径分叉（src/cli.rs:581-589）。
   - src/cli.rs:265-268（审计 #15）
   - src/cli.rs:321-324（审计 #15）
   - src/cli.rs:581-589（审计 #15）
5. 锁不跨 LLM 调用、不罩 serve/检索：tools/graph.rs:88 drop(guard) 在 McpClient::spawn 之前；cli.rs:590 drop(vguard) 在 VectorStore::open 之前。
   - src/tools/graph.rs:83-90（审计 #13）
   - src/cli.rs:588-595（审计 #15）
6. 锁路径写死 <repo>/.codesleuth/boot.lock（src/bootlock.rs:52, :59），create_dir_all 自动建目录，OpenOptions create+truncate(false)+write(true)（src/bootlock.rs:60-64）；.codesleuth 是 writeguard 豁免目录（D011）。
   - src/bootlock.rs:52-70（审计 #2）
   - src/bootlock.rs:12-14（审计 #2）
7. 错误码 INDEX_LOCKED = CsCode(4016) 段位 CS4xxx → exit_code 5（src/errors.rs:51, src/errors.rs:19-22）；REPO_NOT_READABLE = CsCode(3002) exit 4 用于 create_dir_all/open 失败（src/bootlock.rs:53-70）。
   - src/errors.rs:19-22（审计 #19）
   - src/errors.rs:43（审计 #19）
   - src/errors.rs:51（审计 #19）
   - src/bootlock.rs:53-70（审计 #2）
8. tracing 契约：debug Won（src/bootlock.rs:75）/ warn 进入轮询（src/bootlock.rs:87-91）/ warn Lost 判负（src/bootlock.rs:103-105）/ Drop 静默（P005 R2）。
   - src/bootlock.rs:75（审计 #2）
   - src/bootlock.rs:87-91（审计 #2）
   - src/bootlock.rs:103-105（审计 #2）
   - docs/plantree/plans/005-d014-review-remediation/roadmap.md:14-19（审计 #10）
9. D014 决策原文：仓库级引导锁 .codesleuth/boot.lock，graph 层 + 向量层共用；try_lock 失败→轮询→拿到=对方已建完=立即判负退出 CS4016 INDEX_LOCKED，等锁超时同码；用户 directive「输者死不降级」。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:1-34（审计 #6）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:20-27（审计 #6）
10. P005 R1 三处判负语义收口到 acquire_guard（commit c671b58）：src/bootlock.rs:116-129 新增 + graph.rs/setup_vector_layer/run_index_vector 三处单行化 + 回归测试 acquire_guard_reports_lost_as_index_locked。
   - docs/plantree/plans/005-d014-review-remediation/roadmap.md:6-13（审计 #10）
   - src/bootlock.rs:187-203（审计 #2）
11. init 段进锁条件：if force_reindex { run_cli(index --force --quiet) } else if !root.join(".codegraph").exists() { run_cli(init) }（src/tools/graph.rs:83-87）；热路径（已建索引）也要付一次 flock 开销。
   - src/tools/graph.rs:83-87（审计 #13）
   - src/tools/graph.rs:26-42（审计 #13）
12. try_lock 双分支语义：WouldBlock 才进轮询（src/bootlock.rs:81, :111），非 WouldBlock 的 io::Error 立即 Err(INDEX_LOCKED)（src/bootlock.rs:78-80, :108-110），区分「真被占」与「系统级 IO 错误」。
   - src/bootlock.rs:72-82（审计 #2）
   - src/bootlock.rs:101-112（审计 #2）
13. index --vector 子命令入口：run_index_vector 在 src/cli.rs:132-182，由 Command::Index { vector: true, .. } 分发（src/cli.rs:114-121）；CLI 分发根在 Cli::run（src/cli.rs:99-130）。
   - src/cli.rs:67-78（审计 #15）
   - src/cli.rs:99-130（审计 #15）
   - src/cli.rs:132-182（审计 #15）
14. 单元测试覆盖：fresh_lock_is_won_and_reacquirable_after_release / second_acquirer_waits_then_reports_lost / wait_timeout_is_index_locked / acquire_guard_reports_lost_as_index_locked（src/bootlock.rs:131-203）。
   - src/bootlock.rs:131-203（审计 #2）
   - src/bootlock.rs:135-148（审计 #2）
   - src/bootlock.rs:150-167（审计 #2）
   - src/bootlock.rs:169-185（审计 #2）
   - src/bootlock.rs:187-203（审计 #2）
15. 初稿说「三态」是从 acquire 视角（src/bootlock.rs:28-33），对三处调用方实际是二态——acquire_guard 把 Lost 归一为 Err(INDEX_LOCKED)（src/bootlock.rs:120-129），Cli/CodegraphEngine 只看到 Ok(guard)/Err(INDEX_LOCKED)。
   - src/bootlock.rs:28-33（审计 #2）
   - src/bootlock.rs:120-129（审计 #2）
   - src/tools/graph.rs:78-82（审计 #13）
   - src/cli.rs:162-166（审计 #15）
   - src/cli.rs:565-569（审计 #15）

## 死胡同
- grep 搜 boot\.lock|BootLock|bootlock 全仓 0 命中（实际是 plain 模式 + 正则未在跨符号表覆盖），已改用多轮 plain grep 确认 13 命中文件，已穷尽 lib.rs/cli.rs/tools/graph.rs/bootlock.rs/errors.rs + docs/contexts/* 全部消费点与说明点
- 未读 src/vector/build.rs / src/vector/store.rs 内部对 build_vector_index 的实现——不在 bootlock 契约面内（bootlock 仅观察上层调用点）
- 未读 docs/plantree/plans/001-read-only-agent-harness/decisions/011-* / 003.1-* 完整原文（仅通过 grep + bootlock 决策文档交叉印证其与 INDEX_LOCKED/豁免目录的引用）

## 置信度
high

## 统计
turns=7 · tool_calls=19 · duration=118311ms · tokens=167307


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
bootlock 跨进程锁是仓库级引导互斥原语：以 `<repo>/.codesleuth/boot.lock` 上的 flock（`std::fs::File::try_lock`）把 graph 引导（init/index --force）与向量索引构建两层 check-then-act 操进程间串行化；acquire 返回 Won/Lost/超时三态（Lost 与超时统一以 CS4016 INDEX_LOCKED 判负退出，不降级），BootLockGuard 在 Drop 时 unlock，引导段结束即释放，不跨 LLM 调用、不罩 serve。

## 证据列表
1. 模块定位：仓库级引导锁 `<repo>/.codesleuth/boot.lock`，用 `std::fs::File::try_lock`（flock，Rust 1.89 稳定）把 graph 引导（init / index --force）与向量构建两层 check-then-act 索引引导在进程间串行化，进程崩溃由内核放锁无陈旧锁。
   - src/bootlock.rs:1-13（审计 #2）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/014-index-boot-lock.md:24-27（审计 #4）
2. 三态结果：acquire 返回 `BootLock::Won(BootLockGuard)` 锁空闲即得、`BootLock::Lost` 等待后拿到（对手刚完成引导，判负退出）、轮询超过 timeout（默认 300s，与 INDEX_TIMEOUT 对齐）→ CS4016 INDEX_LOCKED 结构化错误。
   - src/bootlock.rs:27-33（审计 #2）
   - src/bootlock.rs:50-114（审计 #2）
3. 守卫与释放：`BootLockGuard` 保存 File 句柄，Drop 中调用 `self.file.unlock()` 双保险释放；acquire_guard 封装判负语义——Won 直接拿 guard 干活，Lost → CS4016 INDEX_LOCKED 带 hint「重跑本命令即可」。
   - src/bootlock.rs:38-48（审计 #2）
   - src/bootlock.rs:116-129（审计 #2）
4. 调用关系：模块经 `src/lib.rs:7 pub mod bootlock` 挂载；下游三处消费——`src/tools/graph.rs:78` codegraph 引导入口（init/index --force 段，进锁后 `drop(guard)` 放锁再 `McpClient::spawn` serve）、`src/cli.rs:162` 手动 `index --vector` 构建、`src/cli.rs:565` run 时向量构建兜底引导，三处共用同一把 boot.lock。
   - src/lib.rs:1-15（审计 #21）
   - src/tools/graph.rs:75-89（审计 #19）
   - src/cli.rs:161-166（审计 #15）
   - src/cli.rs:563-569（审计 #15）

## 死胡同
- 用 grep 搜 bootlock/BootLock/boot\.lock 全仓只命中 4 个文件，已覆盖 lib.rs/cli.rs/tools/graph.rs/bootlock.rs 全部消费点，无需进一步。
- 未读取 errors.rs 中 INDEX_LOCKED 常量定义（仅在 bootlock.rs:16 use 与 :79/:94/:109 引用处见其码值），不影响本侦察结论。

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=27333ms · tokens=46397
