//! LLM provider 抽象（Q001/D009）：OpenAI 兼容协议，async-openai + 自定义 base_url。
//! 重试/退避在本层做；最终失败后 retryable=false（宿主不再重试）。

use crate::errors::{CsError, CsResult, LLM_BAD_RESPONSE, LLM_RATE_LIMITED, LLM_UNREACHABLE};
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
        let attempts = self.max_retries.saturating_add(1).max(1);
        let mut last: Option<CsError> = None;
        for attempt in 0..attempts {
            if attempt > 0 {
                // 指数退避：500ms / 1s / 2s / 4s（封顶）
                tokio::time::sleep(Duration::from_millis(500u64 << attempt.min(4))).await;
            }
            let outcome = self
                .http
                .post(&url)
                .bearer_auth(&self.api_key)
                .json(&body)
                .send()
                .await;
            match outcome {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        let snippet = resp.text().await.unwrap_or_default();
                        let ce = if status.as_u16() == 429 || status.is_server_error() {
                            CsError::new(
                                LLM_RATE_LIMITED,
                                format!("LLM HTTP {status}: {}", safe_prefix(&snippet, 200)),
                            )
                            .with_retryable(true)
                        } else {
                            CsError::new(
                                LLM_BAD_RESPONSE,
                                format!("LLM HTTP {status}: {}", safe_prefix(&snippet, 200)),
                            )
                        };
                        if ce.retryable && attempt + 1 < attempts {
                            tracing::warn!("LLM 第 {} 次调用失败（可重试）: {}", attempt + 1, ce);
                            last = Some(ce);
                            continue;
                        }
                        tracing::warn!("LLM 调用终局失败: {ce}");
                        return Err(ce.with_retryable(false));
                    }
                    match resp.json::<CreateChatCompletionResponse>().await {
                        Ok(resp) => return map_response(resp),
                        Err(e) => {
                            let ce =
                                CsError::new(LLM_BAD_RESPONSE, format!("LLM 响应解析失败: {e}"));
                            tracing::warn!("LLM 调用终局失败: {ce}");
                            return Err(ce.with_retryable(false));
                        }
                    }
                }
                Err(e) => {
                    let ce = CsError::new(LLM_UNREACHABLE, format!("LLM 请求失败: {e}"))
                        .with_retryable(true);
                    if attempt + 1 < attempts {
                        tracing::warn!("LLM 第 {} 次调用失败（可重试）: {}", attempt + 1, ce);
                        last = Some(ce);
                        continue;
                    }
                    tracing::warn!("LLM 调用终局失败: {ce}");
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

#[cfg(test)]
use crate::errors::{CONFIG_MISSING, CsCode, LLM_SERVER};
#[cfg(test)]
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
fn safe_prefix(s: &str, max_bytes: usize) -> &str {
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
