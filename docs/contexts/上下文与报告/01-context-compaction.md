# 上下文压缩（上下文与报告）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
上下文压缩（D013 v2）由 Harness::run 主循环驱动：每轮用 模型窗口 × 百分比（默认 1M × 60%）算阈值，由 should_compact 判定；触发后三段式 —— ①调 build_handoff 拼承上启下文本（零 LLM）→ ②调 compact 保留头部 2 条 + handoff + 最近 KEEP_RECENT=6 条、中间驱逐 → ③写 compaction_begin / compaction 审计留痕；被驱逐的原文通过 recall 工具按审计 seq 范围续读，不重不漏。

## 证据列表
1. 压缩触发判定在 Harness::run 主循环内：算阈值 → should_compact → 三段式执行。
   - src/harness.rs:100-109（审计 #16）
   - src/harness.rs:102（审计 #16）
   - src/harness.rs:103（审计 #16）
2. 阈值函数为「模型上下文 × 百分比」，percent>100 按 100 封顶。
   - src/context.rs:30-33（审计 #2）
   - src/context.rs:109-114（审计 #2）
3. 承上启下 handoff 是确定性拼装、零 LLM，先于压缩生成，包含任务重述、recall 范围指引、下一步指引。
   - src/context.rs:44-54（审计 #2）
   - src/harness.rs:104-107（审计 #16）
4. compact 纯函数保留头部 2 条 + handoff User 消息 + 最近 keep_recent 条，并保证 tool/assistant 配对不被裁剪边界拆散。
   - src/context.rs:56-82（审计 #2）
   - src/context.rs:131-152（审计 #2）
   - src/context.rs:162-195（审计 #2）
5. KEEP_RECENT=6 为保留尾部消息条数常量。
   - src/context.rs:8-9（审计 #2）
6. 真正产生驱逐时（evicted_count>0）才写 compaction_begin / compaction 审计；no-op 时不产噪音。
   - src/harness.rs:110-123（审计 #16）
7. recall 工具通过 context::recall_message 把审计原文按 seq 升序包装为 User 消息回灌，承接被驱逐内容的不重不漏续读。
   - src/context.rs:84-94（审计 #2）
   - src/harness.rs:288（审计 #16）
   - src/harness.rs:1020（审计 #16）
8. callers 图确认 compact / should_compact / build_handoff / compact_threshold_tokens 的唯一生产调用方均为 Harness::run（除测试）。
   - src/harness.rs:74（审计 #16）
9. D013 决策文档规定三段式时序与「窗口是物理口径」原则，并要求报告落盘到 reports/<session>.md|.json。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/013-compaction-v2-report-persistence.md:15-18（审计 #4）
10. 状态目录约定：~/.codesleuth/ 统一放 reports/audit/ledger/eval；上下文压缩必须确定性、零 LLM。
   - AGENTS.md:11（审计 #51）
   - AGENTS.md:14（审计 #51）
   - src/config.rs:164（审计 #53）
11. RunOutcome 携带 Report、answer、turns、tool_calls、audit_path，Harness 持有 context_tokens 与 compact_percent 配置。
   - src/harness.rs:25-44（审计 #16）
   - src/harness.rs:60-64（审计 #16）

## 死胡同
- grep "write_markdown|finish_report|report_path|finish_session" 0 命中——报告持久化写盘逻辑未在本次会话被读到（仅由 D013 决策与 AGENTS.md/config.rs 间接确认落点）。
- grep "pub async fn run" 仅 1 命中，RunOutcome 字段确认，但持久化报告的写文件实现在 harness.rs 后续行未逐行读完。

## 置信度
high

## 统计
turns=13 · tool_calls=20 · duration=52159ms · tokens=125652

