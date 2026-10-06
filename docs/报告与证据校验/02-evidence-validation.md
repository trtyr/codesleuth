# 证据校验（报告与证据校验）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
证据校验功能保证侦察报告中的每条 evidence(file:lines) 都来自本会话真实读取过的文件：会话期间 EvidenceStore 记录所有观察到的路径及其审计 seq；模型 submit_report 时逐条用 cite_seq 匹配，未读过的引用直接打回并要求先 read 或删除；同时打回零 findings / 无证据 finding 的报告。通过的 evidence 会被回填 audit_seq，作为报告与审计日志的互查锚点。

## 证据列表
1. EvidenceStore 是会话内证据库：记录真实观察过的路径 → 首次观察的审计 seq，供 submit_report 校验与互查；read 工具走 observe_exact 精确入库
   - src/evidence.rs:26-37（审计 #4）
2. 工具输出处理时，harness 对每次工具调用结果调用 evidence.observe(output, seq)，read 调用额外 observe_exact 记录精确路径
   - src/harness.rs:370-378（审计 #6）
3. submit_report 时逐条 evidence 用 cite_seq(&file) 匹配；命中则回填 audit_seq，未命中进入 uncited 拒收列表
   - src/harness.rs:504-515（审计 #6）
4. 存在未读文件引用时返回错误「evidence 引用了本会话未读过的文件」，即拒收机制本体
   - src/harness.rs:541-545（审计 #6）
5. 附加打回：零 findings 但会话读过文件且无 dead_ends 时打回提炼；findings 无 evidence 时也打回（FINDING-012）
   - src/harness.rs:547-577（审计 #6）
6. 测试验证互查能力：evidence.audit_seq 可在审计日志中定位到对应的 tool_call 记录
   - src/harness.rs:903-913（审计 #6）
7. 数据结构：Report/Finding/Evidence 定义于 report.rs，Evidence 含 file/lines/audit_seq（审计互查锚点）
   - src/report.rs:9-16（审计 #2）
   - src/report.rs:18-22（审计 #2）

## 死胡同
- grep plain 模式检索 compound pattern 无命中，改用 regex 后命中

## 置信度
high

## 统计
turns=7 · tool_calls=8 · duration=33569ms · tokens=41520

