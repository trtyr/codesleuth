# 微观 · LLM provider抽象（llm-provider）

> 置信度 high · 5 条发现 · 337213 tokens

LLM provider 链路（LlmProvider trait → OpenAiProvider::chat → Harness 主循环消费 ChatResponse）整体实现扎实：重试/退避、网关容错解码（D015 剥 service_tier）、错误码分段与 retryable=false 契约基本自洽，safe_prefix 的 UTF-8 边界处理有专门测试。但读穿后确认 3 个真实正确性缺陷：①HTTP 200 但响应体读取失败被 unwrap_or_default() 吞成空串，转成不可重试的 CS2004，语义应为可重试 CS2001——传输类中途错误被错误分类且失去重试机会（P2）；②5xx 服务器错误被标为 CS2002 LLM_RATE_LIMITED 而非 CS2003 LLM_SERVER，与错误码表常量定义、与测试侧 classify_llm_error 辅助函数三方不一致（P3）；③退避注释「500ms/1s/2s/4s 封顶」与实际 `500 << attempt.min(4)`（1s/2s/4s/8s）不符（P3 文档漂移）。死代码维：classify_llm_error 为 #[cfg(test)] 专用的生产逻辑复制品且已与生产实现分叉（正是缺陷②的温床），是最值得清理的一项。bug 密度最高文件：src/llm.rs。

1. P2｜200 状态下响应体读取失败被静默吞掉并错分类：`resp.text().await.unwrap_or_default()` 把传输错误（连接在响应头后中断）变成空串，随后 decode_response 对空串报 CS2004 LLM_BAD_RESPONSE 且 286 行 with_retryable(false)，而该场景本质是网络瞬断，应为可重试 CS2001。触发条件：网关 200 后连接 reset。错误行为：终局失败 exit 3 且不重试；应有：走重试退避。上游防线不成立：外层无再分类，Harness 收到即 return Err（harness.rs:155-166），CLI 直接 e.exit_code()。（src/llm.rs:282-287）
2. P3｜5xx 错误码分类三方不一致：生产路径把 429 与 5xx 统一标 CS2002 LLM_RATE_LIMITED（llm.rs:262-264），而错误码体系专门定义了 CS2003 LLM_SERVER、测试侧 classify_llm_error 也把 s>=500 映射为 LLM_SERVER（llm.rs:320）。5xx 是服务器故障非限流，语义误导排查。两者 exit code 同为 3，仅人读语义受损，故 P3。（src/llm.rs:262-267、src/errors.rs:36-42、src/llm.rs:315-332）
3. P3｜退避序列注释与实现不符：注释称「500ms / 1s / 2s / 4s（封顶）」，实际 `500u64 << attempt.min(4)` 在 attempt=1..4 产生 1000/2000/4000/8000ms——首重试即 1s（无 500ms 档），封顶 8s 非 4s。纯文档漂移。（src/llm.rs:246-249）
4. 链路其余部分核对干净及审法：①重试终局各 return 点均 with_retryable(false)，与 Harness 只记审计不重试（harness.rs:155-166）的期望一致；②decode_response 容错剥 service_tier 有对照组测试（llm.rs:393-411）；③safe_prefix UTF-8 回退有边界测试；④to_request_message 四角色映射有 round-trip 测试；⑤config 链 thinking_on→thinking_disabled 取反往返一致（config.rs:286-288 与 517-519）；⑥resolve_api_key 判空规则与 provider 构造参数匹配；⑦CLI 硬编码 max_retries=2 与 new 语义一致。（src/llm.rs:234-306、src/llm.rs:110-118、src/llm.rs:335-344）
5. 维B·死代码：classify_llm_error 为 #[cfg(test)] 专用、生产零引用，是 chat() 内联分类逻辑的复制品且已分叉（5xx 一个 LLM_SERVER 一个 LLM_RATE_LIMITED）——复制品非但未成为单一事实源反而埋雷。最值得先清理：让 chat() 改调它（顺带修缺陷②）或删除。未发现成块注释代码/调试残留；cli.rs:274 硬编码 max_retries=2 不在配置面属可议设计非死代码。（src/llm.rs:312-332、src/llm.rs:421-437、src/cli.rs:271-276）
