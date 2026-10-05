//! 只读对抗三断言（D009 三层防线的产品命门验证）：
//!   ① 工具面无写能力：全部注册工具名 ⊆ 只读允许清单
//!   ② 路径围栏：symlink/绝对路径逃逸一律拒（CS3003）
//!   ③ 目标仓库零写入：完整会话前后，目标目录指纹逐字节不变

use codesleuth::audit::{Audit, Ledger};
use codesleuth::fence::Fence;
use codesleuth::harness::Harness;
use codesleuth::llm::{ChatRequest, ChatResponse, LlmProvider, ToolCallSpec, Usage};
use codesleuth::tools::ToolRegistry;
use codesleuth::tools::fuzzy::{FileFinderTool, FuzzyEngine, GrepTool};
use codesleuth::tools::read::ReadTool;
use sha2::Digest;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// 只读允许清单（D011：explore/callers/callees/impact/files 仅转发 codegraph MCP 查询，无写面）。
const READ_ONLY_TOOLS: &[&str] = &[
    "read",
    "find_files",
    "grep",
    "explore",
    "callers",
    "callees",
    "impact",
    "files",
];

fn build_registry(root: &Path) -> ToolRegistry {
    let fence = Arc::new(Fence::new(root).unwrap());
    let mut reg = ToolRegistry::new();
    reg.register(Box::new(ReadTool::new(fence.clone())));
    let engine = Arc::new(FuzzyEngine::new(root).unwrap());
    reg.register(Box::new(FileFinderTool::new(engine.clone())));
    reg.register(Box::new(GrepTool::new(engine)));
    reg
}

/// 递归指纹：(相对路径, sha256(content))，按路径排序。
fn fingerprint(root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    fn walk(dir: &Path, root: &Path, out: &mut BTreeMap<String, String>) {
        for entry in std::fs::read_dir(dir).expect("read_dir") {
            let entry = entry.unwrap();
            let p = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if matches!(
                name.as_str(),
                ".git" | ".codegraph" | "target" | "node_modules"
            ) {
                continue; // 工具元数据不参与源码指纹（.codegraph 归 codegraph 所有，D011）
            }
            if p.is_dir() {
                walk(&p, root, out);
            } else {
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                let content = std::fs::read(&p).unwrap();
                let h = sha2::Sha256::digest(&content);
                out.insert(rel, format!("{:x}", h));
            }
        }
    }
    walk(root, root, &mut out);
    out
}

struct Scripted {
    responses: Mutex<VecDeque<ChatResponse>>,
}
#[async_trait::async_trait]
impl LlmProvider for Scripted {
    async fn chat(&self, _req: &ChatRequest) -> codesleuth::errors::CsResult<ChatResponse> {
        self.responses.lock().unwrap().pop_front().ok_or_else(|| {
            codesleuth::errors::CsError::new(codesleuth::errors::LLM_BAD_RESPONSE, "脚本耗尽")
        })
    }
}

fn resp_call(id: &str, name: &str, args: &str) -> ChatResponse {
    ChatResponse {
        text: None,
        tool_calls: vec![ToolCallSpec {
            id: id.into(),
            name: name.into(),
            arguments: args.into(),
        }],
        usage: Usage::default(),
    }
}

/// 递归复制目录（测试用，跳过 .codegraph/.git/target）。
fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();
        if matches!(
            name.as_str(),
            ".git" | ".codegraph" | "target" | "node_modules"
        ) {
            continue;
        }
        let from = entry.path();
        let to = dst.join(&name);
        if from.is_dir() {
            copy_dir(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

#[tokio::test]
async fn assertion_1_tool_surface_has_no_write_capability() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn a() {}\n").unwrap();
    let reg = build_registry(dir.path());
    let names: Vec<String> = reg.schemas().into_iter().map(|s| s.name).collect();
    assert!(!names.is_empty());
    for n in &names {
        assert!(
            READ_ONLY_TOOLS.contains(&n.as_str()),
            "工具 {n} 不在只读允许清单——出现写能力即产品 bug"
        );
    }
    // schema 层面也不得出现写入语义（string 检查 description/properties）
    for s in reg.schemas() {
        let blob = serde_json::to_string(&s).unwrap().to_lowercase();
        for banned in ["\"write\"", "delete_file", "run_command", "exec"] {
            assert!(
                !blob.contains(banned),
                "工具 {} 的 schema 含疑似写语义: {banned}",
                s.name
            );
        }
    }
}

#[tokio::test]
async fn assertion_2_fence_rejects_escapes() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join(".ssh_keys"), "secret").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        outside.path().join(".ssh_keys"),
        dir.path().join("innocent.txt"),
    )
    .unwrap();
    let fence = Fence::new(dir.path()).unwrap();
    for hostile in ["innocent.txt", "/etc/passwd", "../../etc/passwd"] {
        let err = fence.resolve(hostile).unwrap_err();
        assert_eq!(
            err.code,
            codesleuth::errors::FENCE_DENIED,
            "敌对路径 {hostile} 应被围栏拒绝"
        );
    }
}

#[tokio::test]
async fn assertion_3_target_repo_untouched_after_full_session() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fixture-rs");
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("target-repo");
    copy_dir(&src, &repo);
    let before = fingerprint(&repo);

    // 完整会话：find_files → read → submit_report（读真实文件，真实提交）
    let fence = Arc::new(Fence::new(&repo).unwrap());
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(ReadTool::new(fence.clone())));
    let engine = Arc::new(FuzzyEngine::new(&repo).unwrap());
    registry.register(Box::new(FileFinderTool::new(engine)));
    let responses: Vec<ChatResponse> = vec![
        resp_call("1", "find_files", "{\"query\":\"retry\"}"),
        resp_call("2", "read", "{\"path\":\"src/retry.rs\"}"),
        resp_call(
            "3",
            "submit_report",
            r#"{"answer":"重试在 src/retry.rs","findings":[{"statement":"retry_with_backoff","evidence":[{"file":"src/retry.rs","lines":"7-22"}]}],"dead_ends":[],"confidence":"high"}"#,
        ),
    ];
    let provider = Arc::new(Scripted {
        responses: Mutex::new(responses.into_iter().collect()),
    });
    let agent = Harness::new(
        provider,
        registry,
        Audit::create(dir.path(), "adv").unwrap(),
        Ledger::load(dir.path().join("l.json")),
        "m".into(),
        1_000_000,
        60,
    );
    let outcome = agent.run("重试逻辑在哪").await.expect("会话失败");
    assert!(!outcome.report.degraded);

    let after = fingerprint(&repo);
    assert_eq!(before, after, "目标仓库在完整会话后被改动——只读边界被击穿");
}
