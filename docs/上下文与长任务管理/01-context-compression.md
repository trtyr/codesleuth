# 上下文三段式压缩（上下文与长任务管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
上下文三段式压缩：当会话消息的估算 token 达到「模型窗口 × 百分比」阈值时，Agent 循环自动执行「持久化（审计账本落盘）→ 生成确定性 handoff 衔接通知 → 压缩窗口」，全程零 LLM 调用，使长任务不因上下文装满而中断，被驱逐的原文可经 recall 从审计原文不重不漏地续读。入口在 src/context.rs，由 src/harness.rs 主循环触发。

## 证据列表
1. 功能定义：窗口阈值触发三段式压缩（持久化→handoff→压缩），handoff 内含 recall 指引，零 LLM
   - src/context.rs:1-4（审计 #2）
   - src/context.rs:44-53（审计 #2）
2. 入口与核心函数：compact_threshold_tokens/should_compact 触发判断；compact 为确定性纯函数，保留头部 system+task 与最近 KEEP_RECENT=6 条，中间替换为 handoff
   - src/context.rs:30-37（审计 #2）
   - src/context.rs:56-82（审计 #2）
3. 调用链：harness 主循环每轮算阈值并 should_compact 判断，触发后依次调用 build_handoff 与 compact，再通过 self.audit.record 写 compaction_begin/compaction 审计留痕
   - src/harness.rs:100-124（审计 #10）
4. 交互：上游依赖审计账本（last_seq/record）与 ContextConfig（compact_at_percent 默认 60）；压缩带 tool 调用对感知（不拆散 assistant/tool pair），被驱逐原文供 recall 钻取
   - src/config.rs:92-97,115-118（审计 #15）
   - src/context.rs:63-71（审计 #2）

## 死胡同
- grep 误召回 src/vector/recall.rs 的向量索引 compact，与本功能无关

## 置信度
high

## 统计
turns=6 · tool_calls=5 · duration=25309ms · tokens=43192

