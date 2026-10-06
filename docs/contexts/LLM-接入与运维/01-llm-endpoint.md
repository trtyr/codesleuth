# OpenAI 兼容端点接入（LLM 接入与运维）

> 深挖详稿 · codesleuth 逐功能深挖 · 2026-10-06

> 配图：`01-llm-endpoint-diagram.html`

# 侦察报告

任务：你在只读侦察一个代码仓库。下面是一份功能点的初稿文档，它就是你的任务书：接着它往下挖，把这份初稿升级成详细报告。

【初稿内容开始】
# OpenAI 兼容端点接入（LLM 接入与运维）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
OpenAI 兼容端点接入是 codesleuth 的 LLM 通道：通过 OpenAI Chat Completions 兼容协议（支持 newapi 等网关）向任意兼容端点发对话/工具调用请求，用户可用 --model/--base-url 覆盖 ~/.codesleuth/config.toml 的 [llm] 配置，从而灵活换供应商。入口在 CLI（src/cli.rs:35-40 定义旗标、src/cli.rs:231-236 构造 provider），实现是 src/llm.rs:81-105 的 OpenAiProvider。调用链：cli run → config::load（应用 CLI 覆盖）→ OpenAiProvider::new → Harness 持有 Arc<dyn LlmProvider> 在每轮对话中调用。下游被 harness 代理循环消费；嵌入端点（[vector]）缺省跟随 [llm] 的 base_url/key（可显式覆盖）。

## 证据列表
1. OpenAiProvider 是 OpenAI 兼容协议实现，注释明示支持 newapi 等网关，持有 base_url/api_key，直发通道可注入网关扩展字段（thinking）
   - src/llm.rs:81-90（审计 #2）
2. CLI 提供 --model / --base-url 覆盖旗标，构造 CliOverrides 传给 config::load 合入生效配置
   - src/cli.rs:35-40（审计 #11）
   - src/cli.rs:212-216（审计 #11）
3. LlmConfig 定义 base_url/model/api_key，默认 base_url 为 https://api.openai.com/v1、model 为 gpt-4o-mini
   - src/config.rs:99-114（审计 #4）
4. 入口用生效配置构造 OpenAiProvider 并以 Arc<dyn LlmProvider> 共享
   - src/cli.rs:231-236（审计 #11）
5. Harness 持有该 provider，作为代理循环中每轮 LLM 调用的执行者
   - src/harness.rs:35-56（审计 #16）
6. 嵌入供应商缺省跟随 [llm] 的 base_url/api_key，显式 [vector] 覆盖优先（有测试 embed_endpoint_fallback_follows_llm 佐证）
   - src/cli.rs:138-142（审计 #11）

## 死胡同
- grep 'base-url|base_url' plain 模式初查 0 命中（正则字符类问题），改用 regex 后命中

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=18995ms · tokens=35560


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
OpenAI 兼容端点接入是 codesleuth 的唯一 LLM 通道：CLI --model/--base-url 与三层 TOML 配置（CLI>项目>全局>默认，env 层已移除）合成 Config 后构造 OpenAiProvider（src/cli.rs:231-236），以 Arc<dyn LlmProvider> 交给 Harness，主循环每轮经 provider.chat POST {base_url}/chat/completions（bearer 认证、重试 3 次、退避 500ms/1s/2s/4s、超时 300s），支持网关扩展字段 thinking（默认禁用）。api_key 只认配置文件直配，缺失即 CONFIG_MISSING 退出码 2；429/5xx/网络错误可重试，最终失败 retryable=false 由宿主终止（退出码 3）。嵌入端点 [vector] 缺省跟随 [llm]。初稿整体正确，需修正两点：api_key 不支持环境变量（env 层整体移除）；重试次数 2 为硬编码非配置。

## 证据列表
1. 完整调用链：Cli 定义 --model/--base-url → CliOverrides → config::load 三层合并 → resolve_api_key → OpenAiProvider::new(base_url, api_key, 2, thinking_disabled) → Harness 持 Arc<dyn LlmProvider> → 每轮 provider.chat
   - src/cli.rs:35-40（审计 #4）
   - src/cli.rs:212-236（审计 #4）
   - src/cli.rs:353-365（审计 #4）
   - src/harness.rs:126-136（审计 #25）
2. 配置链为 CLI > 项目(.codesleuth/config.toml 兼容 ./codesleuth.toml) > 全局(~/.codesleuth/config.toml) > 默认；env 层已整体移除；api_key 只认配置文件直配，缺失报 CONFIG_MISSING 且值不落日志
   - src/config.rs:173-200（审计 #12）
   - src/config.rs:245-252（审计 #12）
   - src/config.rs:456-468（审计 #12）
3. 默认值：base_url=https://api.openai.com/v1、model=gpt-4o-mini、thinking_disabled=true、[vector] 端点 None 跟随 [llm]
   - src/config.rs:107-133（审计 #12）
4. 请求流：build_request 转 async-openai 类型 → JSON 注入 thinking 扩展字段（默认 disabled）→ POST {base_url}/chat/completions；重试仅 429/5xx/网络错误，退避 500ms/1s/2s/4s 封顶，HTTP 超时 300s；错误分类 401/403=CONFIG_MISSING；错误体 safe_prefix 200 字节 UTF-8 安全截断
   - src/llm.rs:220-296（审计 #2）
   - src/llm.rs:223-228（审计 #2）
   - src/llm.rs:304-333（审计 #2）
5. 响应流：map_response 取首个 choice 映射 tool_calls/usage → Harness 记账（turns/tokens/duration）并写审计 llm / llm_error 事件；tool_calls 的 arguments 为原始 JSON 字符串，解析在宿主侧
   - src/llm.rs:183-217（审计 #2）
   - src/harness.rs:136-165（审计 #25）
   - src/llm.rs:45-51（审计 #2）
6. 嵌入端点契约：[vector] base_url/api_key 缺省时跟随 [llm].base_url + 主 api_key，显式覆盖优先（resolve_embed_endpoint）
   - src/cli.rs:138-144（审计 #4）
   - src/cli.rs:519（审计 #4）
7. 落盘：审计与最终报告写 ~/.codesleuth/（reports/{session_id}.md/.json），审计 llm 事件含完整 text；退出码 2=配置/凭据、3=上游 LLM
   - src/cli.rs:428-447（审计 #4）
   - src/cli.rs:1-2（审计 #4）
8. 重试次数 2 为 cli.rs 构造处硬编码、HTTP 超时 300s 为 llm.rs 硬编码，均非配置项（初稿未指出）
   - src/cli.rs:234（审计 #4）
   - src/llm.rs:98（审计 #2）

## 死胡同
- thinking 扩展字段对官方 OpenAI API 的实际行为未验证（推断为被忽略）
- 初稿提到的 grep plain 模式 0 命中，本轮改用 regex 完成

## 置信度
high

## 统计
turns=8 · tool_calls=12 · duration=121329ms · tokens=160308


---

## 附：初稿（已被上方详稿取代，留档比对）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
OpenAI 兼容端点接入是 codesleuth 的 LLM 通道：通过 OpenAI Chat Completions 兼容协议（支持 newapi 等网关）向任意兼容端点发对话/工具调用请求，用户可用 --model/--base-url 覆盖 ~/.codesleuth/config.toml 的 [llm] 配置，从而灵活换供应商。入口在 CLI（src/cli.rs:35-40 定义旗标、src/cli.rs:231-236 构造 provider），实现是 src/llm.rs:81-105 的 OpenAiProvider。调用链：cli run → config::load（应用 CLI 覆盖）→ OpenAiProvider::new → Harness 持有 Arc<dyn LlmProvider> 在每轮对话中调用。下游被 harness 代理循环消费；嵌入端点（[vector]）缺省跟随 [llm] 的 base_url/key（可显式覆盖）。

## 证据列表
1. OpenAiProvider 是 OpenAI 兼容协议实现，注释明示支持 newapi 等网关，持有 base_url/api_key，直发通道可注入网关扩展字段（thinking）
   - src/llm.rs:81-90（审计 #2）
2. CLI 提供 --model / --base-url 覆盖旗标，构造 CliOverrides 传给 config::load 合入生效配置
   - src/cli.rs:35-40（审计 #11）
   - src/cli.rs:212-216（审计 #11）
3. LlmConfig 定义 base_url/model/api_key，默认 base_url 为 https://api.openai.com/v1、model 为 gpt-4o-mini
   - src/config.rs:99-114（审计 #4）
4. 入口用生效配置构造 OpenAiProvider 并以 Arc<dyn LlmProvider> 共享
   - src/cli.rs:231-236（审计 #11）
5. Harness 持有该 provider，作为代理循环中每轮 LLM 调用的执行者
   - src/harness.rs:35-56（审计 #16）
6. 嵌入供应商缺省跟随 [llm] 的 base_url/api_key，显式 [vector] 覆盖优先（有测试 embed_endpoint_fallback_follows_llm 佐证）
   - src/cli.rs:138-142（审计 #11）

## 死胡同
- grep 'base-url|base_url' plain 模式初查 0 命中（正则字符类问题），改用 regex 后命中

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=18995ms · tokens=35560
