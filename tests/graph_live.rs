//! codegraph MCP 生命周期真机验收（默认忽略，需本机装有 codegraph）：
//!   cargo test --test graph_live -- --ignored --nocapture
//!
//! 验收 D011：init（缺失时）→ spawn `codegraph serve --mcp` → initialize 握手 →
//! 五工具真实调用 → Drop 回收。

use codesleuth::tools::graph::CodegraphEngine;
use std::path::PathBuf;

#[tokio::test]
#[ignore = "codegraph MCP 真机验收：需要本机安装 codegraph"]
async fn codegraph_mcp_lifecycle() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fixture-rs");

    // 1) 生命周期入口：索引缺失则 init，然后 spawn MCP server + 握手
    let engine = CodegraphEngine::start(&repo, false)
        .await
        .expect("codegraph 引擎启动失败");
    assert!(
        repo.join(".codegraph").exists(),
        "init 应产出 .codegraph/ 索引目录"
    );

    // 2) 五工具真实调用（MCP tools/call）
    let explore = engine
        .call("explore", serde_json::json!({"query": "retry"}))
        .await
        .expect("explore 失败");
    println!("=== explore ===\n{explore:.500}");
    assert!(!explore.trim().is_empty());

    let callers = engine
        .call(
            "callers",
            serde_json::json!({"symbol": "retry_with_backoff"}),
        )
        .await
        .expect("callers 失败");
    println!("=== callers ===\n{callers:.300}");
    let files = engine
        .call("files", serde_json::json!({}))
        .await
        .expect("files 失败");
    assert!(files.contains("retry.rs"), "files 应列出 retry.rs：{files}");

    // 3) Drop 回收（无断言——不 panic 即可；进程由 Drop kill）
    drop(engine);
}
