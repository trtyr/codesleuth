# 宏观 · errors

> 置信度 high · 9 条发现 · 154909 tokens

总评：健全偏上（有统一体系+主链路全通+可观测，但存在成体系的"静默降级"漏网带）。本项目拥有成熟度较高的错误处理系统：集中式错误码表（errors.rs CS1xxx~CS5xxx 段位→exit code 映射+单测锁定）、单一 CsResult 贯穿全仓、错误携带结构化信息（码/消息/hint/retryable/source_text 根因链）、LLM 层带退避重试、CLI 装配层系统性降级+审计留痕（degraded 行）、失败路径统一 report_error 收口。主要弱点不在类型体系而在"防线自身的失败被吞"：writeguard 考前快照 .ok() 静默、审计 record 留痕 let _ = 吞错、日志轮转 rename let _ = 吞错、LLM 200 响应体读失败 unwrap_or_default() 错分类为不可重试、5xx 误标 RATE_LIMITED 且 LLM_SERVER 生产零引用、CLI 失败路径短路跳过零写入自证——即错误管理系统自己的错误通道约 6-8 处不设防。分级：健全（有统一体系+主链路可观测），带"部分覆盖"的例外带（防护性/审计性通道）。

1. 【域1·最强】统一错误类型体系：CsCode 段位常量表（CS1xxx 用户/CS2xxx LLM/CS3xxx 目标库/CS4xxx 索引/CS5xxx 内部）+ CsError 一等结构（code/message/hint/retryable/source_text 根因链），exit_code 段位映射集中且有单测（exit_codes_follow_segment, errors.rs:126-133）。（src/errors.rs:8-53）
2. 【域1/域3/域4·最强】错误携带结构化上下文并可程序化溯源：with_source 保留内层错误原文、with_hint 给修复建议、report_error 统一 stderr 三行输出（主行+根因+hint）；main 入口统一 std::process::exit(code) 收口。（src/errors.rs:58-119、src/main.rs:5-12）
3. 【域6·最强】系统性恢复策略在位：LLM 调用带指数退避重试（attempts 循环+warn/error 分级日志+retryable 标志驱动 continue）；CLI 装配层 codegraph/向量层失败走降级（warn+审计 degraded 行）而非崩溃，仅 INDEX_LOCKED 超时判负不降级——降级/判负/重试三层分明。（src/llm.rs:243-305、src/cli.rs:299-311、src/cli.rs:355-366）
4. 【域2·最弱】LLM 错误分类三处失真：①200 状态下 resp.text().await.unwrap_or_default() 把传输中途失败吞成空串→decode_response 报不可重试 CS2004，失去重试机会（llm.rs:261/282）；②5xx 与 429 统一标 CS2002 LLM_RATE_LIMITED（llm.rs:262-267），专设的 LLM_SERVER(2003) 生产零引用，误报速率限制；③退避注释 500ms/1s/2s/4s 与实现 500<<attempt（1s/2s/4s/8s）不符（llm.rs:247-248）。（src/llm.rs:261-288）
5. 【域2/域4·最弱】防线自身的失败被静默吞掉：①考前 writeguard 快照 .ok() 无审计留痕，零写入自证可无声失效（cli.rs:287）；②agent.run 的 ? 短路使一切 LLM/熔断失败跳过考后 write_check 自证（cli.rs:419-424）；③五处降级留痕 let _ = audit_log.record(...) 连留痕失败也被吞（cli.rs:306/362/385）；④日志轮转 rename 失败 let _ = 吞且时 tracing 未初始化（logs.rs:28，grep 穷举确认）。（src/cli.rs:287-287、src/cli.rs:419-443）
6. 【域4·敏感泄漏】错误消息经 safe_prefix(&snippet,200) 截断入消息，请求体含 bearer_auth(api_key)（llm.rs:253）——api_key 本身不进错误消息（仅响应体片段），未见密钥直接泄漏路径；错误消息对 CJK 网关响应体的字节截断 panic 风险在 vector/embed.rs（底稿已证，属崩溃式而非泄漏）。（src/llm.rs:253-265）
7. 【域4·可观测】session_id 入口生成串线日志与审计（main.rs:8-9），失败路径 tracing::warn!(component=..., error=%e, ...) 结构化字段降级日志 + 审计 JSONL degraded 行；report_error 只写 stderr 保持 stdout 纪律。（src/main.rs:8-10、src/cli.rs:291-303）
8. 【域5·边界覆盖】网络（LLM 重试+终局收敛）、外部进程（MCP spawn 错误分类+timeout 包裹+bootlock flock）、文件 IO（config parse/写盘、审计 append 均有码位）、用户输入（clap+USER_INPUT 码+find_key 未知键 CS1012）均有守卫；并发边界（bootlock 三态/超时 CS4016）覆盖。缺口：响应体读取失败无守卫（见上）、MCP 读行无长度上限（底稿 mcp.rs:158）、embed UTF-8 切片 panic（embed.rs:171-174）。（src/llm.rs:290-299、src/cli.rs:287-291、src/llm.rs:257-288）
9. 【域1·反模式面】崩溃式展开 grep 穷举：生产代码（非 #[cfg(test)]）中 unwrap/expect 极少——13 文件命中里 harness.rs:922-925、config.rs:548-614、bootlock.rs:145+、logs.rs:79+、cli.rs:770+、graph.rs:281+ 全部位于测试模块；生产路径的核心缺陷是 unwrap_or_default 吞错（llm.rs:261/282）而非裸 panic，唯一的 panic 族风险集中在 vector/chunk.rs 越界切片与 embed.rs UTF-8 截断（底稿已核）。（src/errors.rs:9-23）
