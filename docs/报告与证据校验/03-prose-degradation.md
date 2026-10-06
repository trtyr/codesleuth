# prose 降级路径（报告与证据校验）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
prose 降级路径是报告与证据校验体系的兜底机制：当模型连续两轮不调用 submit_report 结构化提交、只回纯文本时，harness 将其文本包装为 degraded=true、confidence=low 的降级 Report 返回，保证调用方始终拿到带统计的报告，且能明确区分"真收敛"与"散文兜底"（eval 硬门 G4 将 degraded 视为不收敛）。

## 证据列表
1. 功能：模型未走 submit_report 时，用其纯文本 prose 构造诚实标注 degraded=true 的降级报告，避免结构化提交失败导致无输出。
   - src/harness.rs:176-204（审计 #6）
   - src/report.rs:1-2（审计 #2）
2. 入口与关键文件：构造入口 Report::degraded_prose（src/report.rs:46-57），触发点在 harness 主循环（src/harness.rs:193-206），人类渲染附加降级说明（src/report.rs:108-115），测试覆盖 degraded_report_marks_itself 与 prose_falls_back_to_degraded_report（src/report.rs:173-179；src/harness.rs:870-881）。
   - src/report.rs:46-57（审计 #2）
   - src/harness.rs:193-206（审计 #6）
   - src/report.rs:108-115（审计 #2）
   - src/report.rs:173-179（审计 #2）
3. 运作：run 主循环检测 tool_calls 为空时计数 prose_streak——首轮将 prose 入史并重新引导 submit_report，连续第二轮则调用 Report::degraded_prose 构造降级报告并向审计记录 answer(degraded=true) 后返回 RunOutcome。
   - src/harness.rs:177-203（审计 #6）
   - src/harness.rs:870-881（审计 #6）
4. 交互：上游依赖 LLM 轮次与审计模块（self.audit.record 记录 prose_steer/answer 事件，src/harness.rs:189-203）；下游被 eval 硬门消费——G4 将 degraded=true 判为不收敛（scripts/eval/hardgate.py:62-64）。
   - src/harness.rs:189-203（审计 #6）
   - scripts/eval/hardgate.py:62-64（审计 #22）

## 死胡同
无

## 置信度
high

## 统计
turns=7 · tool_calls=9 · duration=27759ms · tokens=36725

