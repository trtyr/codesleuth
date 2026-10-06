# System Prompt（侦察编排）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
System Prompt 是 codesleuth 只读侦察 Agent 的恒定身份与行为底座，定义于 `src/prompt.rs:4-30` 的 `SYSTEM_PROMPT` 常量，由 `Harness::run` 在每次会话启动时注入到 LLM 首条 system 消息（src/harness.rs:76-78），把「只读身份 / 检索纪律 / 无空转 / 输出契约」一次性钉死，保证后续轮次不因 user 任务诱导越界（D005）。

## 证据列表
1. SYSTEM_PROMPT 常量在 src/prompt.rs:4-30 定稿，内容为「只读身份 + 行为边界 + 检索纪律（层进协议）+ 无空转 + 输出契约」的中文 system prompt。
   - src/prompt.rs:4-30（审计 #2）
2. 锚点测试 prompt_contains_contract_anchors（src/prompt.rs:37-56）断言 SYSTEM_PROMPT 含「只读 / 拒绝 / explore / find_files / grep / callers / read / file:line / 无新信息 / submit_report / 置信度 / 死胡同」等关键锚点，防止定稿漂移。
   - src/prompt.rs:37-56（审计 #2）
3. Harness::run 在启动时构造 messages，首条 ChatMessage::System 的 content 引用 crate::prompt::SYSTEM_PROMPT（src/harness.rs:76-78），紧随 ChatMessage::User 装载具体 task（src/harness.rs:79-81），system 与 user 严格隔离。
   - src/harness.rs:74-82（审计 #6）
4. D005 决策（docs/plantree/plans/001-read-only-agent-harness/decisions/005-prompt-layering.md:6-11）拍板「system 定只读身份（产品资产，随发行版走），user 载具体任务，任务细节永不进 system；只读身份永不因 user prompt 改变」——把 SYSTEM_PROMPT 抬升为不可漂移的产品资产。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/005-prompt-layering.md:6-11（审计 #4）
5. D006 决策（docs/plantree/plans/001-read-only-agent-harness/decisions/006-all-tools-layered-protocol.md:15-26）把「层进式检索协议」明确写入 D005 的 system 层，使 SYSTEM_PROMPT 的「检索纪律」段获得设计依据。
   - docs/plantree/plans/001-read-only-agent-harness/decisions/006-all-tools-layered-protocol.md:15-26（审计 #19）
6. SYSTEM_PROMPT 的下游消费面：Harness::run 整轮 LLM 决策（src/harness.rs:136 `self.provider.chat(&req).await`）都建立在注入的 system 身份之上；测试路径 run_with（src/harness.rs:803-821）也复用 Harness::new→Harness::run 链，间接覆盖 SYSTEM_PROMPT 注入。
   - src/harness.rs:136（审计 #6）
   - src/harness.rs:803-821（审计 #6）

## 死胡同
- src/vector/repomap.rs（相似度 0.52，grep 后与 SYSTEM_PROMPT 无引用关系，弃用）
- src/vector/chunk.rs（相似度 0.48，grep 后与 SYSTEM_PROMPT 无引用关系，弃用）

## 置信度
high

## 统计
turns=6 · tool_calls=8 · duration=19739ms · tokens=38034

