# codesleuth · 总体框架（初稿）

> docs-init 第一步骨架侦察产物 · codesleuth · 2026-10-06

## 结论

codesleuth 是一个 Rust 实现的只读代码侦察 Agent CLI：用户输入自然语言问题 + 目标仓库路径，Agent 自主检索（结构图/模糊搜索/强读/语义召回）并输出带 file:line 证据、可审计回溯的结构化报告，全程对目标仓库零写入。面向需要在大型代码库中快速定位代码并要求答案可验证的开发者与上游 Agent。

## 证据列表

1. 项目定位：Rust 只读侦察 Agent CLI，问题进、带证据报告出，对目标仓库零写入（接口无写能力 + 路径围栏 + 前后快照比对自证）。入口 `codesleuth "<task>" --repo <path>`。
    - README.md:22-36（审计 #2）
    - src/cli.rs:14-55（审计 #7）
2. 大体框架：CLI 层（cli.rs）→ Agent 主循环 Harness（harness.rs：LLM 决策→工具调度→收敛/熔断）→ 工具注册表（tools/：read、fuzzy、graph、vector_search）→ MCP 客户端（mcp.rs，对接 codegraph 结构图服务）+ 向量子系统（vector/：chunk/embed/store/repomap）→ 支撑层：audit、context、fence、writeguard、bootlock、llm、report、config、errors。
    - src/cli.rs:57-93（审计 #7）
    - src/harness.rs:1-33（审计 #15）
    - src/tools/mod.rs:1-59（审计 #10）
    - src/context.rs:1-54（审计 #17）
3. 功能域：CLI 命令面与配置管理。`config get/set/path` 查看/写入 ~/.codesleuth/config.toml（llm、vector、context、behavior 段），`index <path> --vector` 构建向量索引。
    - src/cli.rs:57-93（审计 #7）
4. 功能域：Agent 侦察执行。主循环驱动 LLM 调用九个工具（explore/callers/callees/impact/files/find_files/grep/read/vector_search/submit_report/recall），支持 --repo/--focus/--json/--out/--fresh-index/--vector/--repo-map 等旗标。
    - src/harness.rs:1-23（审计 #15）
    - README.md:108-119（审计 #2）
5. 功能域：检索工具面（只读）。read（强读+锚点+分页）、find_files/grep（plain/regex/fuzzy 三态）、explore/callers/callees/impact/files（codegraph 结构图），接口上不存在写能力。
    - src/tools/mod.rs:1-8（审计 #10）
    - README.md:110-119（审计 #2）
6. 功能域：向量语义检索。切块→嵌入（raw/composite 模式）→SQLite 落地，text_hash 增量复用+失效块 GC；vector_search 语义召回；repo map 预算化导航图注入（默认 24000 字符）。索引存于 `<project>/.codesleuth/`，嵌入可独立供应商配置。
    - src/vector/build.rs:1-40（审计 #31）
    - src/vector/repomap.rs:1-12（审计 #29）
    - src/cli.rs:128-178（审计 #7）
7. 功能域：上下文与长任务管理。窗口阈值触发确定性三段式压缩（持久化→handoff→压缩，零 LLM），recall 工具从审计原文不重不漏续读；空转熔断（连续无进展 5 步）与零增量转向（2 回合）。
    - src/context.rs:1-80（审计 #17）
    - src/harness.rs:18-23（审计 #15）
8. 功能域：报告与证据校验。submit_report 结构化报告（schema 版本化，答案/findings/dead_ends/confidence/stats），证据必须来自会话真实读取否则拒收；prose 降级路径诚实标注 degraded=true。
    - src/report.rs:1-50（审计 #26）
    - src/harness.rs:1-3（审计 #15）
9. 功能域：审计流水。全量 JSONL 审计（seq 单调），报告 ↔ 审计双向回溯（evidence.audit_seq），recall 以 seq 范围续读原文。
    - src/audit.rs:1-60（审计 #22）
    - README.md:34-35（审计 #2）
10. 功能域：只读边界与安全。路径围栏（词汇预检+symlink canonicalize 复检拒越界）、per-file manifest 零写入自证（快照比对+写面归因）、MCP 进程生命周期回收（Drop）、bootlock 引导锁防并发索引竞争。

- src/fence.rs:1-56（审计 #20）
- src/writeguard.rs:1-60（审计 #24）
- src/mcp.rs:67-102（审计 #12）
- src/cli.rs:157-162（审计 #7）

11. 功能域：LLM 接入与运维。OpenAI 兼容端点（--model/--base-url 可覆盖）、结构化错误与退出码体系（0-6）、-v 日志详细度、CI 脚本与多语言 fixture 评测（scripts/eval 硬门+judge 判分）。

- src/mcp.rs:1-16（审计 #12）
- src/cli.rs:1-2（审计 #7）
- README.md:124-130（审计 #2）

## 死胡同

- docs/ 下旧文档未展开阅读（任务书声明仅作参考，功能清单已由源码侦察覆盖）
- src/tools/{fuzzy,read,graph,vector_search}.rs 内部实现未逐个深读（概览任务，工具面信息已从 README+mod.rs+harness 获得足够广度）

## 置信度

high

## 统计

turns=7 · tool_calls=13 · duration=32783ms · tokens=83914
