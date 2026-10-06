# Agent 主循环 Harness（Agent 侦察执行）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
Harness（src/harness.rs）是 codesleuth 的 Agent 自主侦察主循环：接收用户任务后驱动「LLM 决策 → 工具调度 → 增量记账 → 收敛/熔断」的多轮循环，直到模型调用 submit_report 提交带证据校验的结构化报告（或 prose 降级），为调用方交付带审计留痕的 RunOutcome。入口为 src/cli.rs:353-365（run_task_inner 构造并 block_on agent.run），核心循环在 src/harness.rs:74 的 Harness::run。上游依赖：LLM 接入（src/llm.rs 的 LlmProvider）、工具注册表（src/tools/mod.rs 的 ToolRegistry，schemas() 注入工具面、get() 分发执行，src/harness.rs:130,331）、审计（src/audit.rs）、上下文压缩（src/context.rs，src/harness.rs:100-124）、首条消息导航图后缀（with_first_user_suffix，src/harness.rs:69）。收敛通道为内置 submit_report 工具（src/harness.rs:224-250 经 build_report 出 Report）；空转熔断 fuse_if_hit 在连续 5 步无进展时返回 LLM_FUSE（src/harness.rs:19,227,331-333），连续零增量注入转向指令（harness.rs:781-804），上下文超阈触发确定性压缩+handoff（harness.rs:100-124）。下游消费：RunOutcome 交回 cli.rs 渲染输出，Report/Evidence 由 src/report.rs、src/evidence.rs 承载。

## 证据列表
1. Harness 是 Agent 主循环：LLM 决策→工具调度→增量记账→收敛/熔断，收敛 = submit_report 或 prose 降级，对外产出 RunOutcome（report/answer/turns/tool_calls/audit_path）
   - src/harness.rs:1-3,74（审计 #2）
2. 入口在 CLI：run_task_inner 构造 Harness（provider/registry/audit/model/上下文参数）并 block_on agent.run(&task)
   - src/cli.rs:353-365（审计 #14）
   - src/harness.rs:46-65（审计 #2）
3. 主循环每轮：超阈值先 context 压缩+handoff，再组装 ChatRequest（注册工具 schemas + 内置 submit_report/recall schema）调 provider.chat
   - src/harness.rs:99-136（审计 #2）
4. 工具调度：循环遍历 resp.tool_calls，submit_report 走 build_report 收敛，普通工具经 self.tools.get() 分发执行
   - src/harness.rs:222-250（审计 #2）
   - src/harness.rs:331-333（审计 #2）
5. 熔断与转向：连续 5 步无进展（MAX_NO_PROGRESS_STREAK）经 fuse_if_hit 返回 LLM_FUSE；连续零增量注入「无新信息」转向指令
   - src/harness.rs:18-21（审计 #2）
   - src/harness.rs:227,331-333（审计 #2）
   - src/harness.rs:781-804（审计 #2）
6. 上游/下游交互：依赖 LlmProvider（src/llm.rs）、ToolRegistry（src/tools/mod.rs）、Audit（src/audit.rs）、context 压缩（src/context.rs）；RunOutcome 交回 cli.rs 渲染，模块全景见 docs/00-overview.md
   - src/cli.rs:353-360（审计 #14）
   - docs/00-overview.md:12（审计 #29）

## 死胡同
无

## 置信度
high

## 统计
turns=8 · tool_calls=11 · duration=36230ms · tokens=70038
