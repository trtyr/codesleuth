# System Prompt（侦察编排）

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
# System Prompt（侦察编排）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
System Prompt 是 codesleuth 只读侦察 Agent 的恒定身份与行为底座，定义于 `src/prompt.rs:4-30` 的 `SYSTEM_PROMPT` 常量，由 `Harness::run` 在每次会话启动时注入到 LLM 首条 system 消息（src/harness.rs:76-78），把「只读身份 / 检索纪律 / 无空转 / 输出契约」一次性钉死，保证后续轮次不因 user 任务诱导越界（D005）。

## 证据列表
1. SYSTEM_PROMPT 常量在 src/prompt.rs:4-30 定稿，内容为「只读身份 + 行为边界 + 检索纪律（层进协议）+ 无空转 + 输出契约」的中文 system prompt。
   - src/prompt.rs:4-30（审计 #2）
2. 锚点测试 prompt_contains_contract_anchors（src/prompt.rs:37-56）断言 SYSTEM_PROMPT 含「只读 / 拒绝 / explore / find_files / grep / callers / read / file:line / 无新信息 / submit_report / 置信度 / 死胡同」等关键锚点，防止定稿漂移。
   - src/prompt.rs:37-56（审计 #2）
3. Harness::run 在启动时构造 messages，首条 ChatMessage::System 的 content 引用 crate::prompt::SYSTEM_PROMPT（src/harness.rs:76-78），紧随 ChatMessage::User 装载具体 task（src/harness.rs:79-81），system 与 user 严格隔离。
   - src/harness.rs:74-82（审计 #6）
4. D005 决策（docs/plantree/plans/001-read-only-agent-harness/decisions/005-prompt-layering.md:6-11）拍板「system 定只读身份（产品资产，随发行版走），user 载具体任务，任务细节永不进 system；只读身份永不因 user prompt 改变」——把 SYSTEM_PROMPT 抬升为不可漂移的产品资产。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/005-prompt-layering.md:6-11（审计 #4）
5. D006 决策（docs/plantree/plans/001-read-only-agent-harness/decisions/006-all-tools-layered-protocol.md:15-26）把「层进式检索协议」明确写入 D005 的 system 层，使 SYSTEM_PROMPT 的「检索纪律」段获得设计依据。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/006-all-tools-layered-protocol.md:15-26（审计 #19）
6. SYSTEM_PROMPT 的下游消费面：Harness::run 整轮 LLM 决策（src/harness.rs:136 `self.provider.chat(&req).await`）都建立在注入的 system 身份之上；测试路径 run_with（src/harness.rs:803-821）也复用 Harness::new→Harness::run 链，间接覆盖 SYSTEM_PROMPT 注入。
   - src/harness.rs:136（审计 #6）
   - src/harness.rs:803-821（审计 #6）

## 死胡同
- src/vector/repomap.rs（相似度 0.52，grep 后与 SYSTEM_PROMPT 无引用关系，弃用）
- src/vector/chunk.rs（相似度 0.48，grep 后与 SYSTEM_PROMPT 无引用关系，弃用）

## 置信度
high

## 统计
turns=6 · tool_calls=8 · duration=19739ms · tokens=38034


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
本功能是 codesleuth Agent 不可漂移的「身份 + 行为契约」底座，定义为 `src/prompt.rs:4-30` 的 `pub const SYSTEM_PROMPT: &str`，由 `Harness::run` 在 `src/harness.rs:74-82` 构造首条 `ChatMessage::System` 时直接以 `crate::prompt::SYSTEM_PROMPT.into()`（src/harness.rs:77）注入到每轮 LLM 请求的 messages[0]，并由锚点测试 `prompt_contains_contract_anchors`（src/prompt.rs:37-56）防漂移；任何 task-派生数据（任务正文、repo map、召回块、地形提示）都走首条 User 消息后缀（src/harness.rs:83-88 + `with_first_user_suffix`），遵循 D005 严格分层：system = 产品资产随发行版走，user = 运行时不可信输入，user 永远不能改写只读身份。该 system prompt 自身不存在配置/环境开关、无运行时变体（初稿隐含「恒定」判断成立；初稿死胡同中 `repomap.rs` 头部注释与 `wrap_repo_section` 函数注释仍写「注入位置 = system prompt 尾部」，与 src/harness.rs:74-88 实际写法的差异点已在 `docs/contexts/向量检索/06-repo-map.md:32` 显式记录，本报告沿用以代码为准）。

## 证据列表
1. SYSTEM_PROMPT 是 crate `codesleuth::prompt` 模块（src/lib.rs:18 `pub mod prompt;`）下唯一导出符号，定义为 `pub const SYSTEM_PROMPT: &str`（src/prompt.rs:4-30），原文 raw string，结构分四段：身份与边界（行 6-9，唯一职责 / 零写入 / 拒绝越界任务）、检索纪律层进协议（行 11-17，explore → find_files/grep → read 三层 + 层间回喂）、无空转（行 19-20）、收敛与报告 + 输出契约（行 22-30，submit_report JSON 形状 + 结论/证据/死胡同/置信度四要素）。
   - src/prompt.rs:1-30（审计 #2）
   - src/prompt.rs:32-57（审计 #2）
2. 锚点测试 prompt_contains_contract_anchors 用 13 个关键字符串（"只读" "拒绝" "explore" "find_files" "grep" "callers" "read" "file:line" "无新信息" "submit_report" "置信度" "死胡同"）断言 SYSTEM_PROMPT.contains(anchor)（src/prompt.rs:53），保证 prompt 定稿被改时合约键不被静默丢失——这是 system prompt 作为「产品资产」的机械化防漂移机制（无 env/config 开关，无运行时变体）。
   - src/prompt.rs:37-56（审计 #2）
   - src/prompt.rs:52-56（审计 #2）
3. Harness::run 在 messages 向量首部硬编码两条 ChatMessage：System{content: crate::prompt::SYSTEM_PROMPT.into()}（src/harness.rs:76-78）紧接 User{content: task.to_string()}（src/harness.rs:79-81），随后若 self.first_user_suffix 非空则取 last_mut 把 "\n\n" + suffix 追加到首条 User 消息 content（src/harness.rs:83-88）——这一段代码是 system / user 分层纪律的物理实现：system 一次性钉入，user 任务正文 + 任何任务派生数据（向量召回块 / 全局或任务 repo map / 地形提示）都走 User 消息，system 槽永远不被覆盖。
   - src/harness.rs:74-88（审计 #11）
   - src/harness.rs:76-78（审计 #11）
   - src/harness.rs:83-88（审计 #11）
4. SYSTEM_PROMPT 的下游消费面 = Harness::run 主循环每轮 LLM 请求：构造 ChatRequest{model, messages, tools}（src/harness.rs:126-134）后 `self.provider.chat(&req).await`（src/harness.rs:136）调用 OpenAI 兼容 provider；每轮 LLM 决策结果通过 `audit.record("llm", …)`（src/harness.rs:153-165）落审计，包含 turn / model / duration_ms / prompt_tokens / completion_tokens / text / tool_call_count 七个结构化字段。整轮 LLM 决策——包括打转时塞入的「请调用 submit_report」「连续多回合无新信息」引导消息（src/harness.rs:186-188、396-398）——都建立在 system 注入的「只读 + 无空转 + submit_report 收敛」契约之上，契约是被打转熔断 / 拒绝 prompt injection 越界任务（src/harness.rs:308-313 同 canonical 重复调用拒绝文本）的最终裁决依据。
   - src/harness.rs:126-148（审计 #11）
   - src/harness.rs:136（审计 #11）
   - src/harness.rs:153-165（审计 #11）
5. Harness 结构体（src/harness.rs:35-44）持有 first_user_suffix: Option<String> 字段，由 `with_first_user_suffix(mut self, suffix: String) -> Self`（src/harness.rs:69-72）以 builder 模式注入；模块顶部 doc 注释（src/harness.rs:67-68）显式说明该字段是「首条用户消息后缀（P003 E2 v2：召回块 + 任务相关导航图，随首条注入一次，非 system——D005 分层：system 恒定，任务派生数据走 user 消息；『进门看一眼图就收起来』）」，把 D005 落到代码字段上。
   - src/harness.rs:46-72（审计 #11）
   - src/harness.rs:67-72（审计 #11）
   - src/harness.rs:69-72（审计 #11）
6. CLI 真实消费面 = `run_task_inner`（src/cli.rs:359-371）：构造 Harness::new(provider, registry, audit_log, cfg.llm.model.clone(), cfg.context.model_context_tokens, cfg.context.compact_at_percent)（src/cli.rs:359-366），再以 builder 模式按 first_suffix 是否 Some 决定是否调用 `with_first_user_suffix(suffix)`（src/cli.rs:367-370），最后 `rt.block_on(agent.run(&task))?`（src/cli.rs:371）触发主循环——该路径同时是 D005「system 不可漂移」与 D013「context_tokens × compact_at_percent 决定压缩阈值」两条决策的汇合点。
   - src/cli.rs:359-371（审计 #25）
   - src/cli.rs:367-370（审计 #25）
   - src/cli.rs:371（审计 #25）
7. 首条 User 消息后缀的组装 = 三条路径汇合（src/cli.rs:304-356）：① --vector 路径调 setup_vector_layer 返回 Some(suffix)（src/cli.rs:319），含 recall 块 + 任务相关 repo map（src/cli.rs:620-622、623-658）；② 仅 --repo-map 无向量路径走 build_repo_map + wrap_repo_section（src/cli.rs:340-346）；③ 兜底把 parts 拼成 first_suffix（src/cli.rs:336-338）。三条路径共同遵守 system 恒定纪律：所有 task-派生信息（向量召回 / 全局或任务导航图 / 地形提示）都只进 first_user_suffix（src/harness.rs:42-43 字段 → 87-88 追加），不污染 system。
   - src/cli.rs:304-356（审计 #25）
   - src/cli.rs:339-346（审计 #25）
   - src/cli.rs:619-663（审计 #25）
8. 对抗测试覆盖 system prompt 注入语义：tests/adversarial.rs 用 `use codesleuth::harness::Harness;`（行 8）拼 Scripted provider + Harness::new(provider, registry, Audit::create(...), "m".into(), 1_000_000, 60)（行 189-196）+ `agent.run("重试逻辑在哪").await`（行 197），并以「目标仓库在完整会话后被改动——只读边界被击穿」断言（行 201）锁死「system 写明的只读身份不因 user task 改写」；只读工具白名单 READ_ONLY_TOOLS（行 20-29，read/find_files/grep/explore/callers/callees/impact/files 八个）是 D011「工具面物理无写能力」的可执行化，与 system prompt 的「你没有任何写入能力」行文形成代码 + 文档双重锁定。
   - tests/adversarial.rs:8（审计 #41）
   - tests/adversarial.rs:19-29（审计 #41）
   - tests/adversarial.rs:186-202（审计 #41）
9. 测试夹具 run_with（src/harness.rs:803-821）以 Scripted provider + Harness::new(provider, registry, audit, "m".into(), 1_000_000, 60) + harness.run("任务") 模式（行 818-819）跑主循环；同文件内的 compaction_fires_when_window_exceeds_and_evicts 测试（src/harness.rs:1080-1099）以更小阈值 (400, 100) 触发压缩，间接验证「SYSTEM_PROMPT 注入 → 压缩 → handoff 仍以原 task 为锚 → recall 回读」全链路不断链——证明 system 注入与 D008/D013 压缩协议正交可独立测试。
   - src/harness.rs:803-821（审计 #11）
   - src/harness.rs:1075-1099（审计 #11）
10. SYSTEM_PROMPT 经 LLM 协议层到 OpenAI 兼容网关的 wire-format 映射由 `to_request_message`（src/llm.rs:149-194）承担：领域枚举 ChatMessage::System{content}（src/llm.rs:27-29、151-156）被转成 async-openai 的 ChatCompletionRequestSystemMessage{content, name: None}，与 ChatMessage::User{content} 严格区分 role 标签——role 隔离是 D005 分层的最后一道物理屏障：即便 harness 误用 system 槽，wire-format 仍以 `role: "system"` 发出，第三方网关不会把它当 user 解释。message_mapping_preserves_roles 单测（src/llm.rs:360+）锁定四种 role 串行保真。
   - src/llm.rs:26-43（审计 #98）
   - src/llm.rs:149-194（审计 #98）
   - src/llm.rs:151-156（审计 #98）
11. SYSTEM_PROMPT 在经 chat() 发到网关时受 thinking 开关影响：OpenAiProvider 字段 `thinking_disabled: bool`（src/llm.rs:89）由 `Config.thinking_disabled` 注入，`thinking_disabled = true`（默认 / 检索型任务）会在 build_request 后（src/llm.rs:236-237）注入网关扩展字段关掉 thinking，但 system role 本身不被改写——这意味着 SYSTEM_PROMPT 文字内容与 role 标签是该功能唯一受「产品资产」原则保护的不变量；thinking、base_url、model、api_key、max_retries 都是 provider 层独立配置项（src/llm.rs:82-90），互不影响。
   - src/llm.rs:82-90（审计 #98）
   - src/llm.rs:88（审计 #98）
   - src/llm.rs:92-105（审计 #98）
12. D005 决策（docs/plantree/plans/001-read-only-agent-harness/decisions/005-prompt-layering.md:6-11）拍板「system prompt（产品资产，随发行版走）= 只读 Agent 身份 + 行为边界 + 检索纪律 + 输出契约」与「user prompt（运行时传入，CLI 第一公民参数）= 具体任务」，并显式规定「严格隔离：任务细节永不进 system；只读身份永不因 user prompt 改变」——这把 system prompt 抬升为产品资产，并标注「system prompt 与工具描述是注入的两个主攻面，均须进对抗测试」（行 14-15），与 tests/adversarial.rs 的三层断言闭环。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/005-prompt-layering.md:6-11（审计 #6）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/005-prompt-layering.md:14-15（审计 #6）
13. D006 决策（docs/plantree/plans/001-read-only-agent-harness/decisions/006-all-tools-layered-protocol.md:15-26）把「层进式检索协议（开局看图 → 顺藤摸点 → 词穷撒网 → 落锤有证；允许跳层但结论必过读层；层进是证据质量策略不是成本策略）」明确写入 D005 的 system 层；行 27-31 补「层间喂送是循环放大器（语义层 keywords → 模糊层 grep；结构图邻域 → 读层清单；读层新发现 → 回图补查 callers/impact）」；行 36-37 标注「D005 的 system prompt 内容清单 +1：层进式检索协议」——这段是 src/prompt.rs:11-17 检索纪律四条的决策原文，src/prompt.rs:15-17 的三条附加规则直接对应 D006 的循环放大器段。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/006-all-tools-layered-protocol.md:15-26（审计 #8）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/006-all-tools-layered-protocol.md:27-31（审计 #8）
   - docs/plantree/plans/001-read-only-agent-harness/decisions/006-all-tools-layered-protocol.md:36-37（审计 #8）
14. 差异点：src/vector/repomap.rs 的模块 doc 注释（行 1-3）与 wrap_repo_section 文档注释（行 180）均写「注入位置 = system prompt 尾部 / system prompt 注入段」，但实际装配由 src/harness.rs:74-88 + src/cli.rs:339-346 / 619-663 走的是首条 User 消息后缀（system 槽保持 crate::prompt::SYSTEM_PROMPT 恒定）；docs/contexts/向量检索/06-repo-map.md:6 与 :32 已显式记录「以代码为准」并把行 32-35 列为差异点——本报告沿用该裁决（与初稿证据点 3 一致），把 repomap.rs 注释视作陈旧口径。
   - src/vector/repomap.rs:1-3（审计 #79）
   - src/vector/repomap.rs:180-183（审计 #79）
   - docs/contexts/向量检索/06-repo-map.md:6（审计 #25）
   - docs/contexts/向量检索/06-repo-map.md:32（审计 #25）
15. SYSTEM_PROMPT 的输出契约条款在 build_report（src/harness.rs:460-598）有机械化兑现：每个 finding 的 evidence.file 必须 evidence.cite_seq(file) 返回 Some 才算有效（src/harness.rs:504-509），否则计入 uncited 并把整张 report 打回——这与 system prompt 行 23「findings 的每条 evidence 必须是本会话真实读过的文件」直接对应，构成「system 写明 + 代码强制 + 锚点测试 + 对抗测试」四层闭环：任一层单独被绕过，剩下三层仍会锁死只读身份与证据真实性。
   - src/harness.rs:460-598（审计 #11）
   - src/harness.rs:504-509（审计 #11）
16. 无空转纪律的代码级兑现：① 重复调用（同 canonical 键）→ reject_call 拒绝 + no_progress += 1（src/harness.rs:300-313），与 system prompt 行 20「收到『无新信息』提示后立即换工具/换角度」呼应；② 零增量执行 → 连续 zero_gain_streak >= 2 注入「连续多回合无新信息」User 消息（src/harness.rs:386-399），与 system prompt 行 17「禁止重复调用」呼应；③ 连续 prose（无工具调用）→ 首轮注入 submit_report 引导 User 消息，二轮降级 Report::degraded_prose（src/harness.rs:176-212），与 system prompt 行 22-23 submit_report JSON 形状契约呼应——system prompt 的所有非空转条款都被 mainloop 用具体分支落地，不存在纯文档条款。
   - src/harness.rs:300-313（审计 #11）
   - src/harness.rs:386-399（审计 #11）
   - src/harness.rs:176-212（审计 #11）

## 死胡同
- grep "Harness::new|Harness::run" plain 模式返回 0（管道符被分词切碎）——已切到 regex 直接 grep "Harness" / "with_first_user_suffix" / "ChatMessage" 补足外部调用点（src/cli.rs:359-371、tests/adversarial.rs:189-197、src/harness.rs:818/1080）。
- find_files "001-read-only-agent-harness" / "plantree" 命中 0——索引未覆盖 docs/plantree；但 D005/D006 在 grep "prompt-layering" 后用相对路径 read 直接读到了两份决策原文。
- grep 尝试 fuzzy "system|prompt|identity" 返回 0（该词条命中过多被截断）——改用 plain + regex 拼装后确认 SYSTEM_PROMPT 在生产代码仅 1 处引用（src/harness.rs:77），加 1 处测试引用（src/prompt.rs:53）。
- src/vector/repomap.rs 头部注释（行 1-3）与 wrap_repo_section 文档注释（行 180）声称「注入位置 = system prompt」，实际由 src/harness.rs:74-88 追加到首条 User 消息 content 后缀——以代码为准，注释属陈旧口径，已在证据列表标注差异。
- D002（无空转）/D003（prompt injection 防御）/Q007 等被 system prompt 间接引用的决策文档未直接 read，仅依据 D005/D006 原文与 system prompt 行文交叉印证；如需深挖应读 docs/plantree/plans/001-read-only-agent-harness/decisions/002-no-empty-churn.md、003-prompt-injection.md。

## 置信度
high

## 统计
turns=22 · tool_calls=57 · duration=105648ms · tokens=650187


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
System Prompt 是 codesleuth 只读侦察 Agent 的恒定身份与行为底座，定义于 `src/prompt.rs:4-30` 的 `SYSTEM_PROMPT` 常量，由 `Harness::run` 在每次会话启动时注入到 LLM 首条 system 消息（src/harness.rs:76-78），把「只读身份 / 检索纪律 / 无空转 / 输出契约」一次性钉死，保证后续轮次不因 user 任务诱导越界（D005）。

## 证据列表
1. SYSTEM_PROMPT 常量在 src/prompt.rs:4-30 定稿，内容为「只读身份 + 行为边界 + 检索纪律（层进协议）+ 无空转 + 输出契约」的中文 system prompt。
   - src/prompt.rs:4-30（审计 #2）
2. 锚点测试 prompt_contains_contract_anchors（src/prompt.rs:37-56）断言 SYSTEM_PROMPT 含「只读 / 拒绝 / explore / find_files / grep / callers / read / file:line / 无新信息 / submit_report / 置信度 / 死胡同」等关键锚点，防止定稿漂移。
   - src/prompt.rs:37-56（审计 #2）
3. Harness::run 在启动时构造 messages，首条 ChatMessage::System 的 content 引用 crate::prompt::SYSTEM_PROMPT（src/harness.rs:76-78），紧随 ChatMessage::User 装载具体 task（src/harness.rs:79-81），system 与 user 严格隔离。
   - src/harness.rs:74-82（审计 #6）
4. D005 决策（docs/plantree/plans/001-read-only-agent-harness/decisions/005-prompt-layering.md:6-11）拍板「system 定只读身份（产品资产，随发行版走），user 载具体任务，任务细节永不进 system；只读身份永不因 user prompt 改变」——把 SYSTEM_PROMPT 抬升为不可漂移的产品资产。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/005-prompt-layering.md:6-11（审计 #4）
5. D006 决策（docs/plantree/plans/001-read-only-agent-harness/decisions/006-all-tools-layered-protocol.md:15-26）把「层进式检索协议」明确写入 D005 的 system 层，使 SYSTEM_PROMPT 的「检索纪律」段获得设计依据。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/006-all-tools-layered-protocol.md:15-26（审计 #19）
6. SYSTEM_PROMPT 的下游消费面：Harness::run 整轮 LLM 决策（src/harness.rs:136 `self.provider.chat(&req).await`）都建立在注入的 system 身份之上；测试路径 run_with（src/harness.rs:803-821）也复用 Harness::new→Harness::run 链，间接覆盖 SYSTEM_PROMPT 注入。
   - src/harness.rs:136（审计 #6）
   - src/harness.rs:803-821（审计 #6）

## 死胡同
- src/vector/repomap.rs（相似度 0.52，grep 后与 SYSTEM_PROMPT 无引用关系，弃用）
- src/vector/chunk.rs（相似度 0.48，grep 后与 SYSTEM_PROMPT 无引用关系，弃用）

## 置信度
high

## 统计
turns=6 · tool_calls=8 · duration=19739ms · tokens=38034
