# 空转熔断（上下文与长任务管理）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
空转熔断是 Agent 主循环的安全阀：连续 5 步无进展（重复调用、非法调用、零增量）即以会话级错误 LLM_FUSE（CS2099）终止 run()，防止模型无限空转烧钱。核心全在 src/harness.rs：阈值常量、fuse_if_hit 判定函数、run() 主循环多处检查点；新信息增量将 no_progress 归零。

## 证据列表
1. 阈值常量 MAX_NO_PROGRESS_STREAK=5；fuse_if_hit 在 streak≥5 时记录审计 "fuse" 并返回 LLM_FUSE 错误（CS2099，故障域非成本限制）
   - src/harness.rs:18-19（审计 #2）
   - src/harness.rs:616-625（审计 #2）
2. run() 主循环维护 no_progress 计数（第91行），在重复调用、参数非法 JSON、LLM 失败、工具错误等多处检查点 no_progress+=1 并调用 fuse_if_hit，命中即 return Err(fuse)
   - src/harness.rs:91（审计 #2）
   - src/harness.rs:304-307（审计 #2）
   - src/harness.rs:319-322（审计 #2）
   - src/harness.rs:226-227,262-263（审计 #2）
3. recall 结果产生新信息增量（info_keys 命中 seen_keys）时 no_progress 与 zero_gain_streak 归零
   - src/harness.rs:288-294（审计 #2）
4. 测试覆盖：5 步无进展触发 LLM_FUSE（exit_code=3）、工具错误调用也计入熔断
   - src/harness.rs:806-816（审计 #2）
   - src/harness.rs:818-824（审计 #2）

## 死胡同
无

## 置信度
high

## 统计
turns=5 · tool_calls=7 · duration=12075ms · tokens=23991
