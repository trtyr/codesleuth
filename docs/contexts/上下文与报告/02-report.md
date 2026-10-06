# 报告生成（上下文与报告）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
侦察完成。「报告生成」功能由 src/report.rs 定义版本化 Report schema（REPORT_SCHEMA_VERSION=1，src/report.rs:7），提供 build_report 装配（src/harness.rs:460）、degraded_prose 降级（src/report.rs:46）、render_human 人类渲染（src/report.rs:77）三路径；主循环在 submit_report 工具调用时走 build_report，否则兜底 degraded，最终通过 RunOutcome 返回。

## 证据列表
1. Report 是版本化 schema，含 report_schema_version / task / answer / findings / dead_ends / confidence / degraded / stats 字段
   - src/report.rs:32-43（审计 #2）
   - src/report.rs:7（审计 #2）
2. REPORT_SCHEMA_VERSION 常量固定为 1，由 build_report 与 sample 测试共同引用，验证 schema 序列化往返一致
   - src/report.rs:7（审计 #2）
   - src/harness.rs:582（审计 #4）
   - src/report.rs:158-165（审计 #2）
3. degraded_prose 在模型未提交 submit_report 时构造 degraded=true / confidence=low 的兜底 Report
   - src/report.rs:46-57（审计 #2）
   - src/harness.rs:200-211（审计 #4）
   - src/report.rs:173-179（审计 #2）
4. render_human 用确定性模板生成"任务/结论/证据/死胡同/置信度/统计"段落，degraded 时显式标注"（降级：模型未走结构化提交）"
   - src/report.rs:77-125（审计 #2）
   - src/report.rs:167-171（审计 #2）
5. ReportStats 记录 turns / tool_calls / duration_ms / total_tokens 四个统计字段
   - src/report.rs:24-30（审计 #2）
   - src/harness.rs:589-594（审计 #4）
6. build_report 校验 evidence 必须本会话读过、零 findings 但读过文件需 dead_ends 交代、confidence 必须在 high/medium/low 内
   - src/harness.rs:460-598（审计 #4）
   - src/report.rs:59-74（审计 #2）
   - src/report.rs:181-188（审计 #2）
7. 主循环 Harness::run 在收到 submit_report 工具调用时调用 build_report，成功后 audit record("report",…) 并以 RunOutcome 收尾；模型始终不提交则走 degraded_prose 兜底
   - src/harness.rs:222-260（审计 #4）
   - src/harness.rs:193-211（审计 #4）
8. RunOutcome 同时携带结构化 Report 与 render_human 的 answer 字符串，answer 字段就是给调用方看的渲染文本
   - src/harness.rs:25-33（审计 #4）
   - src/harness.rs:206（审计 #4）
   - src/harness.rs:252（审计 #4）

## 死胡同
- grep "use crate::report" 仅命中 src/harness.rs:10，未追查 tests/ 下是否另有消费方
- grep "ReportStats|report_schema_version" 命中 0 处，依赖面仅靠 src/harness.rs:10 显式 use 判断（推断）

## 置信度
high

## 统计
turns=9 · tool_calls=12 · duration=29480ms · tokens=78442

