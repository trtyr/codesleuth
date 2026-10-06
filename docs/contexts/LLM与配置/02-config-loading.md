# 配置加载链（LLM与配置）

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
# 配置加载链（LLM与配置）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
配置加载链按"CLI 参数 > 项目配置 > 全局配置 > 默认值"优先级，把多源 TOML 配置合并成运行时 Config（src/config.rs:191-223），环境变量层已整体移除（src/config.rs:1-3）。入口是 config::load，先解析全局/项目可选路径后委托纯函数 load_layered：先 Config::default，再用 merge_file 依次覆盖全局、项目层，再 apply_profile 应用 --profile 档位，最后 merge_cli 写入 CliOverrides。

## 证据列表
1. 配置加载链的优先级与"环境变量层已移除"在文件头文档注释中明文规定
   - src/config.rs:1-3（审计 #2）
2. load 是公开入口，解析全局与项目两处可选路径后委托 load_layered
   - src/config.rs:191-203（审计 #2）
3. load_layered 是纯函数加载链：默认 ← 全局 ← 项目，再 apply_profile、merge_cli
   - src/config.rs:207-223（审计 #2）
4. apply_profile 先于 merge_cli 执行，--model/--base-url 旗标永远是最后覆写者
   - src/config.rs:225-259（审计 #2）
5. merge_file 逐字段以 Option 形式覆盖，缺席字段不覆盖（FileConfig 全部 Option）
   - src/config.rs:9-22（审计 #2）
   - src/config.rs:261-299（审计 #2）
6. 全局配置固定位于 ~/.codesleuth/config.toml，项目配置为 .codesleuth/config.toml 并兼容旧 ./codesleuth.toml
   - src/config.rs:160-176（审计 #2）
7. CliOverrides 与 clap 解耦，仅承载 base_url/model/profile 三个可选字段
   - src/config.rs:150-157（审计 #2）
8. CLI 多处通过 config::load 注入 CliOverrides 触发加载链
   - src/cli.rs:140（审计 #9）
   - src/cli.rs:221（审计 #9）
   - src/cli.rs:668（审计 #9）
9. run_config（config 子命令入口）展示全局/项目路径，并提供 get/set
   - src/cli.rs:480-509（审计 #9）
10. 测试用例直接覆盖了"CLI > 项目 > 全局"与"缺文件回落默认"两个核心契约
   - src/config.rs:622-643（审计 #2）
   - src/config.rs:681-685（审计 #2）

## 死胡同
- 未读 src/config.rs:300-599 之间的 merge_file 收尾、merge_cli 实现细节（本任务为概览级侦察，无需展开）
- scripts/eval/run_eval.py:110-118 仅作为向量召回线索，未 read 原文确认
- 未在 src/cli.rs 中确认 CliOverrides 各字段的 clap 绑定行号（仅确认 run_config 位于 src/cli.rs:480）

## 置信度
high

## 统计
turns=6 · tool_calls=8 · duration=30702ms · tokens=47925


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
配置加载链按 "CLI > 项目 > 全局 > 默认" 优先级，把多个 TOML 源合并成运行时 Config；环境变量层已整体移除。所有"配置相关"出口（run 主路径、index --vector 向量构建、config get 子命令）都汇入 config::load(CliOverrides) 一个入口，进入纯函数 load_layered 走四阶段：Config::default ← merge_file（global→project）← apply_profile（--profile 档位）← merge_cli（--model/--base-url）。CliOverrides 故意与 clap 解耦，仅 base_url/model/profile 三个可选字段，CLI 多处按需装填。run_config（config 子命令）以及背后键表（config_key_table）把 set/get/未知名报错统一到一张表，避免耦合四处。

## 证据列表
1. 文件头文档注释明文规定"CLI > 项目 > 全局 > 默认"优先级，且环境变量层已整体移除（2026-10-05 用户裁决）。
   - src/config.rs:1-3（审计 #2）
2. 公开入口 config::load 解析全局/项目两处可选路径后委托纯函数 load_layered；项目层优先 .codesleuth/config.toml，缺则回退 ./codesleuth.toml（旧兼容），全缺也允许（双 None 时落回 Config::default）。
   - src/config.rs:190-204（审计 #2）
   - src/config.rs:160-176（审计 #2）
3. load_layered 严格四阶段：Config::default ← [global, project] 顺序 merge_file（后写者覆盖） ← apply_profile（--profile）← merge_cli（--model/--base-url）。档位表（profiles BTreeMap）按 global→project 顺序 extend，注释明说"项目层同名覆盖全局"（D017）。
   - src/config.rs:206-223（审计 #2）
   - src/config.rs:213-218（审计 #2）
4. apply_profile 严格执行"先于 merge_cli"：仅覆盖连接面（base_url/api_key/model）+ 上下文窗口（model_context_tokens），缺席字段跟随主 [llm]；档位不触及 vector/graph/behavior。未知档位名报错 CONFIG_INVALID(CS1012) 并列出可用档位清单。
   - src/config.rs:225-259（审计 #2）
   - src/config.rs:235-244（审计 #2）
   - src/config.rs:687-714（审计 #2）
5. merge_file 逐字段以 Option 形式覆盖——FileConfig 全部 Option，缺席字段不覆盖。FileConfig 顶层为 llm/context/vector/behavior/graph 五段，FileLlm 内嵌 profiles BTreeMap。merge_file 本身不消费 profiles，profiles 由 load_layered 第 217 行单独收集。
   - src/config.rs:9-71（审计 #2）
   - src/config.rs:261-301（审计 #2）
6. merge_cli 仅写入 base_url/model 两个字段；profile 不在这里处理（profile 走 apply_profile）。这保证了"--model/--base-url 永远是最后覆写者"。
   - src/config.rs:303-310（审计 #2）
   - src/config.rs:716-735（审计 #2）
7. CliOverrides 与 clap 解耦，仅 base_url/model/profile 三个 Option<String>；CLI 三处按需装填：run_index_vector 借用一份、run_task_inner 重新构造、config_get 单独构造（仅传 profile）。
   - src/config.rs:150-157（审计 #2）
   - src/cli.rs:102-106（审计 #14）
   - src/cli.rs:132-140（审计 #14）
   - src/cli.rs:216-221（审计 #14）
   - src/cli.rs:667-672（审计 #14）
8. config 子命令三动作：Path 仅打印路径；Get 调 config::load 后无键走 to_file_view 全量序列化、有键走 resolved_get 单键；Set 走 global_config_path + apply_set + TOML 写回（写父目录 + 整文件覆写）。
   - src/cli.rs:480-511（审计 #14）
   - src/cli.rs:667-701（审计 #14）
9. config_key_table 一处声明 13 键（llm.base_url/api_key/model/profile、context.model_context_tokens/compact_at_percent、vector.embed_model/embed_dims/embed_mode/base_url/api_key/repomap_budget、behavior.thinking_on、graph.bin），供 apply_set、resolved_get、未知名 hint 三处共用——加新键只改这张表。llm.profile 是只读键（set 拒绝写盘），档位经 [llm.profiles.<名字>] 定义。
   - src/config.rs:312-466（审计 #2）
   - src/config.rs:357-367（审计 #2）
   - src/config.rs:468-491（审计 #2）
10. resolve_api_key 只认配置文件直配（环境变量层已移除）；空白字符串视为未配置，抛 CONFIG_MISSING(CS1011)。值不落日志。
   - src/config.rs:526-539（审计 #2）
   - src/config.rs:645-652（审计 #2）
11. CONFIG_MISSING(1011)/CONFIG_INVALID(1012) 都映射到 exit code 2（CS1xxx 段位 1010-1099 → 2），错误体系见 src/errors.rs:13-23 与 32-34。
   - src/errors.rs:13-23（审计 #29）
   - src/errors.rs:32-34（审计 #29）
12. Config::default 给出所有缺省值：llm.base_url=OpenAI 官方、llm.model=gpt-4o-mini、context.model_context_tokens=1_000_000、context.compact_at_percent=60、vector.embed_model=Qwen3-Embedding-8B、vector.embed_dims=1024、vector.embed_mode="composite"、vector.repomap_budget=24_000、vector.base_url/api_key=None（跟随 llm）、thinking_disabled=true、graph.bin="codegraph"、active_profile=None。
   - src/config.rs:121-148（审计 #2）
13. 测试矩阵直击核心契约：CLI>项目>全局（precedence_cli_beats_env_beats_project_beats_global）、缺文件回落默认（missing_files_fall_back_to_defaults）、项目覆盖全局同段（project_file_overrides_global_for_vector）、thinking 翻转（thinking_defaults_off_and_file_can_enable）、档位覆盖连接面+窗口（profile_overrides_connection_and_window）、档位+CLI 旗标共存的"旗标永远最后"（profile_full_override_and_cli_still_wins）、未知档位列出可用名（unknown_profile_lists_available_names）、项目同名档位压全局（project_profile_overrides_global_same_name）、坏 TOML 报 CS1012 exit=2（invalid_toml_is_cs1012）、键表全键 set→写盘→load→get 闭环+未知名（key_table_roundtrip_and_unknown_key）。
   - src/config.rs:541-792（审计 #2）
14. Cli 的 --model/--base-url/--profile 旗标都显式声明并直接喂给 CliOverrides；config 子命令（ConfigAction::Get/Set/Path）与 index 子命令（带 --vector）也走同一加载路径。
   - src/cli.rs:14-78（审计 #14）
15. Clap 解析在 main 入口（main.rs:5-12）完成 → Cli::run → run 装配 CliOverrides（src/cli.rs:102-106）；run_task_inner 第二次构造 CliOverrides（src/cli.rs:216-221）。两份构造一致但分布在两条路径（run_config 走独立 config_get）。
   - src/main.rs:5-12（审计 #50）
   - src/cli.rs:99-130（审计 #14）
16. 配置消费侧：OpenAiProvider 接收 (base_url, api_key, max_retries, thinking_disabled)；向量层用 resolve_embed_endpoint 解析嵌入供应商（[vector] 缺省跟随 [llm]）。
   - src/llm.rs:82-105（审计 #71）
   - src/cli.rs:524-536（审计 #14）
   - src/cli.rs:237-242（审计 #14）
17. 初稿在「数据流」上需要修正：初稿只说"四层"，实际加载链还有一层隐式的 [llm.profiles.<名字>] 档位表，由 load_layered 单独收集并在 apply_profile 阶段一次性激活；profiles 表本身是合并语义（同 extend，项目压全局），不归 merge_file。
   - src/config.rs:213-218（审计 #2）
   - src/config.rs:59-71（审计 #2）
   - src/config.rs:758-772（审计 #2）

## 死胡同
- grep 对 src/llm.rs、src/audit.rs 单文件路径偶发 0 命中（直接 read 拿到行号绕过）
- scripts/eval/run_eval.py:110-118 仍只作为向量召回线索，未 read 原文确认
- 未追踪 apply_set / resolved_get 在 main run 路径之外的调用点（仅确认键表自洽）

## 置信度
high

## 统计
turns=29 · tool_calls=28 · duration=100872ms · tokens=816547


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
配置加载链按"CLI 参数 > 项目配置 > 全局配置 > 默认值"优先级，把多源 TOML 配置合并成运行时 Config（src/config.rs:191-223），环境变量层已整体移除（src/config.rs:1-3）。入口是 config::load，先解析全局/项目可选路径后委托纯函数 load_layered：先 Config::default，再用 merge_file 依次覆盖全局、项目层，再 apply_profile 应用 --profile 档位，最后 merge_cli 写入 CliOverrides。

## 证据列表
1. 配置加载链的优先级与"环境变量层已移除"在文件头文档注释中明文规定
   - src/config.rs:1-3（审计 #2）
2. load 是公开入口，解析全局与项目两处可选路径后委托 load_layered
   - src/config.rs:191-203（审计 #2）
3. load_layered 是纯函数加载链：默认 ← 全局 ← 项目，再 apply_profile、merge_cli
   - src/config.rs:207-223（审计 #2）
4. apply_profile 先于 merge_cli 执行，--model/--base-url 旗标永远是最后覆写者
   - src/config.rs:225-259（审计 #2）
5. merge_file 逐字段以 Option 形式覆盖，缺席字段不覆盖（FileConfig 全部 Option）
   - src/config.rs:9-22（审计 #2）
   - src/config.rs:261-299（审计 #2）
6. 全局配置固定位于 ~/.codesleuth/config.toml，项目配置为 .codesleuth/config.toml 并兼容旧 ./codesleuth.toml
   - src/config.rs:160-176（审计 #2）
7. CliOverrides 与 clap 解耦，仅承载 base_url/model/profile 三个可选字段
   - src/config.rs:150-157（审计 #2）
8. CLI 多处通过 config::load 注入 CliOverrides 触发加载链
   - src/cli.rs:140（审计 #9）
   - src/cli.rs:221（审计 #9）
   - src/cli.rs:668（审计 #9）
9. run_config（config 子命令入口）展示全局/项目路径，并提供 get/set
   - src/cli.rs:480-509（审计 #9）
10. 测试用例直接覆盖了"CLI > 项目 > 全局"与"缺文件回落默认"两个核心契约
   - src/config.rs:622-643（审计 #2）
   - src/config.rs:681-685（审计 #2）

## 死胡同
- 未读 src/config.rs:300-599 之间的 merge_file 收尾、merge_cli 实现细节（本任务为概览级侦察，无需展开）
- scripts/eval/run_eval.py:110-118 仅作为向量召回线索，未 read 原文确认
- 未在 src/cli.rs 中确认 CliOverrides 各字段的 clap 绑定行号（仅确认 run_config 位于 src/cli.rs:480）

## 置信度
high

## 统计
turns=6 · tool_calls=8 · duration=30702ms · tokens=47925
