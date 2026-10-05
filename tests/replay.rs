//! 录制回放契约测试（D003.3）：LLM 响应序列录制为 fixture，回放时断言
//! 工具调用序列与结构化报告——全程零网络、零真实 token。

use codesleuth::audit::{Audit, Ledger};
use codesleuth::fence::Fence;
use codesleuth::harness::Harness;
use codesleuth::llm::{ChatRequest, ChatResponse, LlmProvider, ToolCallSpec, Usage};
use codesleuth::tools::ToolRegistry;
use codesleuth::tools::fuzzy::{FileFinderTool, FuzzyEngine, GrepTool};
use codesleuth::tools::read::ReadTool;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

struct Replay {
    responses: Mutex<VecDeque<ChatResponse>>,
}

#[async_trait::async_trait]
impl LlmProvider for Replay {
    async fn chat(&self, _req: &ChatRequest) -> codesleuth::errors::CsResult<ChatResponse> {
        self.responses.lock().unwrap().pop_front().ok_or_else(|| {
            codesleuth::errors::CsError::new(codesleuth::errors::LLM_BAD_RESPONSE, "回放脚本耗尽")
        })
    }
}

fn load_script() -> (Arc<Mutex<Vec<String>>>, VecDeque<ChatResponse>) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/replay/fixture-rs-replay.json");
    let fixture: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("回放 fixture 缺失"))
            .expect("回放 fixture 非法");
    let mut out = VecDeque::new();
    let seq = Arc::new(Mutex::new(Vec::new()));
    for resp in fixture["responses"].as_array().expect("responses 数组") {
        let calls: Vec<ToolCallSpec> = resp["tool_calls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| ToolCallSpec {
                id: c["id"].as_str().unwrap().into(),
                name: c["name"].as_str().unwrap().into(),
                arguments: c["arguments"].as_str().unwrap().into(),
            })
            .collect();
        for c in &calls {
            seq.lock().unwrap().push(c.name.clone());
        }
        out.push_back(ChatResponse {
            text: None,
            tool_calls: calls,
            usage: Usage::default(),
        });
    }
    (seq, out)
}

#[tokio::test]
async fn recorded_replay_produces_same_tool_sequence_and_report() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fixture-rs");
    let (tool_sequence, responses) = load_script();

    let fence = Arc::new(Fence::new(&repo).unwrap());
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(ReadTool::new(fence.clone())));
    let engine = Arc::new(FuzzyEngine::new(&repo).unwrap());
    registry.register(Box::new(FileFinderTool::new(engine.clone())));
    registry.register(Box::new(GrepTool::new(engine)));

    let provider = Arc::new(Replay {
        responses: Mutex::new(responses),
    });
    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::create(dir.path(), "replay").unwrap();
    let agent = Harness::new(
        provider,
        registry,
        audit,
        Ledger::load(dir.path().join("l.json")),
        "replayed-model".into(),
        1_000_000,
        60,
    );

    let outcome = agent
        .run("这个样例仓库的重试逻辑在哪个文件哪一行？")
        .await
        .expect("回放失败");

    // 契约 1：工具调用序列与录制一致
    let executed: Vec<&str> = vec!["find_files", "grep", "read", "submit_report"];
    assert_eq!(outcome.tool_calls, 3);
    assert_eq!(
        tool_sequence
            .lock()
            .unwrap()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
        executed,
    );
    // 契约 2：报告 schema + 证据指向
    assert_eq!(outcome.report.report_schema_version, 1);
    assert!(!outcome.report.degraded);
    assert_eq!(outcome.report.findings[0].evidence[0].file, "src/retry.rs");
}
