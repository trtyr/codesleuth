# 报告↔审计双向回溯（审计流水）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
报告↔审计双向回溯：每次 read/工具调用经 Audit.record 写入审计 JSONL 时返回全局单调 seq，EvidenceStore 记录「路径→首次观察 seq」，submit_report 组装 Evidence 时回填 audit_seq，使每条报告证据都能用 seq 在审计文件中定位到那次工具调用原文；反向也可由审计行回查报告。价值：报告证据可审计、可验证、不凭空捏造。

## 证据列表
1. Evidence 结构含 audit_seq 字段，注释明确为「报告↔审计互查锚点」
   - src/report.rs:9-16（审计 #4）
2. EvidenceStore 以路径→首次观察 seq 记账（observe_exact/cite_seq），submit_report 组装时回填 Evidence.audit_seq
   - src/evidence.rs:26-37（审计 #20）
   - src/harness.rs:504-514（审计 #12）
3. Audit.record 写审计行返回单调 seq；read_range 按 seq 范围读回原文不重不漏
   - src/audit.rs:114-163（审计 #2）
4. 测试验证 evidence.audit_seq 在审计文件中命中 kind=tool_call 行，构成正向回溯
   - src/harness.rs:903-913（审计 #12）
5. 评估硬门 G2 强制每条 evidence 带 audit_seq 且能回溯到工具调用行，缺失或无法回溯即判失败
   - scripts/eval/hardgate.py:43-52（审计 #25）
6. handoff/recall 机制复用同一审计 seq 体系（seq 从 1 起号、按范围 recall），是 seq 的另一消费方
   - src/context.rs:125-128（审计 #27）

## 死胡同
无

## 置信度
high

## 统计
turns=8 · tool_calls=10 · duration=26271ms · tokens=69603

