# 微观 · 上下文压缩（context-compaction）

> 置信度 high · 5 条发现 · 191578 tokens

上下文压缩链路（cli 装配→Harness::run 阈值判定→build_handoff→compact→审计留痕→recall 续读）整体实现与文档口径一致：seq 起号 1、handoff from=1、tool-call 对回退、审计原子取号均正确。但发现 3 个真实缺陷：①P1——recall 工具分支（harness.rs:337-361）完全豁免 no-progress 熔断（零增益也不递增），且主循环无轮数上限，模型循环 recall 可无限烧 LLM 调用；②P2——当超阈值体量集中在保留侧（单条超大 read/recall 消息），compact 恒为 no-op 但 should_compact 恒真，每轮发超真实窗口的请求直至 API 400 硬终止，无降级路径；③P3——chars/4 的 token 估算对 CJK 内容低估约 4 倍，真实窗口可在压缩触发前先爆。死代码维度链内干净（0 确定死/0 疑似死）。最优先修复：recall 分支补零增益计数。

1. P1：recall 分支豁免无进展熔断——零增益召回既不递增 no_progress 也不递增 zero_gain_streak，而常规工具路径零增益会 no_progress+=1；主循环无轮数上限，熔断是唯一终止保险，模型循环 recall(1,1) 可无限调用 LLM。修复：recall 零增益分支补 no_progress += 1。（src/harness.rs:337-361、src/harness.rs:443-452）
2. P2：单条超大消息（一次大 read 或无上限 recall）落在 KEEP_RECENT=6 保留侧时，compact 恒 no-op 而 should_compact 恒真，每轮把超真实窗口的请求发给 LLM，最终 API 400 → llm_error → run 硬终止，且审计无'压缩超限'线索；应对保留侧超限有检测/截断或专门错误码。（src/harness.rs:118-143、src/context.rs:57-82、src/harness.rs:155-166）
3. P3：estimate_tokens 用 chars/4，对 CJK（本仓库工具输出主体）真实 token 低估约 4 倍；60% 阈值的 40% 余量不足以兜底，真实窗口可在压缩触发前先爆。属已声明启发式，列债务。（src/context.rs:25-28、src/config.rs:129-132）
4. 核对无误项：审计 seq 锁内取号从 1 起号单调（record），handoff from=1 硬编码与之一致（P005 R4.1 修复在位并有回归测试）；tool-call 对边界回退对 A/T1/T2、keep_recent=1 等边界代入均不产生孤儿 tool 消息；compact_threshold_tokens 饱和乘+percent≤100 封顶正确；read_range 按 seq 过滤排序不重不漏。（src/audit.rs:144-150、src/harness.rs:125-126、src/context.rs:66-71）
5. 死代码维度：链内 compact/KEEP_RECENT/should_compact/build_handoff/estimate_tokens/recall_message/compact_threshold_tokens 全部有生产调用，确定死 0、疑似死 0；audit.rs 证据账本砍除为有效债务登记注释；append_line 与 record 取号逻辑双轨是轻微残余。（src/cli.rs:393-400、src/harness.rs:120-128、src/audit.rs:166-168）
