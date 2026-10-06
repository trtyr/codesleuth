# 审计日志（安全与防护）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
审计日志是 codesleuth 的事实账本：每会话一个 JSONL 文件按 `seq` 单调记录 LLM/工具调用/压缩/写面自证等宿主级事件，供报告 finding 引用原文回溯，并通过 `audit_seq` 区间让压缩后被驱逐的早期消息可被 recall 续读。

## 证据列表
1. 审计模块的宿主级事件续号入口 `append_line` 在 src/audit.rs:21-51，取现有最大 seq + 1 写入 JSONL，保持单调不重不漏。
   - src/audit.rs:20-51（审计 #2）
2. 会话 ID 生成 `new_session_id` 用纳秒时间 + pid 的 hex 拼接。
   - src/audit.rs:53-60（审计 #2）
3. `Audit` 结构 + `create`/`record`/`read_range` 实现按 seq 单调的 JSONL 账本，单代轮转 64MB 上限。
   - src/audit.rs:62-163（审计 #2）
4. 进程入口在 main 生成 session_id 并注入日志与审计，保证日志串线与审计同一身份。
   - src/main.rs:5-12（审计 #25）
   - src/lib.rs:24-66（审计 #8）
5. CLI 用同一 session_id 创建审计日志，任务结束后调 `audit::append_line` 写 `write_check` 宿主级行（含失败留痕）。
   - src/cli.rs:231-235（审计 #22）
   - src/cli.rs:375-427（审计 #22）
6. Harness 在压缩/失败时通过 `self.audit.record` 留痕 `compaction_begin`/`compaction`/`llm_error`，并取 `last_seq` 喂给 handoff。
   - src/harness.rs:100-148（审计 #35）
7. 下游消费：`context::build_handoff` 引用 `audit_from..audit_to` 续读被驱逐原文；评估脚本从 stderr 解析审计路径并加载审计行做硬关；CLI stderr 摘要指向审计 `write_check` 行。
   - src/context.rs:47-54（审计 #42）
   - scripts/eval/run_eval.py:30-77（审计 #44）
   - src/cli.rs:411-411（审计 #22）
8. 单元测试断言 `append_line` 在已有 seq 之后续号为 max+1（写面自证 3 条 → seq 1,2,3 单调）。
   - src/writeguard.rs:226-249（审计 #4）
   - src/audit.rs:174-193（审计 #2）

## 死胡同
- grep "audit_path|Audit::create|\.record\(" 0 命中（正则需转义）
- grep "audit" 扫到 53 文件后未续扫剩余命中（已足以取证）

## 置信度
high

## 统计
turns=10 · tool_calls=17 · duration=52941ms · tokens=150525

