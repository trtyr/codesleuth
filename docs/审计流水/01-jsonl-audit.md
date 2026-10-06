# 全量 JSONL 审计（审计流水）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
全量 JSONL 审计：按 seq 单调递增把会话事件追加写入 {session_id}.jsonl，作为全量原文恢复源；harness/writeguard/cli 各环节调用 Audit::record 写入，压缩 handoff 用 last_seq 记录范围，recall 经 read_range 按区间读回。

## 证据列表
1. 审计 JSONL 是全量原文恢复源，支持按 seq 区间读回（read_range 升序不重不漏）
   - src/audit.rs:1-3（审计 #2）
   - src/audit.rs:114-133（审计 #2）
2. Audit::create 建每会话文件并做 64MB 单代轮转；record 在锁内取 seq+写盘保证单调
   - src/audit.rs:63-103（审计 #2）
   - src/audit.rs:135-163（审计 #2）
3. harness::run 压缩前用 last_seq 定区间并写 compaction 事件；writeguard::write_check 用 append_line 追加宿主级事件
   - src/harness.rs:99-124（审计 #16）
   - src/audit.rs:20-51（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=6 · tool_calls=6 · duration=25619ms · tokens=33571

