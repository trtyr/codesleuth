# Harness主循环（侦察编排）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
Harness::run（src/harness.rs:74-419）是 codesleuth Agent 的主循环：以单次任务字符串为输入，循环驱动 LLM 决策 → 只读工具调度 → 增量记账 → 通过 submit_report 收敛为带 evidence 校验的结构化 RunOutcome，或在连续无进展达 MAX_NO_PROGRESS_STREAK=5（src/harness.rs:19）时由 fuse_if_hit 返回 LLM_FUSE 熔断。证据校验在 build_report（src/harness.rs:460-598）里完成：findings 引用的 file 必须本会话真实观察到（cite_seq），否则连同零 findings 错误一并打回。

## 证据列表
1. Harness::run 是主循环入口，在 src/harness.rs:74-419 定义，构造 system+user 首消息并进入 loop。
   - src/harness.rs:74-99（审计 #2）
2. Harness 结构体持有 provider / tools / audit / evidence / model / context_tokens / compact_percent。
   - src/harness.rs:35-44（审计 #2）
3. 熔断阈值常量 MAX_NO_PROGRESS_STREAK = 5 在 src/harness.rs:19 定义。
   - src/harness.rs:18-19（审计 #2）
4. fuse_if_hit 在 streak >= MAX_NO_PROGRESS_STREAK 时返回 LLM_FUSE 错误并 audit.record("fuse")。
   - src/harness.rs:653-666（审计 #2）
5. 主循环每轮先按 context::should_compact 判断压缩，再 provider.chat 调 LLM。
   - src/harness.rs:100-148（审计 #2）
6. 工具调度按 submit_report / recall / 去重 / 工具存在性 / 工具执行分支处理；submit_report 走 build_report 校验 evidence 引用。
   - src/harness.rs:222-417（审计 #2）
7. 重复调用、非法 JSON、未知工具、零增量执行、工具错误均 no_progress += 1 并触发 fuse_if_hit。
   - src/harness.rs:300-416（审计 #2）
8. build_report 校验 evidence 引用的 file 必须 evidence.cite_seq 能查到（cite_seq 返回 None 计入 uncited 并返回 Err）。
   - src/harness.rs:460-598（审计 #2）
9. run_with 测试夹具用 Scripted provider + Harness::new 调用 harness.run，验证主循环语义。
   - src/harness.rs:803-821（审计 #2）
10. CLI 入口 src/cli.rs:371 用 rt.block_on(agent.run(&task)) 触发主循环并消费 RunOutcome。
   - src/cli.rs:359-371（审计 #36）
11. tests/adversarial.rs:197 也调用 agent.run(...) 消费 RunOutcome。
   - tests/adversarial.rs:189-197（审计 #31）
12. 单测 fuse_after_five_no_progress_steps 与 tool_errors_count_toward_no_progress_fuse 验证 5 步熔断与工具错误计入熔断。
   - src/harness.rs:871-895（审计 #2）

## 死胡同
- grep 检索 "\.run\(" / "\brun\(" / "Harness::|Harness\s*\{" 在 Rust 项目里匹配过宽或被分词切碎，未能直接拿到除 cli.rs 与 src/harness.rs:803 之外的外部调用点；callers 图确认 Harness::run 只有 run_with 与 compaction_fires_when_window_exceeds_and_evicts 两个内部调用者，外部真实消费来自 src/cli.rs:371 与 tests/adversarial.rs:197。

## 置信度
high

## 统计
turns=11 · tool_calls=15 · duration=40394ms · tokens=139589

