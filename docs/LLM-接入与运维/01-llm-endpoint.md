# OpenAI 兼容端点接入（LLM 接入与运维）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-06

## 结论
OpenAI 兼容端点接入是 codesleuth 的 LLM 通道：通过 OpenAI Chat Completions 兼容协议（支持 newapi 等网关）向任意兼容端点发对话/工具调用请求，用户可用 --model/--base-url 覆盖 ~/.codesleuth/config.toml 的 [llm] 配置，从而灵活换供应商。入口在 CLI（src/cli.rs:35-40 定义旗标、src/cli.rs:231-236 构造 provider），实现是 src/llm.rs:81-105 的 OpenAiProvider。调用链：cli run → config::load（应用 CLI 覆盖）→ OpenAiProvider::new → Harness 持有 Arc<dyn LlmProvider> 在每轮对话中调用。下游被 harness 代理循环消费；嵌入端点（[vector]）缺省跟随 [llm] 的 base_url/key（可显式覆盖）。

## 证据列表
1. OpenAiProvider 是 OpenAI 兼容协议实现，注释明示支持 newapi 等网关，持有 base_url/api_key，直发通道可注入网关扩展字段（thinking）
   - src/llm.rs:81-90（审计 #2）
2. CLI 提供 --model / --base-url 覆盖旗标，构造 CliOverrides 传给 config::load 合入生效配置
   - src/cli.rs:35-40（审计 #11）
   - src/cli.rs:212-216（审计 #11）
3. LlmConfig 定义 base_url/model/api_key，默认 base_url 为 https://api.openai.com/v1、model 为 gpt-4o-mini
   - src/config.rs:99-114（审计 #4）
4. 入口用生效配置构造 OpenAiProvider 并以 Arc<dyn LlmProvider> 共享
   - src/cli.rs:231-236（审计 #11）
5. Harness 持有该 provider，作为代理循环中每轮 LLM 调用的执行者
   - src/harness.rs:35-56（审计 #16）
6. 嵌入供应商缺省跟随 [llm] 的 base_url/api_key，显式 [vector] 覆盖优先（有测试 embed_endpoint_fallback_follows_llm 佐证）
   - src/cli.rs:138-142（审计 #11）

## 死胡同
- grep 'base-url|base_url' plain 模式初查 0 命中（正则字符类问题），改用 regex 后命中

## 置信度
high

## 统计
turns=6 · tool_calls=9 · duration=18995ms · tokens=35560

