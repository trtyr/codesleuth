# 零增量转向（上下文与长任务管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
零增量转向是 harness 主循环中的防打转软机制：连续 2 回合工具执行无新信息增量时，向对话注入一条「无新信息」的用户消息，促使模型换工具/换角度或直接收敛，不终止运行。

## 证据列表
1. 功能价值：当模型连续执行但产出无新信息时注入转向提示，促其换策略或收敛，区别于 5 步无进展熔断（不终止运行）
   - src/harness.rs:393-398（审计 #2）
2. 入口：阈值常量 ZERO_GAIN_STEER_THRESHOLD=2 定义于 src/harness.rs:20-21；计数器 zero_gain_streak 在主循环中初始化（src/harness.rs:92）
   - src/harness.rs:20-21（审计 #2）
   - src/harness.rs:92（审计 #2）
3. 运作：主循环工具执行后用 info_keys 对 seen_keys 去重判定增量，无增量则 streak+1（harness.rs:379-388），达到阈值即注入 User 转向消息并归零计数（harness.rs:394-402）
   - src/harness.rs:379-388（审计 #2）
   - src/harness.rs:394-402（审计 #2）
4. 交互：注入消息与 system prompt 的「无空转」条款（src/prompt.rs:19-20）呼应；零增量步同时计入 no_progress 熔断计数并经 fuse_if_hit 检查（harness.rs:399-401）；测试 zero_gain_streak_injects_steering 固化行为（harness.rs:781-804）
   - src/prompt.rs:19-20（审计 #18）
   - src/harness.rs:399-401（审计 #2）
   - src/harness.rs:781-804（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=7 · tool_calls=6 · duration=27597ms · tokens=38645
