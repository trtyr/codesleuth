# submit_report 结构化报告（报告与证据校验）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
submit_report 是 harness 内置的收敛工具：LLM 调查结束时提交版本化结构化报告（answer/findings/dead_ends/confidence/stats），harness 逐条校验 evidence 引用的文件必须是本会话真实观察过的（EvidenceStore），未读文件则拒绝并列出；不通过时回传拒因允许修正重提，通过后渲染为人类可读报告返回。若模型不走结构化提交，则以 degraded_prose 降级构造并诚实标注 degraded=true。

## 证据列表
1. submit_report 是 harness 内置工具（SUBMIT_TOOL），schema 要求 answer/findings/confidence 必填、confidence 限 high|medium|low
   - src/harness.rs:421-443（审计 #4）
2. 报告结构为 Report{report_schema_version=1, answer, findings, dead_ends, confidence, degraded, stats}，并有 validate() 校验置信度与空 answer
   - src/report.rs:7, 32-43（审计 #2）
   - src/report.rs:59-74（审计 #2）
3. 调用链：主循环识别 SUBMIT_TOOL → build_report 逐条用 EvidenceStore.cite_seq 校验证据 file 是否会话内真实读过，未读文件列入拒绝理由；被拒后 reject_call 允许修正重提，成功则 audit.record('report') 并 render_human 返回 RunOutcome
   - src/harness.rs:223-270（审计 #4）
   - src/harness.rs:460-544（审计 #4）
   - src/evidence.rs:26-33（审计 #25）
4. 降级路径：模型无工具调用时先引导提交，仍失败则 Report::degraded_prose 构造 degraded=true 的诚实降级报告
   - src/report.rs:1-2, 46-57（审计 #2）
   - src/harness.rs:176-200（审计 #4）
5. 与其他模块交互：上游依赖 read/审计流水线（EvidenceStore 记录路径→审计 seq，Evidence.audit_seq 作报告↔审计互查锚点）；旧证据账本 Ledger 已裁决砍除，findings 由报告本体承载
   - src/evidence.rs:26-30（审计 #25）
   - src/report.rs:13-15（审计 #2）
   - src/audit.rs:166-168（审计 #30）

## 死胡同
无

## 置信度
high

## 统计
turns=11 · tool_calls=10 · duration=38134ms · tokens=74226

