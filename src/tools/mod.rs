//! 工具注册表（D009 只读边界的根本防线：接口上就不存在写能力）。
//! 子模块：read（强读）/ fuzzy（模糊检索）/ graph（结构图，codegraph MCP）。

pub mod fuzzy;
pub mod graph;
pub mod read;
pub mod vector_search;

use crate::errors::CsResult;
use crate::llm::ToolSchema;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> String;
    /// JSON Schema（对象）。
    fn parameters(&self) -> Value;
    /// 只读执行。args 为已解析 JSON。
    async fn execute(&self, args: Value) -> CsResult<String>;
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .map(|b| b.as_ref())
    }

    pub fn schemas(&self) -> Vec<ToolSchema> {
        self.tools
            .iter()
            .map(|t| ToolSchema {
                name: t.name().to_string(),
                description: t.description(),
                parameters: t.parameters(),
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::CsError;

    struct Dummy;
    #[async_trait]
    impl Tool for Dummy {
        fn name(&self) -> &'static str {
            "dummy"
        }
        fn description(&self) -> String {
            "dummy tool".into()
        }
        fn parameters(&self) -> Value {
            serde_json::json!({"type": "object"})
        }
        async fn execute(&self, _args: Value) -> CsResult<String> {
            Err(CsError::new(crate::errors::INTERNAL, "not used"))
        }
    }

    #[test]
    fn registry_roundtrip() {
        let mut reg = ToolRegistry::new();
        assert!(reg.is_empty());
        reg.register(Box::new(Dummy));
        assert!(!reg.is_empty());
        assert!(reg.get("dummy").is_some());
        assert!(reg.get("nope").is_none());
        let schemas = reg.schemas();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].name, "dummy");
    }
}
