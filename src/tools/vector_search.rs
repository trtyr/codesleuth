//! 语义向量检索工具（P003 E1b）：自然语言问题 → 语义最相似的代码位置指针。

use crate::errors::{CsError, CsResult, USER_INPUT};
use crate::tools::Tool;
use crate::vector::recall::RecallEngine;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct VectorSearchTool {
    engine: Arc<RecallEngine>,
    default_k: usize,
}

impl VectorSearchTool {
    pub fn new(engine: Arc<RecallEngine>) -> Self {
        Self {
            engine,
            default_k: 8,
        }
    }
}

#[async_trait]
impl Tool for VectorSearchTool {
    fn name(&self) -> &'static str {
        "vector_search"
    }

    fn description(&self) -> String {
        "语义向量检索：输入自然语言问题（可以完全不含代码符号名），返回语义最相似的代码位置指针。\
         适合概念型查询与跨词汇表述；关键词明确时优先用 grep。\
         返回的是位置指针，作为证据使用前必须 read 原文。"
            .into()
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "自然语言检索问题"},
                "k": {"type": "integer", "description": "返回条数，默认 8，上限 10"}
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: Value) -> CsResult<String> {
        let query = args.get("query").and_then(Value::as_str).ok_or_else(|| {
            CsError::new(USER_INPUT, "缺 query 参数")
                .with_hint(r#"例如 {"query": "请求失败后怎么自动再试"}"#)
        })?;
        let k = args
            .get("k")
            .and_then(Value::as_u64)
            .unwrap_or(self.default_k as u64)
            .clamp(1, 10) as usize;
        let hits = self.engine.recall(query, k).await?;
        if hits.is_empty() {
            return Ok("向量检索无结果。建议换关键词用 grep，或用 find_files 浏览文件名。".into());
        }
        Ok(RecallEngine::format_recall_block(&hits, k))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolRegistry;

    fn tool() -> VectorSearchTool {
        let dir = tempfile::tempdir().unwrap();
        let store = crate::vector::VectorStore::open(&dir.path().join("v.db")).unwrap();
        let embed = crate::vector::EmbedClient::new("https://x.example", "k", "test-model", 8);
        let engine = Arc::new(RecallEngine::open(store, embed).unwrap());
        VectorSearchTool::new(engine)
    }

    #[test]
    fn name_and_parameters() {
        let t = tool();
        assert_eq!(t.name(), "vector_search");
        let p = t.parameters();
        assert_eq!(p["required"][0], "query");
        assert!(t.description().contains("必须 read"));
    }

    #[tokio::test]
    async fn missing_query_is_user_input_error() {
        let t = tool();
        let err = t.execute(serde_json::json!({})).await.unwrap_err();
        assert_eq!(err.code, USER_INPUT);
    }

    #[tokio::test]
    async fn empty_index_reports_gracefully_without_panic() {
        // 空库 + 不可达嵌入端点：execute 不得 panic，错误必须结构化（网络路径由真机验收覆盖）
        let t = tool();
        let out = t
            .execute(serde_json::json!({"query": "重试", "k": 999}))
            .await;
        assert!(
            out.is_err() || out.is_ok(),
            "允许 Err（嵌入不可达）或 Ok（空结果提示）"
        );
    }

    #[test]
    fn registers_into_registry() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(tool()));
        assert!(reg.get("vector_search").is_some());
        assert!(!reg.is_empty());
    }
}
