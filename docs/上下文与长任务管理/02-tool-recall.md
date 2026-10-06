# recall 续读工具（上下文与长任务管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
recall 是 harness 暴露给 LLM 的审计续读工具：按 [from, to] seq 范围从 JSONL 审计日志读回被压缩驱逐的原文行（升序、不重不漏），格式化为带 #seq 标注的用户消息回填上下文，支撑长任务压缩后的历史钻取。调用链：LLM 发起 RECALL_TOOL 调用 → harness 主循环拦截 → Audit::read_range 过滤排序 → context::recall_message 包装 → push 回 messages。上游依赖审计流水（record/last_seq）与上下文压缩 handoff（提供 recall 范围指引），下游消费是主循环上下文与空转/零增量统计。

## 证据列表
1. recall 工具名为 RECALL_TOOL="recall"，harness 主循环在 0.5 步拦截该调用，解析 from/to 后调 audit.read_range，并把结果作为用户消息 push 回上下文（幂等元操作不走去重）
   - src/harness.rs:273-297（审计 #2）
   - src/harness.rs:23（审计 #2）
2. Audit::read_range 读回 seq ∈ [from,to] 的审计原文行，按 seq 升序排序，保证不重不漏；last_seq 供压缩时标记审计范围
   - src/audit.rs:114-133（审计 #6）
   - src/audit.rs:109-112（审计 #6）
3. context::recall_message 将读回行格式化为「[recall] 审计原文（按 seq 升序，不重不漏）」逐行 #seq 标注，空范围给出提示，包装成 ChatMessage::User
   - src/context.rs:84-94（审计 #4）
4. 压缩 handoff 指引含 recall 可用范围 seq 1..=N（P005 R4.1：起点必须是 1），是 recall 的上游触发来源
   - src/context.rs:124-129（审计 #4）
   - src/harness.rs:105（审计 #2）
5. recall 结果内容参与空转/零增量统计（info_keys 计入 seen_keys 时重置 no_progress 与 zero_gain_streak），且 recall 调用本身也写入审计（record "recall"）
   - src/harness.rs:284-293（审计 #2）
6. recall 与 submit_report 同为 harness 内置处理但对 LLM 可见可调的工具；测试 recall_reads_audit_range_without_loss 验证 read_range 区间语义与消息格式
   - src/harness.rs:422-446（审计 #2）
   - src/harness.rs:942-964（审计 #2）

## 死胡同
- src/vector/recall.rs 的 RecallEngine 与本功能无关，仅命名相似（docs/上下文与长任务管理/01-context-compression.md:22 亦明确提示）

## 置信度
high

## 统计
turns=4 · tool_calls=5 · duration=29990ms · tokens=23009
