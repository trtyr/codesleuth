# LLM provider抽象（LLM与配置）

> 功能初稿 · codesleuth 概览侦察 · 2026-10-07

## 结论
LLM provider 抽象通过 `LlmProvider` trait（src/llm.rs:18-22）暴露统一的 `chat` 接口；当前唯一实现 `OpenAiProvider`（src/llm.rs:81-90, 233-306）走 OpenAI 兼容协议 + 自定义 base_url，重试/退避与网关响应容错均封装在本层内部，最终失败以 `retryable=false` 返给宿主。CLI 在入口处用 `LlmConfig` 构造该 provider 并以 `SharedProvider` 注入 `Harness`，由 `Harness` 在每轮循环中通过 `self.provider.chat(&req)` 驱动调用（src/cli.rs:237-242；src/harness.rs:36, 136）。

## 证据列表
1. 模块文件头声明本层职责：OpenAI 兼容协议 + async-openai + 自定义 base_url，重试/退避在本层做，最终失败 retryable=false。
   - src/llm.rs:1-2（审计 #2）
2. `LlmProvider` trait 仅暴露 `async fn chat(&self, req: &ChatRequest) -> CsResult<ChatResponse>`。
   - src/llm.rs:18-22（审计 #2）
3. `OpenAiProvider` 持有 `max_retries` / `http: reqwest::Client` / `base_url` / `api_key` / `thinking_disabled`，由 `new(base_url, api_key, max_retries, thinking_disabled)` 构造。
   - src/llm.rs:81-105（审计 #2）
4. `chat` 实现包含：构造请求、注入 `thinking: {type:"disabled"}`、指数退避循环（500ms << attempt，封顶 4）、对 429/5xx 标记 `retryable=true` 后重试，耗尽后以 `with_retryable(false)` 终局返回；`SharedProvider = Arc<dyn LlmProvider>` 作为共用别名。
   - src/llm.rs:234-310（审计 #2）
5. CLI 入口处用 `cfg.llm.base_url` / `api_key` / `cfg.thinking_disabled` 实例化 `OpenAiProvider` 并打包为 `SharedProvider`，`max_retries` 写死 2。
   - src/cli.rs:237-242（审计 #26）
6. `Config` 暴露的 LLM 连接面：`LlmConfig { base_url, model, api_key }`；`thinking_disabled` 在 `Config` 顶层，源自 `[behavior].thinking_on`。
   - src/config.rs:41-45（审计 #4）
   - src/config.rs:75-84（审计 #4）
   - src/config.rs:113-119（审计 #4）
   - src/config.rs:141（审计 #4）
7. `Harness` 持有 `provider: Arc<dyn LlmProvider>`，由 `Harness::new` 注入；循环中通过 `self.provider.chat(&req).await` 驱动单轮对话。
   - src/harness.rs:35-65（审计 #9）
   - src/harness.rs:136（审计 #9）
8. 测试桩 `Scripted` 实现同一 `LlmProvider` trait（tests/adversarial.rs:73-77；src/harness.rs:729-743, 1076-1081），用于在 harness/adversarial 测试中替代真实 provider；`OpenAiProvider` 自身也有针对 `decode_response` 非标 `service_tier` 字段的容错测试（src/llm.rs:392-418）。
   - src/harness.rs:729-743（审计 #9）
   - src/harness.rs:1076-1081（审计 #9）
   - src/llm.rs:392-418（审计 #2）
   - tests/adversarial.rs:73-77（审计 #53）
   - tests/adversarial.rs:186-190（审计 #53）
9. 错误体系：`CsError` 含 `code`（段位决定 exit code）/ `message` / `retryable` 等字段；LLM 相关段位为 CS2xxx（2001/2002/2003/2004/2099），由 provider 终局返回。
   - src/errors.rs:12-23（审计 #11）
   - src/errors.rs:35-40（审计 #11）
   - src/errors.rs:58-104（审计 #11）

## 死胡同
- grep `provider\.chat` / `self\.provider\.chat` / `provider\.` 多轮未命中（多次返回 0/82 命中）——后改用更宽的 `provider` 关键词才定位到 src/harness.rs:136 的真实调用点。
- vector_search 起步线索中的 src/cli.rs:524-536 `resolve_embed_endpoint` 与本功能（LLM provider 抽象）不直接相关，未深读。

## 置信度
high

## 统计
turns=15 · tool_calls=20 · duration=60815ms · tokens=273893

