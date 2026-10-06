# 证据库（安全与防护）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
证据库（EvidenceStore）登记会话内真实观察过的路径及其首次观察到的审计 seq，为 submit_report 校验与报告↔审计互查提供锚点；暴露 observe_exact（read 专用，逐字入库）与 observe（按空白/标点切词、提取路径形 token 入库）两类入口，cite_seq 提供锚点查询。

## 证据列表
1. EvidenceStore 结构体定义于 src/evidence.rs:26-30，内部为 path→seq 的 HashMap；模块注释明确为「会话内真实观察过的路径 → 首次观察的审计 seq（submit_report 校验 + 互查锚点）」。
   - src/evidence.rs:26-30（审计 #6）
2. observe_exact（read 工具专用，FINDING-010）位于 src/evidence.rs:33-37，逐字入库不做切词，专为含空格/中文的路径设计。
   - src/evidence.rs:33-37（审计 #6）
3. observed_paths（FINDING-012 零 findings 打回用）位于 src/evidence.rs:39-42。
   - src/evidence.rs:39-42（审计 #6）
4. observe（路径形 token 提取）位于 src/evidence.rs:44-51，按空白与 []()<>"'`,; 切词并经 path_like 过滤后入库。
   - src/evidence.rs:44-51（审计 #6）
5. cite_seq 锚点查询位于 src/evidence.rs:53-55，是 report ↔ audit 互查的桥梁。
   - src/evidence.rs:53-55（审计 #6）
6. 报告侧 Evidence 结构定义于 src/report.rs:9-16，其中 audit_seq 字段即为报告↔审计互查锚点，degraded 时为 None。
   - src/report.rs:9-16（审计 #8）
7. Harness 持有 evidence 字段（Mutex<EvidenceStore>）于 src/harness.rs:35-44，并在 new() 中初始化为 EvidenceStore::default()（src/harness.rs:59）。
   - src/harness.rs:35-44（审计 #10）
   - src/harness.rs:59（审计 #10）
8. evidence_store_observes_citations 测试位于 src/harness.rs:1122-1127，验证 observe → cite_seq 闭环。
   - src/harness.rs:1122-1127（审计 #10）
9. evidence_exact_paths_with_spaces_and_cjk 测试位于 src/harness.rs:1129-1148，是 FINDING-010 回归：含空格/中文路径必须以精确形式存取，碎片化观察不应破坏精确记录。
   - src/harness.rs:1129-1148（审计 #10）

## 死胡同
- 未深挖 Harness 主循环在何处实际调用 observe / observe_exact（仅确认字段持有）
- 未读 src/report.rs 30 行以后确认 Report/Finding 如何消费 audit_seq 字段

## 置信度
high

## 统计
turns=4 · tool_calls=4 · duration=27011ms · tokens=19625

