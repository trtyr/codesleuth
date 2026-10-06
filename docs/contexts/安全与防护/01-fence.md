# Fence路径围栏（安全与防护）

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
# Fence路径围栏（安全与防护）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
仓库的路径围栏（Fence）位于 src/fence.rs，是读类工具访问仓库前统一的硬边界：任何传入路径必须最终解析到 repo root 之内，越界（绝对路径出库、`..` 逃逸、symlink 指向库外）一律返回 CS3003（FENCE_DENIED）。其采用「词汇层预检 + dunce::canonicalize 解 symlink 复检」两道关卡，并在 CLI 启动时由 src/cli.rs:245 装配为 Arc<Fence> 注入 ReadTool 与 FuzzyEngine。

## 证据列表
1. Fence 是仓库只读边界的第二层：所有工具的目标路径必须解析到 repo root 之内，symlink 逃逸一律拒绝，越界返 CS3003 结构化报错。
   - src/fence.rs:1-2（审计 #2）
2. Fence::new 接收仓库根并用 dunce::canonicalize 规范化，失败时返回 REPO_NOT_FOUND（CS3001）。
   - src/fence.rs:12-21（审计 #2）
   - src/errors.rs:42-44（审计 #19）
3. resolve 实现「词汇层预检 + canonicalize 后复检」两道防御：先对 candidate_abs 做 starts_with_root 判定，再用 dunce::canonicalize 解析 symlink 后再判一次，两次越界都返 FENCE_DENIED；不存在的路径返 REPO_NOT_FOUND。
   - src/fence.rs:27-55（审计 #2）
4. 词汇层归一化 normalize 仅折叠 `.` 与 `..`，不解 symlink；围栏前缀判定 starts_with_root 在 Windows 下按 ASCII 大小写不敏感比较，其余平台严格比较。
   - src/fence.rs:58-88（审计 #2）
5. FENCE_DENIED 错误码为 CS3003，对应退出码 4。
   - src/errors.rs:44（审计 #19）
   - src/fence.rs:114-115（审计 #2）
6. CLI 启动期由 cli.rs 装配 Arc<Fence> 并注入 ReadTool 与 FuzzyEngine；同时 FuzzyEngine::new 内部自建一份 Fence 仅用于取规范化根作为模糊检索 base_path。
   - src/cli.rs:245-250（审计 #21）
   - src/tools/fuzzy.rs:22-37（审计 #25）
   - src/tools/read.rs:15-22（审计 #23）
7. ReadTool 在 call 阶段先调 self.fence.resolve(&path_arg) 守门，通过后才落盘读取；越界/逃逸由 Fence 抛 FENCE_DENIED。
   - src/tools/read.rs:86-93（审计 #23）
8. 单测覆盖：内部存在路径放行、symlink 逃逸拒绝、绝对路径越界拒绝、不存在路径返 REPO_NOT_FOUND。
   - src/fence.rs:94-132（审计 #2）
9. 对抗性集成测试 assertion_2_fence_rejects_escapes 验证 innocent.txt（symlink 指向库外 .ssh_keys）、/etc/passwd、../../etc/passwd 三种敌对路径均被 Fence 以 FENCE_DENIED 拒绝。
   - tests/adversarial.rs:141-161（审计 #4）

## 死胡同
- 用 `\.resolve\(` 仅匹配到 src/fence.rs 与 tests/adversarial.rs 的测试调用与 src/tools/read.rs:86 一处生产调用，src/tools/fuzzy.rs 内未发现对 fence.resolve 的调用（仅借 fence.root() 取规范化根）。
- 未在仓库中找到其他对 Fence::new 的生产调用点之外的非测试构造方（grep 命中的 9 处中 5 处为 tests/* 与 src/fence.rs 内 #[cfg(test)]）。

## 置信度
high

## 统计
turns=7 · tool_calls=13 · duration=33460ms · tokens=49780


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
仓库的路径围栏（Fence）位于 src/fence.rs，是 codesleuth 读类工具访问目标仓库的硬边界。`Fence::new` 用 dunce::canonicalize 规范化 repo root（失败抛 CS3001 REPO_NOT_FOUND），`Fence::resolve` 走「词汇层 normalize 预检 + dunce::canonicalize 复检」两道关卡，越界（绝对路径出库、.. 词法逃逸、symlink 指向库外）一律抛 CS3003 FENCE_DENIED。CLI 启动期由 src/cli.rs:245 装配为 Arc<Fence> 注入 ReadTool（src/tools/read.rs:86 守门）与 FuzzyEngine（src/tools/fuzzy.rs:25 自建一份仅取 root 作 picker base_path）。CS3xxx 段位统一映射 exit 4，stderr 走 src/errors.rs:107-115 report_error 三行渲染。Fence 自身无配置项，受 --repo CLI 旗标、dunce 库与 #[cfg(windows)] 平台分支影响；生产路径上唯一 resolve 调用点是 src/tools/read.rs:86。

## 证据列表
1. Fence 结构 + 头注释：D009 只读边界第二层，所有工具目标路径必须解析到 repo root 之内，symlink 逃逸一律拒绝，越界返 CS3003
   - src/fence.rs:1-9（审计 #2）
2. Fence::new 用 dunce::canonicalize 规范化 root，失败抛 CsError{code: REPO_NOT_FOUND, message: "仓库根目录不可达: {root}", source_text: "canonicalize: {e}"}；成功后存 self.root: PathBuf
   - src/fence.rs:11-21（审计 #2）
   - src/errors.rs:42-44（审计 #4）
3. Fence::root 返回 &Path，只读访问规范化根目录（被 FuzzyEngine 用于 FilePicker.base_path）
   - src/fence.rs:23-25（审计 #2）
   - src/tools/fuzzy.rs:25-27（审计 #13）
4. Fence::resolve 主入口：先 root.join(rel) + current_dir().unwrap_or_default() 拼 candidate_abs；第一关 starts_with_root(&normalize(&candidate_abs), &self.root) 越界返 FENCE_DENIED；dunce::canonicalize(&candidate) 失败返 REPO_NOT_FOUND；第二关 starts_with_root(&resolved, &self.root) 复检越界（捕获 symlink 链指向库外）再返 FENCE_DENIED
   - src/fence.rs:27-55（审计 #2）
5. normalize 折叠 . 与 .. 但不解 symlink：ParentDir → out.pop()，CurDir → 跳过，其他原样 push
   - src/fence.rs:58-71（审计 #2）
6. starts_with_root 逐组件 zip 比较；#[cfg(windows)] 走 eq_ignore_ascii_case，其他平台严格 ==；不只做字符串前缀，避免 /foo/bar-evil 误判围栏内
   - src/fence.rs:73-88（审计 #2）
7. 错误码段位：FENCE_DENIED = CsCode(3003)，REPO_NOT_FOUND = CsCode(3001)；3000..=3999 段位统一映射 exit 4（与 FENCE_DENIED 退码相同）
   - src/errors.rs:12-23（审计 #4）
   - src/errors.rs:42-44（审计 #4）
   - src/errors.rs:101-104（审计 #4）
   - src/errors.rs:123-131（审计 #4）
8. CsError 结构：code + message + hint + retryable + source_text；report_error 按 code/根因/hint 三行 eprintln! 到 stderr
   - src/errors.rs:56-115（审计 #4）
9. 模块挂载：src/lib.rs:13 pub mod fence;，由 src/cli.rs:9 use crate::{..., fence, ...} 引入
   - src/lib.rs:6-22（审计 #68）
   - src/cli.rs:9（审计 #6）
10. CLI 装配：src/cli.rs:207-214 dunce::canonicalize(&repo) 解析 --repo 得 repo_abs（失败包装 REPO_NOT_FOUND）；src/cli.rs:245 Arc::new(fence::Fence::new(&repo_abs)?) 构造围栏；src/cli.rs:247 registry.register(Box::new(tools::read::ReadTool::new(fence.clone()))) 注入 ReadTool；src/cli.rs:248-250 FuzzyEngine + FileFinderTool + GrepTool
   - src/cli.rs:184-250（审计 #6）
11. FuzzyEngine 自建一份 Fence：Fence::new(root)? → fence.root().to_string_lossy().into_owned() 作 FilePicker base_path；P005 R6.2 注释说明原 Arc<Fence> 字段零读取已删；FilePickerOptions.watch=false, FFFMode::Ai；失败映射 CS4011 INDEX_BUILD_FAILED
   - src/tools/fuzzy.rs:17-38（审计 #13）
12. ReadTool 结构 + Tool trait 契约：name="read"，parameters 要求 path 必填、offset/limit 可选；execute 守门点 self.fence.resolve(&path_arg)? 紧跟 is_file() 检查（REPO_NOT_READABLE）和 std::fs::read（REPO_NOT_READABLE）；二进制探测、lossy UTF-8、空文件、offset 越界均给出诚实声明
   - src/tools/read.rs:15-93（审计 #11）
   - src/tools/read.rs:95-146（审计 #11）
   - src/tools/mod.rs:14-22（审计 #51）
13. Tool trait 4 方法（name/description/parameters/execute）+ Send + Sync；ToolRegistry 内部 Vec<Box<dyn Tool>>，按 name 线性 find
   - src/tools/mod.rs:14-58（审计 #51）
14. VectorSearchTool 不消费 Fence：通过 RecallEngine.recall 拿语义位置指针，工具描述明确写「作为证据使用前必须 read 原文」（即由 read 工具走 Fence 守门）
   - src/tools/vector_search.rs:10-64（审计 #56）
15. Harness 主循环 dispatch：self.tools.get(&call.name) 取 tool → audit.record("tool_call",{...}) → read 工具从 args 摘 path → tool.execute(args).await；execute Err 透传（推断，未直接读 Err 分支）
   - src/harness.rs:320-399（审计 #63）
16. writeguard 是平行防线：CLI 考前 src/cli.rs:253 writeguard::snapshot(&repo_abs) + 考后 src/cli.rs:377-379 snapshot + diff 拿 Change{path, kind: Added|Removed|Modified} 列表；symlink 一律跳过（防成环 + 不哈希出仓内容）；>1MB 文件指纹化（size + 首 64KB）避免快照分钟级阻塞；SKIP_DIRS 含 .git/.codegraph/.codesleuth/target/node_modules/dist/__pycache__/.venv
   - src/writeguard.rs:1-142（审计 #53）
   - src/cli.rs:253（审计 #6）
   - src/cli.rs:377-379（审计 #6）
17. 单测覆盖：allows_inside_and_resolves（内部存在路径放行）、symlink_escape_denied（#[cfg(unix)] symlink 逃逸拒绝 + assert exit_code 4）、absolute_outside_denied（/etc/hosts 拒）、nonexistent_is_repo_not_found（不存路径返 REPO_NOT_FOUND）
   - src/fence.rs:90-132（审计 #2）
18. 对抗性集成测试 assertion_2_fence_rejects_escapes：构造 innocent.txt 软链指向库外 .ssh_keys（#[cfg(unix)]），遍历 ["innocent.txt", "/etc/passwd", "../../etc/passwd"] 逐个断言 FENCE_DENIED（CS3003）
   - tests/adversarial.rs:141-161（审计 #16）
19. 完整会话零写入测试 assertion_3_target_repo_untouched_after_full_session：fingerprint per-file sha256，before/after BTreeMap 严格相等；fingerprint 与 copy_dir 都跳过 .git/.codegraph/target/node_modules；脚本化 LLM 走 find_files→read→submit_report 真实链
   - tests/adversarial.rs:41-202（审计 #16）
20. 读类工具装配模板在 3 个集成测试中复用：Arc::new(Fence::new(&repo)) + ReadTool::new(fence.clone()) + FuzzyEngine::new(&repo) + FileFinderTool + GrepTool，Harness::new 注入 provider/registry/audit
   - tests/replay.rs:59-90（审计 #42）
   - tests/layered_live.rs:17-59（审计 #44）
   - tests/adversarial.rs:31-39（审计 #16）

## 死胡同
- 未在仓库内读到 harness.rs 中 tool.execute 返回 Err 后的具体处理（熔断计数、错误喂回 LLM message 形态）；仅在 src/harness.rs:320-399 看到 tool_call 成功路径，错误分支由 grep `Err(e)` 推断（验证途径：grep `Err(e)` in src/harness.rs:362+）
- 未直接读 src/main.rs 启动代码（推断行号 5-12 by docs/00-overview.md:6 引用）；验证途径：read src/main.rs
- 未读 src/config.rs：Fence 与 Config 字段无引用关系（推断：grep `fence` in src/config.rs 应为 0 命中，但本会话未直接验证）
- 未读 src/audit.rs：Fence 错误是否进 audit 行未直接验证；仅从 writeguard.rs:226-249 推断审计写行存在
- grep `\.resolve\(` 与 `fence\.resolve` 均 0 命中（除自检 md），生产 resolve 调用点经逐文件读确认仅 src/tools/read.rs:86 一处

## 置信度
high

## 统计
turns=11 · tool_calls=31 · duration=154293ms · tokens=432810


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
仓库的路径围栏（Fence）位于 src/fence.rs，是读类工具访问仓库前统一的硬边界：任何传入路径必须最终解析到 repo root 之内，越界（绝对路径出库、`..` 逃逸、symlink 指向库外）一律返回 CS3003（FENCE_DENIED）。其采用「词汇层预检 + dunce::canonicalize 解 symlink 复检」两道关卡，并在 CLI 启动时由 src/cli.rs:245 装配为 Arc<Fence> 注入 ReadTool 与 FuzzyEngine。

## 证据列表
1. Fence 是仓库只读边界的第二层：所有工具的目标路径必须解析到 repo root 之内，symlink 逃逸一律拒绝，越界返 CS3003 结构化报错。
   - src/fence.rs:1-2（审计 #2）
2. Fence::new 接收仓库根并用 dunce::canonicalize 规范化，失败时返回 REPO_NOT_FOUND（CS3001）。
   - src/fence.rs:12-21（审计 #2）
   - src/errors.rs:42-44（审计 #19）
3. resolve 实现「词汇层预检 + canonicalize 后复检」两道防御：先对 candidate_abs 做 starts_with_root 判定，再用 dunce::canonicalize 解析 symlink 后再判一次，两次越界都返 FENCE_DENIED；不存在的路径返 REPO_NOT_FOUND。
   - src/fence.rs:27-55（审计 #2）
4. 词汇层归一化 normalize 仅折叠 `.` 与 `..`，不解 symlink；围栏前缀判定 starts_with_root 在 Windows 下按 ASCII 大小写不敏感比较，其余平台严格比较。
   - src/fence.rs:58-88（审计 #2）
5. FENCE_DENIED 错误码为 CS3003，对应退出码 4。
   - src/errors.rs:44（审计 #19）
   - src/fence.rs:114-115（审计 #2）
6. CLI 启动期由 cli.rs 装配 Arc<Fence> 并注入 ReadTool 与 FuzzyEngine；同时 FuzzyEngine::new 内部自建一份 Fence 仅用于取规范化根作为模糊检索 base_path。
   - src/cli.rs:245-250（审计 #21）
   - src/tools/fuzzy.rs:22-37（审计 #25）
   - src/tools/read.rs:15-22（审计 #23）
7. ReadTool 在 call 阶段先调 self.fence.resolve(&path_arg) 守门，通过后才落盘读取；越界/逃逸由 Fence 抛 FENCE_DENIED。
   - src/tools/read.rs:86-93（审计 #23）
8. 单测覆盖：内部存在路径放行、symlink 逃逸拒绝、绝对路径越界拒绝、不存在路径返 REPO_NOT_FOUND。
   - src/fence.rs:94-132（审计 #2）
9. 对抗性集成测试 assertion_2_fence_rejects_escapes 验证 innocent.txt（symlink 指向库外 .ssh_keys）、/etc/passwd、../../etc/passwd 三种敌对路径均被 Fence 以 FENCE_DENIED 拒绝。
   - tests/adversarial.rs:141-161（审计 #4）

## 死胡同
- 用 `\.resolve\(` 仅匹配到 src/fence.rs 与 tests/adversarial.rs 的测试调用与 src/tools/read.rs:86 一处生产调用，src/tools/fuzzy.rs 内未发现对 fence.resolve 的调用（仅借 fence.root() 取规范化根）。
- 未在仓库中找到其他对 Fence::new 的生产调用点之外的非测试构造方（grep 命中的 9 处中 5 处为 tests/* 与 src/fence.rs 内 #[cfg(test)]）。

## 置信度
high

## 统计
turns=7 · tool_calls=13 · duration=33460ms · tokens=49780
