//! LLM provider 抽象（Q001/D009）：OpenAI 兼容协议，async-openai + 自定义 base_url。
//! 重试/退避在本层做；最终失败后 retryable=false（宿主不再重试）。
//! P002 加固：解析崩自修（D029，EN-15 宽松归一先行 + 喂回补轮兜底 ≤1 轮）+
//! 失败原因码（D032，EN-17 parse_fail/truncated 细分）+ finish_reason 透传。

use crate::errors::{
    CONFIG_MISSING, CsCode, CsError, CsResult, FailureReason, LLM_BAD_RESPONSE, LLM_RATE_LIMITED,
    LLM_SERVER, LLM_UNREACHABLE,
};
use async_openai::types::chat::{
    ChatCompletionMessageToolCall, ChatCompletionMessageToolCalls,
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestToolMessage,
    ChatCompletionRequestUserMessage, ChatCompletionTool, ChatCompletionTools,
    CreateChatCompletionRequest, CreateChatCompletionRequestArgs, CreateChatCompletionResponse,
    FunctionCall, FunctionObject,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 单轮对话（含重试退避）。
    async fn chat(&self, req: &ChatRequest) -> CsResult<ChatResponse>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum ChatMessage {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tool_calls: Vec<ToolCallSpec>,
    },
    Tool {
        call_id: String,
        content: String,
    },
}

/// 一次工具调用（arguments 为模型产出的原始 JSON 字符串，解析在宿主侧）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallSpec {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ToolSchema>,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub text: Option<String>,
    pub tool_calls: Vec<ToolCallSpec>,
    pub usage: Usage,
    /// EN-17/D032：finish_reason 透传（stop/tool_calls/length/...）——审计与编排可读
    pub finish_reason: Option<String>,
}

/// OpenAI 兼容实现（newapi 等网关同此协议）。
pub struct OpenAiProvider {
    max_retries: u32,
    /// 直发通道：用于注入网关扩展字段（thinking 等），响应仍用 async-openai 类型反序列化。
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    /// 检索型任务默认关闭思考（用户拍板 2026-10-04）；配置 [behavior] thinking_on = true 可开回。
    thinking_disabled: bool,
}

impl OpenAiProvider {
    pub fn new(base_url: &str, api_key: &str, max_retries: u32, thinking_disabled: bool) -> Self {
        // thinking 开关走配置链（config.rs thinking_disabled）；检索型任务默认关
        Self {
            max_retries,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            thinking_disabled,
        }
    }

    /// 网关响应容错解码（D015）：先落 serde_json::Value 剥掉非标枚举字段再走 typed 解码。
    /// 第三方网关（如 MiniMax-M3 经 newapi）会回 `service_tier: "standard"` 等 OpenAI
    /// 官方枚举之外的值，async-openai 严格枚举零容忍，整条响应 CS2004 炸死。
    /// EN-15/D029：typed 仍失败时先过 normalize_response_value 宽松归一（确定性、零 LLM）
    /// 再试一次——单值↔数组、未知 finish_reason、usage 类型漂移等已知炸点就地修复。
    fn decode_response(text: &str) -> CsResult<CreateChatCompletionResponse> {
        let mut v: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| CsError::new(LLM_BAD_RESPONSE, format!("LLM 响应非 JSON: {e}")))?;
        if let Some(obj) = v.as_object_mut() {
            obj.remove("service_tier"); // 非标枚举值头号来源（D015）
        }
        match serde_json::from_value(v.clone()) {
            Ok(resp) => Ok(resp),
            Err(e) => {
                Self::normalize_response_value(&mut v);
                serde_json::from_value(v).map_err(|e2| {
                    CsError::new(
                        LLM_BAD_RESPONSE,
                        format!("LLM 响应解析失败: {e2}（原始: {e}）"),
                    )
                })
            }
        }
    }

    /// 宽松归一化（EN-15/D029 第一层）：typed 解码失败后的确定性修复，零 LLM 调用。
    /// 只处理「形状对但类型漂移」的已知炸点；语义级损坏交给自修轮（build_repair_request）。
    fn normalize_response_value(v: &mut serde_json::Value) {
        let Some(obj) = v.as_object_mut() else { return };
        if let Some(c) = obj.get_mut("choices") {
            // 炸点①：choices 单对象（invalid type: map, expected a sequence）
            if c.is_object() {
                *c = serde_json::Value::Array(vec![c.take()]);
            }
            let Some(arr) = c.as_array_mut() else { return };
            for choice in arr.iter_mut() {
                let Some(cm) = choice.as_object_mut() else {
                    continue;
                };
                // 炸点②：finish_reason 未知/null → "stop"（保底；length 真截断信息归 EN-17 透传）
                if let Some(fr) = cm.get_mut("finish_reason") {
                    let known = matches!(fr,
                    serde_json::Value::String(s) if matches!(
                        s.as_str(), "stop" | "tool_calls" | "length" | "content_filter" | "function_call"
                    ));
                    if !known {
                        *fr = serde_json::Value::String("stop".into());
                    }
                }
                // 炸点③：message 缺失/null → 合成空 assistant（invalid type: null）
                let bad_msg = cm.get("message").map(|m| !m.is_object()).unwrap_or(true);
                if bad_msg {
                    cm.insert(
                        "message".into(),
                        serde_json::json!({"role": "assistant", "content": ""}),
                    );
                }
                if let Some(m) = cm.get_mut("message").and_then(|m| m.as_object_mut()) {
                    if let Some(ct) = m.get_mut("content") {
                        // 炸点④：content multipart 数组 → 拼接 text 部分
                        if let Some(parts) = ct.as_array() {
                            let joined = parts
                                .iter()
                                .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
                                .collect::<Vec<_>>()
                                .join("");
                            *ct = serde_json::Value::String(joined);
                        } else if !ct.is_string() && !ct.is_null() {
                            // 炸点④b：content 其他非串类型（数字等）→ 字符串化，比炸掉好
                            let s = match &*ct {
                                serde_json::Value::String(s) => s.clone(),
                                other => serde_json::to_string(other).unwrap_or_default(),
                            };
                            *ct = serde_json::Value::String(s);
                        }
                    }
                    // 炸点⑤：tool_calls 单对象 → 包数组
                    if let Some(tc) = m.get_mut("tool_calls")
                        && tc.is_object()
                    {
                        *tc = serde_json::Value::Array(vec![tc.take()]);
                    }
                }
            }
        }
        // 炸点⑥：usage null → 删除（map_response unwrap_or_default 兜底）
        if obj.get("usage").map(|u| u.is_null()).unwrap_or(false) {
            obj.remove("usage");
        }
        // 炸点⑦：usage 数值字段字符串数字 → 转数字（invalid type: string, expected u32）
        if let Some(u) = obj.get_mut("usage").and_then(|u| u.as_object_mut()) {
            for k in ["prompt_tokens", "completion_tokens", "total_tokens"] {
                if let Some(serde_json::Value::String(s)) = u.get(k)
                    && let Ok(n) = s.parse::<u32>()
                {
                    u.insert(k.into(), serde_json::json!(n));
                }
            }
        }
    }

    /// 自修轮请求构造（EN-15/D029 第二层）：坏响应原文（截断）+ 解析错误喂回模型重造。
    /// 恰好追加 1 条 User 消息；tools 保留（自修轮可能需要继续产出 tool_calls）。
    fn build_repair_request(req: &ChatRequest, bad_text: &str, err: &CsError) -> ChatRequest {
        let mut messages = req.messages.clone();
        messages.push(ChatMessage::User {
            content: format!(
                "你上一次的响应无法被程序解析（错误：{err}）。上次响应原文（截断）：\n{}\n请重新输出完整、格式正确的响应：content 必须是纯文本字符串（或合法 tool_calls 数组），不要混入其他类型。",
                safe_prefix(bad_text, 2000)
            ),
        });
        ChatRequest {
            model: req.model.clone(),
            messages,
            tools: req.tools.clone(),
        }
    }

    fn build_request(req: &ChatRequest) -> CsResult<CreateChatCompletionRequest> {
        let messages: Vec<ChatCompletionRequestMessage> =
            req.messages.iter().map(to_request_message).collect();
        let mut args = CreateChatCompletionRequestArgs::default();
        args.model(&req.model);
        args.messages(messages);
        if !req.tools.is_empty() {
            args.tools(
                req.tools
                    .iter()
                    .map(|t| {
                        ChatCompletionTools::Function(ChatCompletionTool {
                            function: FunctionObject {
                                name: t.name.clone(),
                                description: Some(t.description.clone()),
                                parameters: Some(t.parameters.clone()),
                                strict: None,
                            },
                        })
                    })
                    .collect::<Vec<_>>(),
            );
        }
        args.build()
            .map_err(|e| CsError::new(LLM_BAD_RESPONSE, format!("请求构建失败: {e}")))
    }
}

#[allow(deprecated)] // AssistantMessage 的 function_call 字段已 deprecated，但结构体字面量必须显式填 None
fn to_request_message(m: &ChatMessage) -> ChatCompletionRequestMessage {
    match m {
        ChatMessage::System { content } => {
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: content.clone().into(),
                name: None,
            })
        }
        ChatMessage::User { content } => {
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: content.clone().into(),
                name: None,
            })
        }
        ChatMessage::Assistant {
            content,
            tool_calls,
        } => ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
            content: content.clone().map(Into::into),
            refusal: None,
            name: None,
            audio: None,
            tool_calls: (!tool_calls.is_empty()).then(|| {
                tool_calls
                    .iter()
                    .map(|tc| {
                        ChatCompletionMessageToolCalls::Function(ChatCompletionMessageToolCall {
                            id: tc.id.clone(),
                            function: FunctionCall {
                                name: tc.name.clone(),
                                arguments: tc.arguments.clone(),
                            },
                        })
                    })
                    .collect()
            }),
            function_call: None,
        }),
        ChatMessage::Tool { call_id, content } => {
            ChatCompletionRequestMessage::Tool(ChatCompletionRequestToolMessage {
                content: content.clone().into(),
                tool_call_id: call_id.clone(),
            })
        }
    }
}

fn map_response(resp: CreateChatCompletionResponse) -> CsResult<ChatResponse> {
    let mut choices = resp.choices;
    let choice = choices
        .drain(..)
        .next()
        .ok_or_else(|| CsError::new(LLM_BAD_RESPONSE, "响应无 choices"))?;
    let usage = resp.usage.unwrap_or_default();
    let tool_calls = choice
        .message
        .tool_calls
        .unwrap_or_default()
        .into_iter()
        .map(|tc| match tc {
            ChatCompletionMessageToolCalls::Function(f) => ToolCallSpec {
                id: f.id,
                name: f.function.name,
                arguments: f.function.arguments,
            },
            ChatCompletionMessageToolCalls::Custom(c) => ToolCallSpec {
                id: c.id,
                name: c.custom_tool.name,
                arguments: c.custom_tool.input,
            },
        })
        .collect();
    Ok(ChatResponse {
        text: choice.message.content,
        tool_calls,
        // EN-17/D032：finish_reason 透传（截断=length 场景编排层可读，不再猜）
        finish_reason: serde_json::to_value(choice.finish_reason)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string)),
        usage: Usage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        },
    })
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, req: &ChatRequest) -> CsResult<ChatResponse> {
        let request = Self::build_request(req)?;
        // 网关扩展字段注入（thinking）：检索型任务默认关闭思考
        let mut body = serde_json::to_value(&request)
            .map_err(|e| CsError::new(LLM_BAD_RESPONSE, format!("请求序列化失败: {e}")))?;
        if self.thinking_disabled {
            body["thinking"] = serde_json::json!({"type": "disabled"});
        }
        let url = format!("{}/chat/completions", self.base_url);
        let text = self.send_with_retry(&url, &body).await?;
        // EN-15/D029：decode（含宽松归一）与 map（choices 空等）失败都触发自修轮——
        // 坏响应 + 解析错误喂回模型重造，≤1 轮；chat() 内聚行为，不占外层 dispatch
        // 重试额度；自修轮单次发送不退避，usage 取自修轮（原轮解析失败无有效 usage）。
        match Self::decode_response(&text).and_then(map_response) {
            Ok(resp) => Ok(resp),
            Err(e) => {
                tracing::warn!("LLM 响应解析失败，启动自修轮（≤1 轮）: {e}");
                let repair = Self::build_repair_request(req, &text, &e);
                let repair_request = Self::build_request(&repair)?;
                let mut repair_body = serde_json::to_value(&repair_request).map_err(|e2| {
                    CsError::new(LLM_BAD_RESPONSE, format!("自修请求序列化失败: {e2}"))
                })?;
                if self.thinking_disabled {
                    repair_body["thinking"] = serde_json::json!({"type": "disabled"});
                }
                let text2 = self.send_once(&url, &repair_body).await.map_err(|ce| {
                    CsError::new(
                        LLM_BAD_RESPONSE,
                        format!("响应解析失败且自修轮发送失败: 原={e}; 自修={ce}"),
                    )
                    .with_retryable(false)
                })?;
                match Self::decode_response(&text2).and_then(map_response) {
                    Ok(resp) => {
                        tracing::info!("LLM 自修轮收敛（解析恢复）");
                        Ok(resp)
                    }
                    Err(e2) => {
                        // EN-17/D032：稳定原因码——坏响应解出 finish_reason=length 判
                        // Truncated（输出约束问题），否则 ParseFail（schema 漂移）
                        let truncated = text.contains(r#""finish_reason":"length""#)
                            || text.contains(r#""finish_reason": "length""#);
                        let ce = CsError::new(
                            LLM_BAD_RESPONSE,
                            format!("LLM 响应解析失败（含自修轮）: 原轮={e}; 自修轮={e2}"),
                        )
                        .with_reason(if truncated {
                            FailureReason::Truncated
                        } else {
                            FailureReason::ParseFail
                        });
                        tracing::error!("LLM 调用终局失败: {ce}");
                        Err(ce.with_retryable(false))
                    }
                }
            }
        }
    }
}

impl OpenAiProvider {
    /// 单次发送（不重试不退避）：状态码分类内聚（P007 R3.1 classify_llm_error），
    /// 200 后读响应体；传输瞬断归 LLM_UNREACHABLE+retryable（P007 R3.2）。
    async fn send_once(&self, url: &str, body: &serde_json::Value) -> Result<String, CsError> {
        let resp = self
            .http
            .post(url)
            .bearer_auth(&self.api_key)
            .json(body)
            .send()
            .await
            .map_err(|e| {
                CsError::new(LLM_UNREACHABLE, format!("LLM 请求失败: {e}")).with_retryable(true)
            })?;
        let status = resp.status();
        if !status.is_success() {
            let snippet = resp.text().await.unwrap_or_default();
            let (code, retryable) = classify_llm_error(Some(status.as_u16()), &snippet);
            return Err(CsError::new(
                code,
                format!("LLM HTTP {status}: {}", safe_prefix(&snippet, 200)),
            )
            .with_retryable(retryable));
        }
        resp.text().await.map_err(|e| {
            CsError::new(
                LLM_UNREACHABLE,
                format!("LLM 响应体读取失败（连接中断？）: {e}"),
            )
            .with_retryable(true)
        })
    }

    /// 带重试退避的发送：attempts = max_retries+1；指数退避 1s/2s/4s/8s（封顶）。
    async fn send_with_retry(&self, url: &str, body: &serde_json::Value) -> CsResult<String> {
        let attempts = self.max_retries.saturating_add(1).max(1);
        let mut last: Option<CsError> = None;
        for attempt in 0..attempts {
            if attempt > 0 {
                // 指数退避：实现为 500ms << attempt（attempt 从 1 起）= 1s / 2s / 4s / 8s（封顶）。
                // P007 R3.15：注释曾写 500ms/1s/2s/4s，与实现不符，已同步。
                tokio::time::sleep(Duration::from_millis(500u64 << attempt.min(4))).await;
            }
            match self.send_once(url, body).await {
                Ok(t) => return Ok(t),
                Err(ce) => {
                    if ce.retryable && attempt + 1 < attempts {
                        tracing::warn!("LLM 第 {} 次调用失败（可重试）: {}", attempt + 1, ce);
                        last = Some(ce);
                        continue;
                    }
                    // 终局失败 ERROR 级必打（P004 T4 语义：防线失败不可随 -v 静默）
                    tracing::error!("LLM 调用终局失败: {ce}");
                    return Err(ce.with_retryable(false));
                }
            }
        }
        Err(last
            .unwrap_or_else(|| CsError::new(LLM_UNREACHABLE, "未知错误"))
            .with_retryable(false))
    }
}

/// Arc 便利别名。
pub type SharedProvider = Arc<dyn LlmProvider>;

/// P007 R3.1：LLM 错误分类单一事实源（原为 #[cfg(test)] 影子分类器且与生产分叉——
/// 生产把 5xx 归 LLM_RATE_LIMITED，测试归 LLM_SERVER）。现生产 chat() 直调本函数。
fn classify_llm_error(status: Option<u16>, body: &str) -> (CsCode, bool) {
    let lower = body.to_lowercase();
    match status {
        Some(429) => (LLM_RATE_LIMITED, true),
        Some(401) | Some(403) => (CONFIG_MISSING, false),
        Some(s) if s >= 500 => (LLM_SERVER, true),
        Some(_) => (LLM_BAD_RESPONSE, false),
        None => {
            if lower.contains("rate") || lower.contains("429") {
                (LLM_RATE_LIMITED, true)
            } else if lower.contains("unauthorized") || lower.contains("api key") {
                (CONFIG_MISSING, false)
            } else {
                (LLM_UNREACHABLE, true)
            }
        }
    }
}

/// 字符边界安全截断：切点落在 UTF-8 多字节字符中缝时回退到上一个边界，绝不 panic。
pub(crate) fn safe_prefix(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_prefix_never_panics_on_multibyte_boundary() {
        // 网关中文错误体（如「余额不足」）恰好被字节 200 切进多字节字符中缝的场景
        let chinese = "余额不足：账户剩余额度请查纳".repeat(20);
        assert!(safe_prefix(&chinese, 200).len() <= 200); // 不 panic 且不超上限
        assert_eq!(safe_prefix("余额不足", 5), "余"); // 字节 5 在「额」中缝 → 回退到 3
        assert_eq!(safe_prefix("abc", 200), "abc"); // 短于上限原样返回
    }

    #[test]
    fn message_mapping_preserves_roles() {
        let msgs = [
            ChatMessage::System {
                content: "s".into(),
            },
            ChatMessage::User {
                content: "u".into(),
            },
            ChatMessage::Assistant {
                content: Some("a".into()),
                tool_calls: vec![ToolCallSpec {
                    id: "1".into(),
                    name: "echo".into(),
                    arguments: "{}".into(),
                }],
            },
            ChatMessage::Tool {
                call_id: "1".into(),
                content: "r".into(),
            },
        ];
        let mapped: Vec<ChatCompletionRequestMessage> =
            msgs.iter().map(to_request_message).collect();
        let v = serde_json::to_value(&mapped).unwrap();
        assert_eq!(v[0]["role"], "system");
        assert_eq!(v[1]["role"], "user");
        assert_eq!(v[2]["role"], "assistant");
        assert_eq!(v[2]["tool_calls"][0]["function"]["name"], "echo");
        assert_eq!(v[3]["role"], "tool");
        assert_eq!(v[3]["tool_call_id"], "1");
    }

    #[test]
    fn decode_response_tolerates_nonstandard_service_tier() {
        // D015：MiniMax-M3 经 newapi 网关回 `service_tier: "standard"`（OpenAI 枚举之外），
        // async-openai 严格枚举零容忍——typed 直解必须炸，容错解码必须活。
        let raw = r#"{"id":"x","choices":[{"finish_reason":"tool_calls","index":0,"message":{"content":"好","role":"assistant","name":"MiniMax AI","tool_calls":[{"id":"c1","type":"function","function":{"name":"grep","arguments":"{\"pattern\":\"retry\"}"},"index":0}],"audio_content":""}}],"created":1,"model":"MiniMax-M3","object":"chat.completion","usage":{"total_tokens":10,"prompt_tokens":8,"completion_tokens":2},"input_sensitive":false,"service_tier":"standard","base_resp":{"status_code":0,"status_msg":""}}"#;

        // 对照组：typed 直解（等价于修复前的 resp.json::<T>()）确实炸
        let direct: Result<CreateChatCompletionResponse, _> = serde_json::from_str(raw);
        assert!(direct.is_err(), "对照组应复现 unknown variant 炸点");

        // 修复后：容错解码成功，字段语义不丢
        let resp = OpenAiProvider::decode_response(raw).expect("非标 service_tier 不应炸解码");
        assert_eq!(resp.choices[0].message.content.as_deref(), Some("好"));
        let tcs = resp.choices[0]
            .message
            .tool_calls
            .clone()
            .unwrap_or_default();
        assert_eq!(tcs.len(), 1);
    }

    #[test]
    fn decode_response_rejects_non_json_and_keeps_error_semantics() {
        let err = OpenAiProvider::decode_response("<html>gateway 502</html>").unwrap_err();
        assert_eq!(err.code, LLM_BAD_RESPONSE);
        assert!(!err.retryable);
    }

    // ===== EN-15/D029：宽松归一化（第一层，纯本地）=====

    #[test]
    fn normalize_wraps_scalar_choices_and_repairs_envelope() {
        // 炸点①②③：choices 单对象 + finish_reason 未知 + message 缺失
        let mut v: serde_json::Value = serde_json::from_str(
            r#"{"id":"x","object":"chat.completion","created":1,"model":"m","choices":{"finish_reason":"mini_max","index":0}}"#,
        )
        .unwrap();
        OpenAiProvider::normalize_response_value(&mut v);
        let resp: CreateChatCompletionResponse = serde_json::from_value(v).unwrap();
        assert_eq!(resp.choices.len(), 1);
        assert_eq!(resp.choices[0].message.content.as_deref(), Some(""));
    }

    #[test]
    fn normalize_joins_multipart_content_and_scalar_tool_calls() {
        // 炸点④⑤：content multipart 数组拼接 + tool_calls 单对象包数组
        let mut v: serde_json::Value = serde_json::from_str(
            r#"{"id":"x","object":"chat.completion","created":1,"model":"m","choices":[{"finish_reason":"stop","index":0,"message":{"role":"assistant","content":[{"type":"text","text":"好"},{"type":"text","text":"的"}],"tool_calls":{"id":"c1","type":"function","function":{"name":"grep","arguments":"{}"}}}}]}"#,
        )
        .unwrap();
        OpenAiProvider::normalize_response_value(&mut v);
        let resp: CreateChatCompletionResponse = serde_json::from_value(v).unwrap();
        assert_eq!(resp.choices[0].message.content.as_deref(), Some("好的"));
        let tcs = resp.choices[0]
            .message
            .tool_calls
            .clone()
            .unwrap_or_default();
        assert_eq!(tcs.len(), 1);
        match &tcs[0] {
            ChatCompletionMessageToolCalls::Function(f) => {
                assert_eq!(f.function.name, "grep");
            }
            _ => panic!("应为 function 型 tool_call"),
        }
    }

    #[test]
    fn normalize_repairs_usage_shape() {
        // 炸点⑥⑦：usage null → 删除；usage 字符串数字 → 转数字
        let mut v: serde_json::Value = serde_json::from_str(
            r#"{"id":"x","object":"chat.completion","created":1,"model":"m","choices":[{"index":0,"finish_reason":"stop","message":{"role":"assistant","content":"x"}}],"usage":null}"#,
        )
        .unwrap();
        OpenAiProvider::normalize_response_value(&mut v);
        let resp: CreateChatCompletionResponse = serde_json::from_value(v).unwrap();
        // usage null 被删除 → typed 解码不再炸，usage 为 None（map_response 层 unwrap_or_default 兕底）
        assert!(resp.usage.is_none());

        let mut v2: serde_json::Value = serde_json::from_str(
            r#"{"id":"x","object":"chat.completion","created":1,"model":"m","choices":[{"index":0,"finish_reason":"stop","message":{"role":"assistant","content":"x"}}],"usage":{"prompt_tokens":"8","completion_tokens":"2","total_tokens":"10"}}"#,
        )
        .unwrap();
        OpenAiProvider::normalize_response_value(&mut v2);
        let resp2: CreateChatCompletionResponse = serde_json::from_value(v2).unwrap();
        let u = resp2.usage.as_ref().unwrap();
        assert_eq!(u.total_tokens, 10);
        assert_eq!(u.prompt_tokens, 8);
    }

    #[test]
    fn decode_response_recovers_after_normalization() {
        // EN-15 端到端（解码层）：typed 直解炸（choices 单对象）→ 宽松归一救回
        let raw = r#"{"id":"x","object":"chat.completion","created":1,"model":"m","choices":{"finish_reason":"stop","index":0,"message":{"role":"assistant","content":"回复正文"}},"usage":{"total_tokens":5,"prompt_tokens":4,"completion_tokens":1}}"#;
        let direct: Result<CreateChatCompletionResponse, _> = serde_json::from_str(raw);
        assert!(direct.is_err(), "对照组应复现 invalid type 炸点");
        let resp = OpenAiProvider::decode_response(raw).expect("宽松归一后应解码成功");
        assert_eq!(resp.choices[0].message.content.as_deref(), Some("回复正文"));
    }

    #[test]
    fn decode_response_non_json_still_fails() {
        // 非 JSON 永远不可修：归一层不碰、自修轮才接手
        let err = OpenAiProvider::decode_response("<html>gateway 502</html>").unwrap_err();
        assert_eq!(err.code, LLM_BAD_RESPONSE);
        assert!(!err.retryable);
    }

    // ===== EN-15/D029：自修轮（第二层）=====

    #[test]
    fn repair_request_appends_single_user_with_error_context() {
        let req = ChatRequest {
            model: "m".into(),
            messages: vec![ChatMessage::User {
                content: "任务".into(),
            }],
            tools: vec![ToolSchema {
                name: "grep".into(),
                description: "d".into(),
                parameters: serde_json::json!({}),
            }],
        };
        let err = CsError::new(LLM_BAD_RESPONSE, "invalid type: map, expected a sequence");
        let repair = OpenAiProvider::build_repair_request(&req, "坏响应原文片段", &err);
        assert_eq!(repair.messages.len(), 2, "恰好追加 1 条消息");
        assert_eq!(repair.model, "m");
        assert_eq!(repair.tools.len(), 1, "tools 保留");
        let ChatMessage::User { content } = &repair.messages[1] else {
            panic!("追加的应是 User 消息");
        };
        assert!(content.contains("invalid type: map"));
        assert!(content.contains("坏响应原文片段"));
    }

    // ===== EN-15/D029：端到端（one-shot TCP mock，零新依赖）=====

    async fn spawn_mock(
        bodies: Vec<String>,
        hits: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            for body in bodies {
                let (mut sock, _) = listener.accept().await.unwrap();
                hits.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                // 读一次请求（测试请求体小，64KB 足够），不解析——HTTP 服务端可未读完先答
                let mut buf = vec![0u8; 65536];
                let _ = AsyncReadExt::read(&mut sock, &mut buf).await;
                let mut http = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                )
                .into_bytes();
                http.extend_from_slice(body.as_bytes());
                let _ = sock.write_all(&http).await;
                let _ = sock.shutdown().await;
            }
        });
        format!("http://{addr}")
    }

    fn good_body(content: &str) -> String {
        format!(
            r#"{{"id":"x","object":"chat.completion","created":1,"model":"m","choices":[{{"finish_reason":"stop","index":0,"message":{{"content":"{content}","role":"assistant"}}}}],"usage":{{"total_tokens":5,"prompt_tokens":4,"completion_tokens":1}}}}"#
        )
    }

    fn unsalvageable_body() -> String {
        // choices 为字符串：宽松归一救不了（既非对象也非数组），必触发自修
        r#"{"id":"x","object":"chat.completion","created":1,"model":"m","choices":"垃圾响应"}"#
            .into()
    }

    fn test_request() -> ChatRequest {
        ChatRequest {
            model: "m".into(),
            messages: vec![ChatMessage::User {
                content: "任务".into(),
            }],
            tools: vec![],
        }
    }

    #[tokio::test]
    async fn chat_lenient_layer_avoids_repair_round() {
        // 第一层宽松归一就地救回（choices 单对象）：仅 1 次请求，零自修零重试
        let hits = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let salvageable = r#"{"id":"x","object":"chat.completion","created":1,"model":"m","choices":{"finish_reason":"stop","index":0,"message":{"role":"assistant","content":"归一救回"}}}"#.to_string();
        let base = spawn_mock(vec![salvageable], hits.clone()).await;
        let provider = OpenAiProvider::new(&base, "k", 0, true);
        let resp = provider
            .chat(&test_request())
            .await
            .expect("宽松层应就地救回");
        assert_eq!(resp.text.as_deref(), Some("归一救回"));
        assert_eq!(
            hits.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "仅 1 次请求：宽松层兜住，未触发自修"
        );
    }

    #[tokio::test]
    async fn chat_recovers_via_single_repair_round() {
        // 原轮不可修畸形 → 自修 1 轮收敛：恰好 2 次请求（max_retries=0，无重试干扰）
        let hits = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let base = spawn_mock(
            vec![unsalvageable_body(), good_body("自修收敛")],
            hits.clone(),
        )
        .await;
        let provider = OpenAiProvider::new(&base, "k", 0, true);
        let resp = provider.chat(&test_request()).await.expect("自修轮应收敛");
        assert_eq!(resp.text.as_deref(), Some("自修收敛"));
        assert_eq!(resp.usage.total_tokens, 5, "usage 取自修轮");
        assert_eq!(
            hits.load(std::sync::atomic::Ordering::SeqCst),
            2,
            "恰好 2 次请求（原 1 + 自修 1）"
        );
    }

    #[tokio::test]
    async fn chat_repair_round_is_capped_at_one() {
        // 两轮全畸形 → CS2004 终局；恰好 2 次请求：自修不循环、不占 max_retries 额度
        let hits = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let base = spawn_mock(
            vec![unsalvageable_body(), unsalvageable_body()],
            hits.clone(),
        )
        .await;
        let provider = OpenAiProvider::new(&base, "k", 0, true);
        let err = provider
            .chat(&test_request())
            .await
            .expect_err("两轮全畸形应终局失败");
        assert_eq!(err.code, LLM_BAD_RESPONSE);
        assert!(!err.retryable);
        assert_eq!(err.reason, Some(FailureReason::ParseFail));
        assert!(
            err.to_string().contains("自修轮"),
            "终局错误应带两轮上下文: {err}"
        );
        assert_eq!(
            hits.load(std::sync::atomic::Ordering::SeqCst),
            2,
            "自修恰好 1 轮：不循环、不占重试额度"
        );
    }

    #[test]
    fn error_classification() {
        let (c, r) = classify_llm_error(Some(429), "");
        assert_eq!(c, LLM_RATE_LIMITED);
        assert!(r);
        let (c, r) = classify_llm_error(Some(401), "unauthorized");
        assert_eq!(c, CONFIG_MISSING);
        assert!(!r);
        let (c, r) = classify_llm_error(None, "connection refused");
        assert_eq!(c, LLM_UNREACHABLE);
        assert!(r);
        let (c, r) = classify_llm_error(Some(400), "bad request");
        assert_eq!(c, LLM_BAD_RESPONSE);
        assert!(!r);
        let (c, r) = classify_llm_error(Some(503), "");
        assert_eq!(c, LLM_SERVER);
        assert!(r);
    }
}
