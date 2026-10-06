# 配置管理子命令（CLI与命令层）

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
# 配置管理子命令（CLI与命令层）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
codesleuth 的 `config` 子命令由 `Cli::run` 派发到 `run_config`，按 `ConfigAction::{Get, Set, Path}` 三个动作分别查看生效配置（单键或全量）、写入全局 `config.toml`、以及打印全局/项目配置文件路径。实现上 `config_get`/`config_set` 走 `config::load` / `config::global_config_path` / `config::apply_set` / `config::resolved_get` / `config::to_file_view` 等下层函数完成读取与持久化，对外不修改其它状态。

## 证据列表
1. Cli::run 顶层派发 Command::Config 分支到 run_config，传入 action 与 profile。
   - src/cli.rs:107-108（审计 #2）
2. ConfigAction 枚举定义三个子动作：Get { key }、Set { key, value }、Path。
   - src/cli.rs:80-96（审计 #2）
3. run_config 按动作分流：Path 打印全局与项目配置路径；Get 调 config_get；Set 调 config_set 并打印写入路径。
   - src/cli.rs:480-511（审计 #2）
4. config_get 内部调 config::load 合并配置，无键走 to_file_view 序列化全量，有键走 resolved_get 读单值。
   - src/cli.rs:667-678（审计 #2）
5. config_set 内部取 global_config_path、parse_file 已有文件（或默认）、apply_set 写入键、然后 toml::to_string_pretty 写回磁盘。
   - src/cli.rs:680-701（审计 #2）
6. config::load 实现「全局文件 + 项目文件 + CLI 覆盖」的分层合并。
   - src/config.rs:190-204（审计 #2）
7. global_config_path 固定指向 ~/.codesleuth/config.toml。
   - src/config.rs:159-162（审计 #2）
8. apply_set / resolved_get 共用同一张键表（find_key），未知键报 CS1012。
   - src/config.rs:487-497（审计 #2）
9. to_file_view 把运行时 Config 投影回 FileConfig，用于 `config get` 全量展示。
   - src/config.rs:499-524（审计 #2）
10. parses_config_subcommands 测试覆盖了 config get|path 与 index --rebuild 的子命令解析正确性。
   - src/cli.rs:775-796（审计 #2）

## 死胡同
- grep "ConfigAction|run_config|Command::Config" 第一次返回 0 命中（管道被工具吞掉），改用分别 grep 与 explore 拿到全部位置。
- 未读 src/config.rs 中 parse_file / FileConfig / merge_file / merge_cli 内部实现，因属实现细节且与本概览无关。

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=33333ms · tokens=65967


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
config 子命令由 Cli::run 派发到 run_config（src/cli.rs:107-108, 480-511），按 ConfigAction::{Get, Set, Path} 三分支（src/cli.rs:80-96）。Get 走 config_get→config::load→to_file_view（全量）或 resolved_get（单键）；Set 走 config_set→global_config_path→parse_file→apply_set→toml 整文件覆盖。加载链在 src/config.rs:191-223 实现"默认 ← 全局 ← 项目 ← CLI"，env 层已整体移除（src/config.rs:1-3），档位 apply_profile 先于 merge_cli。键表 config_key_table 13 写键 + 1 只读（llm.profile），apply_set/resolved_get/keys_hint 三处共用。错误统一走 report_error，CS101x→exit code 2。初稿说"加键改 4-5 处"实际是治理前状态，表驱动后改 1 处；初稿说"环境变量层"实际已移除，测试也只断言 CLI > 项目 > 全局三层。

## 证据列表
1. Cli 顶层用 clap Parser 派生，--base-url/--model/--profile 旗标在 36-43，command 子命令字段在 56-57
   - src/cli.rs:14-58（审计 #2）
2. Command 枚举含 Config { action: ConfigAction } 与 Index { path, rebuild, vector }，Config 是 subcommand 入口
   - src/cli.rs:60-77（审计 #2）
3. ConfigAction 派生 clap Subcommand，三变体 Get{key:Option}、Set{key,value}、Path（无字段）
   - src/cli.rs:80-96（审计 #2）
4. Cli::run 在 match self.command 分发 Command::Config 到 run_config(action, self.profile.as_deref())，仅透传 profile 不透传 base_url/model
   - src/cli.rs:99-130（审计 #2）
5. run_config 三分支：Path 打 global_config_path+project_config_path 路径，Get/Set 调 config_get/config_set 并用 report_error + exit_code
   - src/cli.rs:480-511（审计 #2）
6. config_get 调 config::load(CliOverrides{ profile, ..Default })；无键 toml::to_string_pretty(&to_file_view)，有键 resolved_get
   - src/cli.rs:667-678（审计 #2）
7. config_set 流程：global_config_path()→CONFIG_MISSING 兜底；parse_file 存在文件或 FileConfig::default；apply_set 写键；create_dir_all(parent) 创建；toml::to_string_pretty 后 std::fs::write 整文件覆盖
   - src/cli.rs:680-701（审计 #2）
8. parses_config_subcommands 测试覆盖 config get <key>/config path/index . --rebuild 的 clap 解析
   - src/cli.rs:775-796（审计 #2）
9. Config 加载链入口 config::load，解析 global/project 可选路径后委托 load_layered；project_config_path 优先 .codesleuth/config.toml，缺失回退 codesleuth.toml
   - src/config.rs:169-204（审计 #13）
10. global_config_path 固定 ~/.codesleuth/config.toml；global_state_dir 固定 ~/.codesleuth/
   - src/config.rs:159-167（审计 #13）
11. parse_file 读文件后 toml::from_str 反序列化为 FileConfig（所有字段 Option），错误用 CONFIG_INVALID + 路径 hint
   - src/config.rs:178-188（审计 #13）
12. load_layered 纯函数：Config::default → [global, project] 循环 parse_file+merge_file+profiles.extend → apply_profile → merge_cli
   - src/config.rs:206-223（审计 #13）
13. apply_profile (D017) 先于 merge_cli：未知名报 CONFIG_INVALID + 列出可用档位；已知名按字段覆盖 base_url/api_key/model/model_context_tokens；active_profile = Some(name)
   - src/config.rs:225-259（审计 #13）
14. merge_file/merge_cli 逐字段 Option 覆盖，缺席字段不写；behavior.thinking_on 取反后写入 cfg.thinking_disabled
   - src/config.rs:261-310（审计 #13）
15. ConfigKey 表驱动（P005 R7.2），13 写键 + llm.profile 只读键；apply_set/resolved_get/keys_hint 三处共用 config_key_table()；keys_hint 串全部键名供未知键 hint
   - src/config.rs:312-320（审计 #13）
   - src/config.rs:331-466（审计 #13）
   - src/config.rs:469-475（审计 #13）
   - src/config.rs:477-485（审计 #13）
   - src/config.rs:488-491（审计 #13）
   - src/config.rs:494-497（审计 #13）
16. 键表内行为：llm.api_key 读为 unwrap_or_default（空串）；vector.base_url/api_key 读缺省时回退 llm 字段；thinking_on 仅接 true/false 严格字面量；u64/u32/usize 用 parse_typed 校验
   - src/config.rs:342-348（审计 #13）
   - src/config.rs:410-456（审计 #13）
   - src/config.rs:322-329（审计 #13）
17. to_file_view 把运行时 Config 投影回完整 FileConfig，profiles 强制空，全量打印在 config get 走它
   - src/config.rs:499-524（审计 #13）
18. Config::resolve_api_key 只认配置直配，Some("   ") 与 None 都视作未配并报 CONFIG_MISSING（与 config get llm.api_key 行为不一致）
   - src/config.rs:528-538（审计 #13）
   - src/config.rs:650-651（审计 #13）
19. 测试矩阵：key_table_roundtrip_and_unknown_key 验证 set→load→get 闭环与未知键 CS1012 含 graph.bin hint；READ_ONLY_KEYS 显式跳过 llm.profile 的 set 走 get
   - src/config.rs:545-580（审计 #13）
20. 测试覆盖：precedence（断言 CLI > 项目 > 全局三层，已无 env）；project_file_overrides_global_for_vector；thinking 默认关；profile_overrides/profile_full_override；unknown_profile 列出档位；project 同名档位压全局；invalid_toml 报 CS1012 且 exit_code()==2；apply_set_rejects_unknown_key；missing_files 回退 default
   - src/config.rs:622-791（审计 #13）
21. CsError/CsCode 段位决定 exit code：CS1xxx→1 用法、CS101x→2 配置/凭据、CS2xxx→3 LLM、CS3xxx→4 目标库、CS4xxx→5 索引、CS5xxx→6 内部；具体码 USER_INPUT=1001、CONFIG_MISSING=1011、CONFIG_INVALID=1012、INTERNAL=5001；report_error 走 stderr 打 code+message+source_text+hint
   - src/errors.rs:10-115（审计 #36）
22. main 入口顺序：Cli::parse → init_tracing(verbose, session_id) → cli.run(&session_id) → std::process::exit(code)
   - src/main.rs:1-12（审计 #97）
23. lib.rs 通过 crate::config::global_state_dir() 决定会话日志文件路径（与 config 子命令不直接耦合）
   - src/lib.rs:35-66（审计 #77）
24. Config::default 内置 base_url=https://api.openai.com/v1、model=gpt-4o-mini、context 1_000_000/60%、vector Qwen3-Embedding-8B 1024d composite 24_000、thinking 默认开、graph.bin="codegraph"
   - src/config.rs:121-148（审计 #13）
25. run_task 与 run_index_vector 共享 config::load（src/cli.rs:140, 221），config_get 也调它但 profile 透传、base_url/model 丢弃；config_set 不读 project_config_path
   - src/cli.rs:132-145（审计 #2）
   - src/cli.rs:184-224（审计 #2）
   - src/cli.rs:667-678（审计 #2）
   - src/cli.rs:680-701（审计 #2）

## 死胡同
- grep 复合模式 "ConfigAction|run_config|Command::Config" 首次返回 0 命中（管道/正则被工具吞），改用分别 grep + read 定位
- grep "use crate::config|use config::" 返回 0 命中（符号被 strip），改用 grep "config::load" 等具体引用名定位
- 未读 src/audit.rs、src/writeguard.rs、src/vector/* 等下游客细节，仅以 cli.rs:140, 221 引用点确认它们消费 config::load 的产物
- 未读 tests/layered_live.rs / tests/llm_live.rs / tests/fixtures/fixture-rs/src/main.rs（仅作引用证据 grep 命中）

## 置信度
high

## 统计
turns=17 · tool_calls=44 · duration=237116ms · tokens=553687


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
codesleuth 的 `config` 子命令由 `Cli::run` 派发到 `run_config`，按 `ConfigAction::{Get, Set, Path}` 三个动作分别查看生效配置（单键或全量）、写入全局 `config.toml`、以及打印全局/项目配置文件路径。实现上 `config_get`/`config_set` 走 `config::load` / `config::global_config_path` / `config::apply_set` / `config::resolved_get` / `config::to_file_view` 等下层函数完成读取与持久化，对外不修改其它状态。

## 证据列表
1. Cli::run 顶层派发 Command::Config 分支到 run_config，传入 action 与 profile。
   - src/cli.rs:107-108（审计 #2）
2. ConfigAction 枚举定义三个子动作：Get { key }、Set { key, value }、Path。
   - src/cli.rs:80-96（审计 #2）
3. run_config 按动作分流：Path 打印全局与项目配置路径；Get 调 config_get；Set 调 config_set 并打印写入路径。
   - src/cli.rs:480-511（审计 #2）
4. config_get 内部调 config::load 合并配置，无键走 to_file_view 序列化全量，有键走 resolved_get 读单值。
   - src/cli.rs:667-678（审计 #2）
5. config_set 内部取 global_config_path、parse_file 已有文件（或默认）、apply_set 写入键、然后 toml::to_string_pretty 写回磁盘。
   - src/cli.rs:680-701（审计 #2）
6. config::load 实现「全局文件 + 项目文件 + CLI 覆盖」的分层合并。
   - src/config.rs:190-204（审计 #2）
7. global_config_path 固定指向 ~/.codesleuth/config.toml。
   - src/config.rs:159-162（审计 #2）
8. apply_set / resolved_get 共用同一张键表（find_key），未知键报 CS1012。
   - src/config.rs:487-497（审计 #2）
9. to_file_view 把运行时 Config 投影回 FileConfig，用于 `config get` 全量展示。
   - src/config.rs:499-524（审计 #2）
10. parses_config_subcommands 测试覆盖了 config get|path 与 index --rebuild 的子命令解析正确性。
   - src/cli.rs:775-796（审计 #2）

## 死胡同
- grep "ConfigAction|run_config|Command::Config" 第一次返回 0 命中（管道被工具吞掉），改用分别 grep 与 explore 拿到全部位置。
- 未读 src/config.rs 中 parse_file / FileConfig / merge_file / merge_cli 内部实现，因属实现细节且与本概览无关。

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=33333ms · tokens=65967
