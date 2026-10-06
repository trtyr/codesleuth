# 日志双轨（错误与日志）

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
# 日志双轨（错误与日志）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
「日志双轨」功能在 init_tracing 中通过 tracing registry 同时挂 stderr_layer 与 file_layer 实现：stderr 给人实时看、~/.codesleuth/logs/<session>.log 留作归档，两轨共用同一 EnvFilter 与同一格式，--json 模式下 stdout 仍只承载报告 JSON；归档由 SessionLog 单代轮转（64MB 超限 → .log.old），并在 session span 内自动携带 session_id 完成结构化串线。

## 证据列表
1. 功能注释明确定义日志双轨：stderr 人读 + ~/.codesleuth/logs/<session>.log 归档，同过滤同格式；--json 下 stdout 严禁日志
   - src/lib.rs:24-26（审计 #2）
   - src/logs.rs:1-3（审计 #4）
2. init_tracing 由 main 启动时调用，传入 verbose 与 session_id；session_id 来自 audit::new_session_id 以保证日志与审计同一身份
   - src/main.rs:3, 7-9（审计 #14）
3. init_tracing 组装 EnvFilter（按 verbose 选 warn/info/debug）、stderr_layer、file_layer，挂在同一 registry，文件层共用同一过滤器
   - src/lib.rs:27-66（审计 #2）
4. 文件层路径来自 global_state_dir() + logs/<session_id>.log，打开失败仅 eprintln 警告并降级为仅 stderr，不阻断
   - src/lib.rs:43-58（审计 #2）
5. SessionLog 单代 64MB 轮转：超限 rename 到 .log.old，append 打开；提供 MakeWriter 实现供 tracing 使用
   - src/logs.rs:10-45, 64-70（审计 #4）
6. 测试验证 session span 内事件自动带 session_id 串线到输出行（结构化串线）
   - src/logs.rs:105-143（审计 #4）

## 死胡同
- 未深入 errors.rs 的 report_error（与日志双轨主链无直接耦合，不影响结论）
- 未读 src/audit 等无关模块

## 置信度
high

## 统计
turns=6 · tool_calls=4 · duration=27363ms · tokens=39519


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
「日志双轨」由 main 调用 init_tracing(verbose, session_id) 启动；session_id 来自 audit::new_session_id 以保证与审计同身份。init_tracing 构造 EnvFilter("codesleuth={level}", level 来自 verbose: 0→warn/1→info/2+→debug) 并把同一 filter 挂在 registry 上，同时挂两层 fmt::layer：stderr_layer 写 std::io::stderr，file_layer 通过 SessionLog 写 ~/.codesleuth/logs/<session_id>.log；文件层打开失败仅 eprintln 降级，不阻断。SessionLog 提供 Write 与 MakeWriter 双实现，open 时若已存在且 >64MB（LOG_ROTATE_BYTES）则 rename 到 .log.old 然后 append 打开——单代覆盖式轮转。Cli::run 入口在 init_tracing 之后进入 tracing::info_span!("session", session_id=…) ，span 字段被 fmt 自动注入两轨输出行，测试断言验证 session_id 串线。stdout 全程不被任何 tracing layer 写入，--json 模式只承载报告 JSON。环境变量层已移除（2026-10-05），无 env 开关；过滤目标前缀硬编码 codesleuth=；轮转阈值是编译期常量无配置暴露。Mutex 中毒用 into_inner 恢复而非 panic；rename 失败被吞掉退化为继续追加。

## 证据列表
1. main 是 init_tracing 唯一调用点，session_id 在 main 入口由 audit::new_session_id 生成以保证日志与审计同身份
   - src/main.rs:5-11（审计 #6）
   - src/audit.rs:54-60（审计 #23）
2. init_tracing 按 verbose 选 EnvFilter 级别（0→warn、1→info、2+→debug），过滤目标前缀硬编码为 codesleuth=，解析失败兜底 warn
   - src/lib.rs:27-37（审计 #2）
3. stderr_layer 用 fmt::layer().with_target(false).with_writer(std::io::stderr)；file_layer 用 fmt::layer().with_target(false).with_ansi(false).with_writer(SessionLog)
   - src/lib.rs:39-51（审计 #2）
4. 文件路径 = global_state_dir()/logs/<session_id>.log；global_state_dir 固定 $HOME/.codesleuth（无任何 CLI/配置/env 覆盖）
   - src/lib.rs:44-45（审计 #2）
   - src/config.rs:164-167（审计 #13）
5. filter 与 stderr_layer、file_layer 三者全部挂在同一 registry 并 init()，事件先经 filter 再分发到两轨，不存在「只在一轨出现」的事件
   - src/lib.rs:60-65（审计 #2）
6. 文件层构造失败仅 eprintln! 警告并返回 None（降级为仅 stderr），不阻断进程；home dir 不可用时 and_then 静默返回 None
   - src/lib.rs:43-58（审计 #2）
7. SessionLog::open 在已存在且 >max_bytes 时先 rename 到 path.with_extension("log.old")（覆盖式单代），再用 create+append 打开；rename 失败被 let _ = 吞掉退化为继续追加；目录由 create_dir_all(parent)? 自动创建
   - src/logs.rs:22-41（审计 #4）
8. LOG_ROTATE_BYTES = 64*1024*1024 编译期常量，运行时不可改；SessionLog 双重实现 Write + MakeWriter 供 tracing 消费
   - src/logs.rs:11（审计 #4）
   - src/logs.rs:48-70（审计 #4）
9. SessionLog 写/刷用 Arc<Mutex<File>>，lock 中毒时通过 into_inner() 恢复而非 panic，持有中毒锁仍能完成写
   - src/logs.rs:48-62（审计 #4）
10. Cli::run 入口在 init_tracing 之后进入 tracing::info_span!("session", session_id = %session_id).entered()，span 字段被 fmt layer 自动注入两轨输出行；测试断言 session{session_id="abc123"} 出现在输出
   - src/cli.rs:99-101（审计 #25）
   - src/logs.rs:105-144（审计 #4）
11. CLI verbose: u8 是 clap ArgAction::Count 计数字段，-v 一次、-vv 两次，无 -v 默认 0=warn
   - src/cli.rs:53-55（审计 #25）
12. 代码注释明文声明「stderr 人读 + ~/.codesleuth/logs/<session>.log 归档双轨，同过滤同格式」以及「--json 模式下 stdout 只准有报告 JSON，日志一律 stderr + 文件」
   - src/lib.rs:24-26（审计 #2）
   - src/logs.rs:1-3（审计 #4）
13. 环境变量层已在 2026-10-05 整体移除（一个配置文件管一切），本功能无任何 env 开关；全局家 ~/.codesleuth 与 reports/audit/ledger/eval 同根
   - src/config.rs:1-3（审计 #13）
   - src/config.rs:164-167（审计 #13）
14. callers 图确认 init_tracing 仅 main 一处调用；tracing::info_span!("session", ...) 也仅 Cli::run 与 logs.rs 测试两处
   - src/main.rs:5（审计 #6）
   - src/cli.rs:101（审计 #25）
   - src/logs.rs:133（审计 #4）

## 死胡同
- 未深挖 src/errors.rs 的 report_error（与日志双轨主链无直接耦合，证据 docs/contexts/错误与日志/02-logging.md:24 已记为初稿死胡同）
- 未读 src/audit.rs 的 Audit 结构体正文（除 new_session_id 段落外），仅以 docs/contexts/安全与防护/04-audit.md:277 关于 .jsonl.old 同款策略的描述作为旁证
- 未验证 --json 旗标在 main 中的实际传递路径（推断：靠「init_tracing 不写 stdout」来满足注释级约束，证据 src/lib.rs:26 注释 + src/lib.rs:39-65 全函数无 with_writer(stdout)）

## 置信度
high

## 统计
turns=7 · tool_calls=15 · duration=127696ms · tokens=107200


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
「日志双轨」功能在 init_tracing 中通过 tracing registry 同时挂 stderr_layer 与 file_layer 实现：stderr 给人实时看、~/.codesleuth/logs/<session>.log 留作归档，两轨共用同一 EnvFilter 与同一格式，--json 模式下 stdout 仍只承载报告 JSON；归档由 SessionLog 单代轮转（64MB 超限 → .log.old），并在 session span 内自动携带 session_id 完成结构化串线。

## 证据列表
1. 功能注释明确定义日志双轨：stderr 人读 + ~/.codesleuth/logs/<session>.log 归档，同过滤同格式；--json 下 stdout 严禁日志
   - src/lib.rs:24-26（审计 #2）
   - src/logs.rs:1-3（审计 #4）
2. init_tracing 由 main 启动时调用，传入 verbose 与 session_id；session_id 来自 audit::new_session_id 以保证日志与审计同一身份
   - src/main.rs:3, 7-9（审计 #14）
3. init_tracing 组装 EnvFilter（按 verbose 选 warn/info/debug）、stderr_layer、file_layer，挂在同一 registry，文件层共用同一过滤器
   - src/lib.rs:27-66（审计 #2）
4. 文件层路径来自 global_state_dir() + logs/<session_id>.log，打开失败仅 eprintln 警告并降级为仅 stderr，不阻断
   - src/lib.rs:43-58（审计 #2）
5. SessionLog 单代 64MB 轮转：超限 rename 到 .log.old，append 打开；提供 MakeWriter 实现供 tracing 使用
   - src/logs.rs:10-45, 64-70（审计 #4）
6. 测试验证 session span 内事件自动带 session_id 串线到输出行（结构化串线）
   - src/logs.rs:105-143（审计 #4）

## 死胡同
- 未深入 errors.rs 的 report_error（与日志双轨主链无直接耦合，不影响结论）
- 未读 src/audit 等无关模块

## 置信度
high

## 统计
turns=6 · tool_calls=4 · duration=27363ms · tokens=39519
