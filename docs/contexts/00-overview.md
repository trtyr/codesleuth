# codesleuth · 总体框架（初稿）

> docs-init 第一步骨架侦察产物 · codesleuth · 2026-10-07

## 结论
codesleuth 是一个 Rust 实现的只读代码侦察 Agent CLI（README.md:24, Cargo.toml:2-6）。面向需要在大仓库中可审计地定位代码的开发者与上游 Agent：工具面物理无写能力、路径围栏拒 symlink 逃逸、运行前后逐文件比对自证零写入（README.md:26, src/fence.rs:1-2, src/writeguard.rs:1-4）。结构上由 CLI 入口（src/main.rs:5-12, src/cli.rs:20-58）派发到三类动作：自然语言侦察任务走 Harness 主循环（src/harness.rs:74 Harness::run），配置子命令走 config（src/config.rs），索引子命令走 vector 构建（src/vector/build.rs:25）。Harness 编排只读工具面（read/find_files/grep/explore/callers/callees/impact/files/vector_search/recall/submit_report，见 src/tools/mod.rs:1-2 与 src/cli.rs:247-294、603），通过 LLM 抽象（src/llm.rs:18-22 OpenAI 兼容）调度；结构图工具经 MCP 客户端（src/mcp.rs:19-25）连接外部 codegraph CLI 进程；向量检索覆盖嵌入（src/vector/embed.rs:23）、切块（src/vector/chunk.rs:1-6）、SQLite 真相（src/vector/store.rs:19-27）、召回引擎（src/vector/recall.rs:34）、组装（src/vector/compose.rs:9-12）、repo map 预算注入（src/vector/repomap.rs:1-4, src/harness.rs:69-72）六个子层。安全边界由四道闸组成：Fence 路径围栏（src/fence.rs:30）、bootlock 跨进程引导锁（src/bootlock.rs:50+）、writeguard per-file 快照自证（src/writeguard.rs:29）、审计 JSONL+证据库账本（src/audit.rs:21, src/evidence.rs:28-37）。上下文侧用三段式压缩（持久化+handoff+压缩，src/context.rs:31-37），报告走版本化 schema+人类渲染+degraded 降级（src/report.rs:33-43），错误用 CS-段位码（src/errors.rs:13-22），日志双轨（stderr+文件归档，src/lib.rs:27、src/logs.rs:11）。

## 证据列表
1. 项目定位：只读代码侦察 Agent CLI，对目标仓库零写入。
   - README.md:1-30（审计 #7）
2. 入口 main.rs 解析 CLI、生成 session_id、初始化日志后调用 Cli::run。
   - src/main.rs:1-12（审计 #9）
3. CLI 由 Cli 结构 + Command 子命令（Config/Index） + ConfigAction（Get/Set/Path） 组成，附 model/profile/vector/repo_map/fresh_index 等旗标。
   - src/cli.rs:14-96（审计 #11）
4. Cli::run 三分支：Config 子命令、Index 子命令（支持 --vector）、无子命令走 run_task 进入 Harness。
   - src/cli.rs:99-130（审计 #11）
5. lib.rs 列出全部 16 个子模块并提供 init_tracing：stderr + ~/.codesleuth/logs/<session>.log 双轨。
   - src/lib.rs:1-66（审计 #14）
6. 工具注册表：Tool trait（只读 execute）+ ToolRegistry，物理上无写能力，是只读边界第一道闸。
   - src/tools/mod.rs:1-59（审计 #18）
7. read 工具：行号+行哈希锚点、分页、二进制识别、编码消毒；唯一事实来源。
   - src/tools/read.rs:1-80（审计 #21）
8. find_files/grep（fuzzy 工具）基于 fff-search 引擎，find_files = frecency 路径搜索，grep = plain/regex/fuzzy 三态。
   - src/tools/fuzzy.rs:1-60（审计 #25）
9. vector_search：自然语言问题 → RecallEngine 语义召回，结果须过 read 才可作证据。
   - src/tools/vector_search.rs:1-60（审计 #27）
10. 结构图工具 explore/callers/callees/impact/files：经 McpClient 连接外部 codegraph serve --mcp，会话独占、Drop 回收。
   - src/tools/graph.rs:1-80（审计 #23）
11. ExploreTool name = "explore"；五工具在 cli.rs:290-294 注册。
   - src/tools/graph.rs:185-225（审计 #23）
12. 工具注册现场：read/FileFinder/Grep 在 cli.rs:247-250，结构图五件套在 290-294，vector_search 在 603。
   - src/cli.rs:247-298（审计 #11）
13. Fence 路径围栏：词汇层预检 + canonicalize 解 symlink 复检，越界返 CS3003。
   - src/fence.rs:1-60（审计 #30）
14. bootlock：<repo>/.codesleuth/boot.lock 跨进程 flock，三态 Won/Lost/超时；引导段结束即释放。
   - src/bootlock.rs:1-50（审计 #38）
15. writeguard：per-file manifest 快照 + 写面归因，codesleuth 在目标仓可写面为空集；不可归因变更如实上报。
   - src/writeguard.rs:1-50（审计 #36）
16. 审计：append_line（宿主级事件续号）+ new_session_id（时间+pid hex）。
   - src/audit.rs:1-60（审计 #32）
17. 证据库：observe_exact（read 专用）+ observe（路径形 token 提取），cite_seq 报告↔审计互查锚点。
   - src/evidence.rs:1-56（审计 #34）
18. McpClient：spawn 子进程 + newline-delimited JSON-RPC（initialize/tools_list/tools_call）+ Drop 回收。
   - src/mcp.rs:1-40（审计 #40）
19. 报告：版本化 schema（REPORT_SCHEMA_VERSION=1）+ degraded 降级（confidence=low，findings=空）+ stats 字段。
   - src/report.rs:1-60（审计 #42）
20. 上下文：threshold=窗口×百分比；三段式持久化→handoff→压缩；KEEP_RECENT=6。
   - src/context.rs:1-40（审计 #44）
21. LLM provider 抽象：OpenAI 兼容 + 自定义 base_url，重试/退避在本层做。
   - src/llm.rs:1-40（审计 #46）
22. 配置加载链：CLI 参数 > 项目配置 > 全局配置 > 默认值；环境变量层已整体移除。
   - src/config.rs:1-40（审计 #48）
23. 错误码段位：CS1xxx→exit 1（用法） · CS2xxx→3（LLM） · CS3xxx→4（目标库） · CS4xxx→5（索引） · 其它→6。
   - src/errors.rs:1-40（审计 #50）
24. System Prompt 定稿：只读身份 + 检索纪律（层进协议）+ 无空转 + 输出契约，作为产品资产随发行版走。
   - src/prompt.rs:1-58（审计 #52）
25. 日志：SessionLog 单代轮转（64MB），同过滤同格式双轨，session span 串线。
   - src/logs.rs:1-30（审计 #54）
26. 向量检索六子层：embed/chunk/store/recall/compose/repomap 全部在 vector 模块下公开。
   - src/vector/mod.rs:1-21（审计 #57）
27. build_vector_index 入口：切块→组装→嵌入→SQLite 落地；text_hash 未变 chunk 整体复用。
   - src/vector/build.rs:1-40（审计 #59）
28. RecallEngine：≤10万向量走暴力，HNSW 仅留巨型仓；三段式 HNSW 超采→精确复算→墓碑过滤。
   - src/vector/recall.rs:1-40（审计 #61）
29. SQLite 真相原则：vectors.db 是嵌入索引唯一持久真相；HNSW 是会话内影子。
   - src/vector/store.rs:1-40（审计 #63）
30. 切块策略：codegraph 符号表为边界真源 + 三层规则（主块 / 超大滑窗 / 兜底）。
   - src/vector/chunk.rs:1-30（审计 #67）
31. 嵌入客户端：默认维度 1024（Matryoshka 降维），批并发 4，查询侧加 Instruct、文档侧不加。
   - src/vector/embed.rs:1-30（审计 #69）
32. 组装模式：EmbedMode = Raw | Composite（白拿层 header + 标识符层去重封顶）。
   - src/vector/compose.rs:1-20（审计 #71）
33. repo map 预算化注入：默认 24k 字符（≈6k token），度数中心度贪心装填，写入 system 尾部 [repo map] 段。
   - src/vector/repomap.rs:1-30（审计 #65）
34. Harness 主循环：熔断阈值 MAX_NO_PROGRESS_STREAK=5、零增量转向阈值=2；system 恒定，任务派生数据走首条 user 后缀。
   - src/harness.rs:1-120（审计 #16）
35. Cargo 依赖：async-openai（LLM 协议）、fff-search（模糊）、hnsw_rs（ANN）、rusqlite（SQLite）、tracing/tracing-subscriber（日志）、clap（CLI）、reqwest、tokio、serde。
   - Cargo.toml:1-38（审计 #7）

## 死胡同
- grep 工具自身的 name() 字符串未在本会话内 read 原文直接命中（仅通过 src/cli.rs:250 的注册与 src/tools/fuzzy.rs 同文件结构推断），但 README.md:121-128 与 src/cli.rs 工具表已交叉确认其作为工具面存在。
- tests/fixtures/ 与 scripts/eval/ 仅做列目参考，未深入读取；它们是评测/测试支撑材料，不是本概览的功能点主体。
- docs/plantree/ 路径在 src/lib.rs:3-4 注释中标为本地规划态、.gitignore 排除，未读其内容（按红线 B 仅作参考）。

## 置信度
high

## 统计
turns=12 · tool_calls=41 · duration=130540ms · tokens=308070
