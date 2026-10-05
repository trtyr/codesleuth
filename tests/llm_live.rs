//! 真网关冒烟（默认忽略）：需 ~/.codesleuth/config.toml 配好 [llm]
//!   cargo test --test llm_live -- --ignored --nocapture

use codesleuth::llm::{ChatMessage, ChatRequest, LlmProvider, OpenAiProvider};

#[tokio::test]
#[ignore = "真网关冒烟：需要 ~/.codesleuth/config.toml 配好 [llm]"]
async fn live_gateway_smoke() {
    let cfg = codesleuth::config::load(Default::default()).expect("配置加载失败");
    let key = cfg
        .resolve_api_key()
        .expect("需要 ~/.codesleuth/config.toml 的 [llm] api_key");
    let base = cfg.llm.base_url.clone();
    let model = cfg.llm.model.clone();

    let provider = OpenAiProvider::new(&base, &key, 1, false);
    let req = ChatRequest {
        model,
        messages: vec![ChatMessage::User {
            content: "只回答两个字：在吗".into(),
        }],
        tools: vec![],
    };
    let resp = provider.chat(&req).await.expect("网关调用失败");
    let text = resp.text.clone().unwrap_or_default();
    assert!(!text.trim().is_empty(), "响应文本为空");
    println!("响应: {text}");
    println!("usage: {:?}", resp.usage);
    assert!(resp.usage.total_tokens > 0, "usage 应含 token 计数");
}
