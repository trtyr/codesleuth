# 结构化错误与退出码（LLM 接入与运维）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
结构化错误与退出码功能为 CLI 调用方提供可编程判别的错误体系：每个错误是 CsError（CS 编码 + 人话消息 + 修复建议 + 可重试标记 + 根因链），退出码由错误码段位决定（1000-1009→1 用法、1010-1099→2 配置/凭据、CS2xxx→3 上游 LLM、CS3xxx→4 目标库、CS4xxx→5 索引、其他→6 内部），供脚本按退出码分类处理。核心定义在 src/errors.rs（CsCode::exit_code、CsError、report_error），CLI 各命令入口统一 catch 后调用 report_error 打印并返回 e.exit_code() 作为进程退出码；上游 LLM 模块将 HTTP 状态分类映射到 CS 码（如 429→LLM_RATE_LIMITED、401→CONFIG_MISSING）。下游 config、fence、harness 等模块的测试分别断言配置错误 exit_code=2、围栏错误=4、LLM 熔断错误=3，验证分段契约。

## 证据列表
1. CsCode::exit_code 按码段映射退出码 1-6：1000-1009→1 用法、1010-1099→2 配置、2000-2999→3 上游 LLM、3000-3999→4 目标库、4000-4999→5 索引、其余→6 内部
   - src/errors.rs:13-22（审计 #2）
2. CsError 结构含 code、message、hint、retryable、source_text（根因链）；report_error 将其以主行 + 根因 + hint 行输出到 stderr
   - src/errors.rs:56-67（审计 #2）
   - src/errors.rs:106-115（审计 #2）
3. 错误码常量按段位定义：CS1xxx 用户/配置、CS2xxx 上游 LLM（含 LLM_FUSE 熔断 2099）、CS3xxx 目标库/围栏、CS4xxx 索引、CS5xxx 内部
   - src/errors.rs:32-53（审计 #2）
4. CLI 命令入口（如 run_index_vector 与 run_task）统一 catch CsError：report_error 打印后返回 e.exit_code() 作为进程退出码
   - src/cli.rs:110-122（审计 #13）
   - src/cli.rs:180-188（审计 #13）
   - src/cli.rs:7-7（审计 #13）
5. 上游 LLM 错误分类：classify_llm_error 按 HTTP 状态映射 CS 码——429→LLM_RATE_LIMITED（可重试）、401/403→CONFIG_MISSING、5xx→LLM_SERVER、其他→LLM_BAD_RESPONSE、无状态→LLM_UNREACHABLE
   - src/llm.rs:304-321（审计 #4）
6. 下游模块测试消费该契约：config.rs 断言配置错误 exit_code=2，fence.rs 断言围栏拒绝 exit_code=4，harness.rs 断言 LLM 熔断 exit_code=3
   - src/config.rs:606-610（审计 #18）
   - src/fence.rs:112-116（审计 #20）
   - src/harness.rs:812-816（审计 #22）

## 死胡同
无

## 置信度
high

## 统计
turns=7 · tool_calls=8 · duration=31946ms · tokens=49039

