//! 层进 v0 端到端真网关冒烟（默认忽略）：
//!   需 ~/.codesleuth/config.toml 配好 [llm] \
//!   cargo test --test layered_live -- --ignored --nocapture
//!
//! 验收：摸点（find_files/grep）→ 落锤有证（read）→ 报告含 file:line 定位。

use codesleuth::audit::{Audit, Ledger};
use codesleuth::fence::Fence;
use codesleuth::harness::Harness;
use codesleuth::llm::{LlmProvider, OpenAiProvider};
use codesleuth::tools::ToolRegistry;
use codesleuth::tools::fuzzy::{FileFinderTool, FuzzyEngine, GrepTool};
use codesleuth::tools::read::ReadTool;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::test]
#[ignore = "层进 v0 真网关冒烟：需要 ~/.codesleuth/config.toml 配好 [llm]"]
async fn layered_v0_smoke() {
    let cfg = codesleuth::config::load(Default::default()).expect("配置加载失败");
    let key = cfg.resolve_api_key().expect("需要 ~/.codesleuth/config.toml 的 [llm] api_key");
    let base = cfg.llm.base_url.clone();
    let model = cfg.llm.model.clone();

    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fixture-rs");
    let fence = Arc::new(Fence::new(&repo).unwrap());
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(ReadTool::new(fence.clone())));
    let engine = Arc::new(FuzzyEngine::new(&repo).unwrap());
    registry.register(Box::new(FileFinderTool::new(engine.clone())));
    registry.register(Box::new(GrepTool::new(engine)));

    let provider: Arc<dyn LlmProvider> = Arc::new(OpenAiProvider::new(&base, &key, 2, false));
    let dir = tempfile::tempdir().unwrap();
    let audit = Audit::create(dir.path(), "smoke").unwrap();
    let ledger = Ledger::load(dir.path().join("ledger.json"));
    let agent = Harness::new(provider, registry, audit, ledger, model, 1_000_000, 60);

    let outcome = agent
        .run("这个样例仓库的重试逻辑在哪个文件哪一行？给出 file:line 证据。")
        .await
        .expect("侦察失败");
    println!(
        "=== 报告 ===\n{}\n=== turns={} tool_calls={} ===",
        outcome.answer, outcome.turns, outcome.tool_calls
    );
    assert!(
        outcome.tool_calls >= 2,
        "层进 v0 应至少摸点+读证据两次工具调用"
    );
    let lower = outcome.answer.to_lowercase();
    assert!(
        lower.contains("retry"),
        "报告应定位到 retry 模块：{}",
        outcome.answer
    );
    assert!(outcome.answer.contains(':'), "报告应含 file:line 证据");
}
