# codesleuth · 总览

> docs-init 第三步总览升级 · 2026-10-07

> 全局架构图：`00-architecture.html`

## 项目定位

codesleuth 是一份面向"陌生仓库快速结构化侦察"场景的只读 agent 单二进制 CLI。它把一次任务以"主循环 + 三段式上下文压缩 + 提交型 Report"的方式落地为结构化 Report 与人类可读 answer 双份产物，并在路径越界、并发写、凭据泄露、跨进程索引竞态四类风险面上以 Fence / bootlock / writeguard / 审计 围栏钉死。核心价值是给 LLM 驱动的代码理解工具提供一份"安全可控、可审计、可回灌、可压缩"的工作底盘：模型始终在只读工具面内行动，所有 host 级事件落 JSONL 事实账本，被压缩驱逐的原文可经 recall 工具按审计 seq 续读。

## 全局架构图

按"入口 → 装配 → 主循环 → 旁路索引与安全面"四层组织。入口层 main() 调 init_tracing 把 EnvFilter + stderr/file 两层 fmt::layer 挂到同一 registry 并生成 session_id，再由 Cli::run 按子命令派发到 run_config / run_index_vector / run_task 三轨；配置轨 run_config 经 config::load 把 CLI 覆写、~/.codesleuth/config.toml、项目 .codesleuth/config.toml、default 四层合并为运行时 Config；索引轨 run_index_vector 在 bootlock 保护下走 plan_chunks → compose_input → EmbedClient.embed → commit_build 单事务把 chunk 落 .codesleuth/indexes/<fingerprint>/vectors.db。任务轨 run_task_inner 装配 audit + fence + ToolRegistry（ReadTool / FileFinderTool / GrepTool + 可选 VectorSearchTool + codegraph 五个 MCP Tool）+ RecallEngine，向 Harness 注入 SYSTEM_PROMPT 首条 system 消息、任务 User 正文以及 first_user_suffix（vector recall 起步线索与 [repo map] 段），随后 Harness::run 启动主循环：每轮做 should_compact → provider.chat → 工具分发（按 name 查 ToolRegistry.get → Tool::execute）→ 增量记账 EvidenceStore.observe + Audit::record，最终以 submit_report → build_report 收敛或 Report::degraded_prose 兜底，回填 RunOutcome 由 Cli 同时落 ~/.codesleuth/reports/<sid>.{md,json}。旁路安全面与 codegraph 子进程由 bootlock::acquire_guard 串行化、独占 Arc 持有并在 Drop 时回收，回填文本经 extract_tool_text 拼成 `[codegraph <tool>] <label>\n<out>` 回注主循环；日志双轨与审计 JSONL 共享同一 session_id 身份贯穿全程。

## 功能域导航

- docs/contexts/CLI与命令层/01-cli-dispatch.md：main → init_tracing → Cli::run 按子命令分派 config / index --vector / run。
- docs/contexts/CLI与命令层/02-config-cmd.md：config 子命令 run_config 三分支 Get/Set/Path 读写 ~/.codesleuth/config.toml。
- docs/contexts/CLI与命令层/03-index-cmd.md：index --vector 写侧 run_index_vector 编排 bootlock → plan_chunks → embed → commit_build。
- docs/contexts/LLM与配置/01-llm-provider.md：LlmProvider trait + OpenAiProvider 实现统一 chat 接口与重试退避。
- docs/contexts/LLM与配置/02-config-loading.md：config::load → load_layered 四阶段（CLI > project > global > default）合并。
- docs/contexts/上下文与报告/01-context-compaction.md：Harness 每轮按"模型窗口×百分比"做零 LLM 三段式收敛，recall 工具按审计 seq 续读被驱逐原文。
- docs/contexts/上下文与报告/02-report.md：Report schema 装配与 degraded_prose 兜底，render_human 渲染 answer，Cli 落 md/json 双份。
- docs/contexts/侦察编排/01-harness.md：Harness::run 主循环：压缩判断 → LLM 决策 → 只读工具调度 → 增量记账 → submit_report / 熔断。
- docs/contexts/侦察编排/02-system-prompt.md：SYSTEM_PROMPT 常量把"只读身份 + 行为边界 + 无空转 + 输出契约"钉入首条 system 消息。
- docs/contexts/只读工具面/01-tool-registry.md：ToolRegistry 统一寻址 + 模式导出（name 索引 + ToolSchema 列表）。
- docs/contexts/只读工具面/02-read-tool.md：read 工具按 offset/limit 分页读取 Fence 围栏内文件，输出"行号:锚点|内容"。
- docs/contexts/只读工具面/03-fuzzy-search.md：find_files / grep（plain/regex/fuzzy 三态）由 fff-search FilePicker 同步索引驱动。
- docs/contexts/只读工具面/04-vector-search-tool.md：vector_search 工具把自然语言问题经 RecallEngine 映射到 top-K 代码位置指针。
- docs/contexts/向量检索/01-vector-embed.md：EmbedClient 调 Qwen3 兼容 /embeddings 端点，输出 1024 维 Matryoshka 向量。
- docs/contexts/向量检索/02-vector-chunk.md：plan_chunks 按"主块 / 超大滑窗 / 兜底"三层规则把符号切成可嵌入 Chunk。
- docs/contexts/向量检索/03-vector-store.md：vectors.db 持久化 chunk 元数据与嵌入，HNSW 会话内影子，规模自适应切换。
- docs/contexts/向量检索/04-vector-recall.md：RecallEngine 单条嵌入后三段式召回（HNSW k×3 超采 → cosine 复算 → alive 过滤）。
- docs/contexts/向量检索/05-vector-compose.md：compose_input 按 EmbedMode（Raw / Composite）拼嵌入模型输入，cap=20。
- docs/contexts/向量检索/06-repo-map.md：build_repo_map / build_task_map 按 24_000 字符预算做度数中心度贪心，段以 first_user_suffix 注入。
- docs/contexts/安全与防护/01-fence.md：Fence 词汇预检 + dunce canonicalize 复检两道关卡，越界返 CS3003。
- docs/contexts/安全与防护/02-bootlock.md：bootlock 跨进程 flock 串行化 codegraph 引导与 vector 构建。
- docs/contexts/安全与防护/03-writeguard.md：writeguard::snapshot + diff 自证任务起止无可归因变更。
- docs/contexts/安全与防护/04-audit.md：审计 JSONL 按 seq 单调追加宿主级事件，recall 续读与 G2 校验的唯一事实锚点。
- docs/contexts/安全与防护/05-evidence.md：EvidenceStore 登记 observe 过的路径与首次 seq，submit_report 校验锚点。
- docs/contexts/结构图与MCP/01-graph-tools.md：CodegraphEngine 引导 codegraph 子进程并暴露 explore/callers/callees/impact/files 五个 Tool。
- docs/contexts/结构图与MCP/02-mcp-client.md：McpClient + newline JSON-RPC stdio 通道，protocolVersion 2025-03-26，90s 超时。
- docs/contexts/错误与日志/01-error-codes.md：CSxxxx 段位体系 + report_error 三行 stderr + 按段映射 exit code。
- docs/contexts/错误与日志/02-logging.md：init_tracing 同 EnvFilter 双轨（stderr + ~/.codesleuth/logs/<session>.log），64MB 单代轮转。

## 全局交互

各功能域围绕"Harness::run 主循环 + session_id 单点身份"协同：CLI 入口在同一 session_id 下同时挂日志双轨（init_tracing）与审计 JSONL（Audit::create），run_task_inner 把 Config 注入所有需要 [llm]/[vector] 的子组件；Harness 持有 Arc<dyn LlmProvider> 与 ToolRegistry，按 provider.chat 拿到 tool_calls 后以 name 查 reg.get → Tool::execute 分发到 read / fuzzy / codegraph / vector_search / recall，每条 tool_call / tool_result 经 Audit::record + EvidenceStore.observe 双写；codegraph 子进程由 bootlock::acquire_guard 串行化引导、由 McpClient 独占持有并在 drop 时回收，回填文本经 extract_tool_text 拼装回主循环；向量通道两侧共用 EmbedClient，build 端走 plan_chunks + commit_build 落 vectors.db，recall 端走 EmbedClient::instruct_query + RecallEngine 三段式，format_recall_block 既被 VectorSearchTool 工具消费、也被 first_user_suffix 一次性注入主循环；上下文压缩触发时 build_handoff 以 Audit::last_seq 起点串联被驱逐原文、由 recall 工具按 seq 区间从 JSONL 钻取回灌；所有错误经 CsError 五字段归一，由 report_error 三行打 stderr 并按 e.exit_code() 退出，最终 RunOutcome 由 Cli 同时落 md/json 两份报告。

## 未解问题

### 跨详稿冲突（已列双方说法，未裁决）

- run_index_vector 行号范围：初稿记 `src/cli.rs:109-115`（仅 match 分支入口），详稿记 `src/cli.rs:109-127`（含完整 if/else 至 127 行）。
- HNSW 阈值：详稿 `MIN_HNSW_SIZE=100_000`（10 万）走 HNSW、否则暴力；初稿记 "<1000 向量走暴力"，数量级差两个数量级。
- EmbedClient 默认 dimensions：初稿称 EmbedClient 默认 1024；详稿记 EmbedClient::new 构造时不设默认，1024 来自 `cfg.vector.embed_dims` 默认值。
- [repo map] 注入位置：src/vector/repomap.rs:1-4 头部注释与 wrap_repo_section 函数注释均称"注入到 system prompt 尾部 [repo map] 段"；详稿据 `src/harness.rs:74-88` 实际拼到首条 User 消息后缀 first_user_suffix。
- codegraph 独占机制：初稿说"独占进程靠 `CODEGRAPH_NO_DAEMON=1`"；详稿记仅存于 `src/tools/graph.rs:1-5,70` 文档注释，代码侧无 `.env()` 调用、独占靠 Arc 所有权 + Drop。
- CS1xxx 退出码映射：初稿 "CS1xxx→1"；详稿记 1000-1009→1 用法、1010-1099→2 配置/凭据。
- classify_llm_error 位置：初稿列为 LLM 分类器；详稿记是 `#[cfg(test)]` 测试辅助、生产分类内联在 OpenAiProvider::chat。
- writeguard Modified 语义：字面 Modified 应识别内容或元数据变更；详稿记 mtime 不参与指纹、"重写后内容相同"不被报为 Modified。
- McpClient 协议字段：详稿纠错 `protocolVersion="2025-03-26"`、`next_id=AtomicU64`、`reader=tokio::sync::Mutex<BufReader<ChildStdout>>`、`timeout_at` 包裹单次 `read_line`、索引引导为 `index --force --quiet`、`list_tools` 不缓存；初稿以 "整体 90s 包裹 read_line"、"走 init 前置"、"list_tools 缓存"表述（具体初稿表述未保留，详稿已显式标注差异）。
- Harness::run 调用链：初稿隐含 main→cli.run→Harness::run；详稿纠错 main→Cli::run→run_task→Harness::run。
- harness 详稿证据完整度：详稿「证据列表」自承"无结构化发现——降级报告"；初稿记 12 条证据及 high 置信度。

### 推断（待验证，不裁决）

CLI/配置：CliOverrides 仅 base_url/model/profile 三项的限制与日志过滤级别默认值的具体来源；config_get 构造 CliOverrides 仅传 profile 而 base_url/model 走 Default、与 Cli::run 顶层旗标联动存疑；`toml::to_string_pretty` 整文件覆盖丢注释；`config.rs` keys_hint 实际命中数与 13 写键全表未在详稿中列全；`apply_profile` "项目层同名覆盖全局"语义依赖 BTreeMap extend 顺序；CONFIG_MISSING/CONFIG_INVALID 完整映射规则未读全；`scripts/eval/run_eval.py:110-118` 仅作向量召回线索未读原文。

LLM：thinking_disabled 与 OpenAI 协议 thinking 字段对齐方式；CS2xxx 段位（2001/2002/2003/2004/2099）具体语义；vector 嵌入端点跟随 llm.base_url 的去重规则。

Harness 与 Prompt：build_handoff 唯一生产调用方显式证据；Harness 运行时是否自带 spawn/local_set；System Prompt 无运行时配置/环境变量开关/变体；Harness::run 跨异步边界补遗点；ToolRegistry 无去重/校验机制、无全局/共享存储；VectorSearchTool 通过测试夹具注册但非 cli 默认装配。

Read / Fuzzy：read 工具二进制探测仅扫前 8192 字节、超长二进制可能误判为文本；limit 静默 `clamp(1,2000)` 限幅无错误提示；Harness 旁路 `observe_exact` 行号；`src/prompt.rs:42-43` 实际是 `prompt_contains_contract_anchors` 断言数组元素而非契约锚点；cli.rs:244-250 与 cli.rs:359-371 关于 `Arc<Fence>` 在 ReadTool 与 FuzzyEngine 之间共享顺序未交叉验证；FuzzyEngine 内自建 Fence 仅取 `root()` 作 base_path、未走 resolve 守门、模糊检索阶段是否受 Fence 二次约束存疑；`Fence::new` / `fence.resolve` 唯一生产 resolve 调用点仅 read 一处。

Vector 全链路：RecallEngine::recall "HNSW k×3 超采→余弦重算→墓碑过滤"三段式实现细节未深入；setup_vector_layer 完整前文（cli.rs:665 之前）未通读；HNSW 常量 `M=16 / MAX_LAYER=16 / EF_CONSTRUCTION=200 / EF_SEARCH=64 / OVERFETCH_FACTOR=3 / MIN_HNSW_SIZE=100_000 / TOMBSTONE_REBUILD_RATIO=0.2` 未带完整行号；chunk_key 完整拼接表达式（推断含 file+symbol+line_start）；commit_build 事务边界行号；BuildReport 字段语义靠 path:line 推断；tombstone/compact 路径当前仓库无生产调用方、callers 仅命中测试位点，疑为预留入口；暴力分支与 HNSW 分支 cosine 复算是否走同一余弦公式；compose_input 查询侧走 instruct_query 指令前缀与构建侧嵌入空间天然不对称、mode 字段不强制对齐仍可工作；非 "raw" 字符串静默回落 Composite、无显式错误；cap=20 硬编码而非配置项；`relations_for_symbol` 实现未直接读、仅凭调用点推断返回 (callers, callees)。

MCP / Codegraph：extract_tool_text 把 isError=true 与空内容均视为"符号不存在或索引未就绪"宽松降级；90s DEFAULT_TIMEOUT 对大仓库 impact/explore 是否足够；codegraph 二进制不在仓库内、启动失败仅 audit "degraded" 留痕不重试；`list_tools` 每次都请求 `tools/list` 不缓存；`timeout` 字段默认为 90s 但与 `timeout_at` 实际取值关系未交叉验证。

写面/审计/证据：writeguard 死胡同——`audit::append_line` 落盘路径与 seq 单调不重实现细节仅到 `src/cli.rs:391`；`sha2::Sha256` use 声明未显式读到；symlink 在 ">1MB 之后内容修改、首 64KB 相同" 场景下指纹不重会漏检；`append_line` 走"读全文取 max+1"与 `Audit::record` 锁内 `fetch_add+写入` 不互锁、缺乏结构性保护；`Audit::record` `flush` 失败仅 warn；64MB 单代轮转到 `.jsonl.old` 后 old 文件被覆盖前的窗口期行为；`read_range` 每次重读全文件、长会话下 I/O 成本无缓存；read 工具分支精确调用 `observe_exact` 行号；path_like 过滤函数定义；current_seq 注入 EvidenceStore 的具体行号。

错误/日志：tool 层 read/fuzzy/graph 错误列举在详稿末尾被截断；`--json` 旗标在 main 中的显式传递路径未读到具体 if 分支。

### 存疑（需补读源码）

- `config.rs` keys_hint 实际命中数与 13 写键全表（`src/config.rs:469-485` 续读）。
- plan_chunks layer 2（按文件聚合）与 layer 3（兜底 walk）完整判定逻辑与字符上限（`src/vector/chunk.rs:258-474`）。
- `reports/<session>.md|.json` 报告写盘函数 grep 0 命中，落点仅由 D013 决策与 AGENTS.md 间接确认。
- Report::validate 的具体位置与失败分支细节（src/report.rs 字面量构造之后）。
- 详稿在 src/vector/recall.rs:13- 末尾被截断的 HNSW 常量行号。
- 详稿 src/vector/repomap.rs "证据 #9 末尾 index --force --quiet" 处被截断。
- 详稿 bootlock "debug Won（src/bootlock.rs:75）/" 处 trace 事件表与 Lost 归 INDEX_LOCKED 的具体日志分支。
- 详稿 error-codes 末尾 "tool 层 read/fuzzy/graph 错误列举" 第 15 条证据未闭合。
