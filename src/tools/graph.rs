//! 结图层：codegraph MCP 接入（D011，二次拍板：MCP 而非 CLI，生命周期归 codesleuth）。
//!
//! 生命周期：`start`（索引缺失则一次性 `codegraph init`；force 则重建；然后 spawn
//! `codegraph serve --mcp` 并完成 initialize 握手）→ 会话内五工具复用 → Drop 回收 server。
//! P007 R3.23：独占性实际由 Arc 所有权 + Drop start_kill 保证（旧注释声称设
//! CODEGRAPH_NO_DAEMON=1，代码从未设置——已删该虚假承诺）；
//! 子进程只注入 CODEGRAPH_TELEMETRY=0 与 DO_NOT_TRACK=1。增量同步交给 server 的
//! connect-time catch-up。

use crate::errors::{
    CsError, CsResult, INDEX_BUILD_FAILED, INDEX_NOT_AVAILABLE, INDEX_TIMEOUT, USER_INPUT,
};
use crate::mcp::McpClient;
use crate::tools::Tool;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

/// P007 R1.6：对手释放锁但产物缺失时的重试上限。
const LOST_ARTIFACT_RETRIES: u32 = 3;

/// R1.6 决策核（纯函数便于测试）：OpponentFinished 分支的下一步。
#[derive(Debug, PartialEq, Eq)]
enum LostStep {
    /// 产物就绪，直接 serve
    Proceed,
    /// 产物缺失，重新竞争引导锁
    Retry,
    /// 连续多次对手都没留下产物（疑似反复崩溃），放弃并报错
    Fail,
}

fn lost_step(artifact_exists: bool, retries: u32) -> LostStep {
    if artifact_exists {
        LostStep::Proceed
    } else if retries >= LOST_ARTIFACT_RETRIES {
        LostStep::Fail
    } else {
        LostStep::Retry
    }
}

#[derive(Debug)]
pub struct CodegraphEngine {
    client: McpClient,
    project_path: PathBuf,
}

/// 一次性 CLI 动作（init / index --force）。
async fn run_cli(bin: &str, args: &[&str], cwd: &Path) -> CsResult<()> {
    let out = tokio::time::timeout(
        Duration::from_secs(300),
        async {
            tokio::process::Command::new(bin)
                .args(args)
                .current_dir(cwd)
                .env("CODEGRAPH_TELEMETRY", "0")
                .env("DO_NOT_TRACK", "1")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
        },
    )
    .await
    .map_err(|_| CsError::new(INDEX_TIMEOUT, "codegraph init/index 超时（300s）"))?
    .map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CsError::new(INDEX_NOT_AVAILABLE, "codegraph CLI 未安装").with_hint(
                "npm i -g @colbymchenry/codegraph，或 curl -fsSL https://raw.githubusercontent.com/colbymchenry/codegraph/main/install.sh | sh",
            )
        } else {
            CsError::new(INDEX_NOT_AVAILABLE, format!("codegraph 启动失败: {e}"))
        }
    })?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        // P007 R3.23：stderr 为空时不再只给通用 hint——stdout 里的诊断一并入错误消息
        let stdout = String::from_utf8_lossy(&out.stdout);
        let detail = if !stderr.trim().is_empty() {
            stderr.to_string()
        } else if !stdout.trim().is_empty() {
            format!(
                "（stderr 为空，stdout 诊断）{}",
                crate::llm::safe_prefix(stdout.trim(), 500)
            )
        } else {
            String::new()
        };
        let mut err = CsError::new(
            INDEX_BUILD_FAILED,
            format!("codegraph {} 失败: {}", args.join(" "), detail),
        );
        // codegraph 锁竞争时 stderr 为空（2026-10-05 事故实测）：给不出根因就给出路
        if stderr.trim().is_empty() {
            err = err.with_hint(
                "codegraph 未输出错误详情；若曾并发初始化，可能是残留锁：codegraph unlock 后重试",
            );
        }
        return Err(err);
    }
    Ok(())
}

impl CodegraphEngine {
    /// 生命周期入口（D011；P005 R7.2：可执行名由配置链传入，CODEGRAPH_BIN env 已移除）。
    pub async fn start(root: &Path, bin: &str, force_reindex: bool) -> CsResult<Self> {
        Self::start_with_bin(root, bin, force_reindex).await
    }

    async fn start_with_bin(root: &Path, bin: &str, force_reindex: bool) -> CsResult<Self> {
        // D014：索引引导（init/force）进仓库级引导锁，进程间串行化；
        // D020：等待后获得 = 对手刚完成引导，索引已就绪——跳过 init/force 直接进 serve。
        // P007 R1.6：Lost≠「对手成功完成」——对手可能在 init/index --force 半途崩溃
        // （内核放锁，产物缺失/残缺），此时不得在空 .codegraph 上直接 serve；
        // 重新竞争引导锁（上限 LOST_ARTIFACT_RETRIES 次），超限诚实报错。
        let mut lost_retries: u32 = 0;
        loop {
            match crate::bootlock::acquire_guard(
                root,
                crate::bootlock::DEFAULT_TIMEOUT,
                "codegraph 索引引导",
            )? {
                crate::bootlock::BootLockOutcome::Won(guard) => {
                    if force_reindex {
                        run_cli(bin, &["index", "--force", "--quiet"], root).await?;
                    } else if !root.join(".codegraph").exists() {
                        run_cli(bin, &["init"], root).await?;
                    }
                    drop(guard); // 引导段结束即放锁；serve 阶段不持锁（D014）
                    break;
                }
                crate::bootlock::BootLockOutcome::OpponentFinished
                    if root.join(".codegraph").exists() =>
                {
                    tracing::warn!(
                        "对手进程刚完成 codegraph 索引引导，本进程复用其产物直接进 serve（D020）"
                    );
                    break;
                }
                crate::bootlock::BootLockOutcome::OpponentFinished => {
                    match lost_step(false, lost_retries) {
                        LostStep::Fail => {
                            return Err(
                                CsError::new(
                                    crate::errors::INDEX_LOCKED,
                                    "对手进程释放引导锁但未留下 .codegraph 产物（疑似多次中途崩溃）",
                                )
                                .with_hint("删除目标仓库的 .codesleuth/boot.lock 后重试，或手动运行 codegraph init"),
                            );
                        }
                        LostStep::Retry => {
                            lost_retries += 1;
                            tracing::warn!(
                                "对手释放引导锁但 .codegraph 缺失（对手疑似中途崩溃），重新竞争引导锁（{lost_retries}/{LOST_ARTIFACT_RETRIES}）"
                            );
                        }
                        LostStep::Proceed => unreachable!("artifact_exists=false 不会判 Proceed"),
                    }
                }
            }
        }
        let client = McpClient::spawn(bin, &["serve", "--mcp"], root).await?;
        let initialized = client.initialize().await?;
        // 服务器在 initialize 响应里自带 usage guidance（Phase 4 可捕获并入 system prompt）
        if let Some(instr) = initialized["result"]
            .get("instructions")
            .and_then(Value::as_str)
        {
            tracing::debug!("codegraph usage guidance: {instr}");
        }
        Ok(Self {
            client,
            project_path: root.to_path_buf(),
        })
    }

    /// 调用 codegraph MCP 工具（自动附 projectPath）。
    pub async fn call(&self, tool: &str, mut args: Value) -> CsResult<String> {
        args["projectPath"] = Value::String(self.project_path.display().to_string());
        self.client
            .call_tool(&format!("codegraph_{tool}"), args)
            .await
    }
}

fn symbol_args(args: &Value, tool: &str) -> CsResult<Value> {
    let symbol = args
        .get("symbol")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| CsError::new(USER_INPUT, format!("{tool} 缺少 symbol 参数")))?
        .to_string();
    let mut v = serde_json::json!({"symbol": symbol});
    if let Some(limit) = args.get("limit").and_then(Value::as_u64) {
        v["limit"] = Value::from(limit);
    }
    if let Some(depth) = args.get("depth").and_then(Value::as_u64) {
        v["depth"] = Value::from(depth);
    }
    Ok(v)
}

macro_rules! cg_symbol_tool {
    ($name:ident, $tool:literal, $doc:literal) => {
        pub struct $name {
            engine: Arc<CodegraphEngine>,
        }
        impl $name {
            pub fn new(engine: Arc<CodegraphEngine>) -> Self {
                Self { engine }
            }
        }
        #[async_trait::async_trait]
        impl Tool for $name {
            fn name(&self) -> &'static str {
                $tool
            }
            fn description(&self) -> String {
                $doc.to_string()
            }
            fn parameters(&self) -> Value {
                serde_json::json!({
                    "type": "object",
                    "required": ["symbol"],
                    "properties": {
                        "symbol": {"type": "string", "description": "符号名"},
                        "limit": {"type": "integer", "description": "返回条数上限"},
                        "depth": {"type": "integer", "description": "遍历深度（impact 用，1-5）"}
                    }
                })
            }
            async fn execute(&self, args: Value) -> CsResult<String> {
                let a = symbol_args(&args, $tool)?;
                let label = a["symbol"].as_str().unwrap_or("").to_string();
                let out = self.engine.call($tool, a).await?;
                Ok(format!("[codegraph {}] {label}\n{out}", $tool))
            }
        }
    };
}

cg_symbol_tool!(
    CallersTool,
    "callers",
    "谁调用了这个符号（结构图反向边）。args: {symbol, limit?}"
);
cg_symbol_tool!(
    CalleesTool,
    "callees",
    "这个符号调用了谁（结构图正向边）。args: {symbol, limit?, depth?}"
);
cg_symbol_tool!(
    ImpactTool,
    "impact",
    "改这个符号的影响面（传递闭包 blast radius）。args: {symbol, depth?}"
);

/// explore：一次性返回相关符号源码 + 调用路径 + 影响面（开局看图用这个）。
pub struct ExploreTool {
    engine: Arc<CodegraphEngine>,
}

impl ExploreTool {
    pub fn new(engine: Arc<CodegraphEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait::async_trait]
impl Tool for ExploreTool {
    fn name(&self) -> &'static str {
        "explore"
    }

    fn description(&self) -> String {
        "结构图一把梭（开局看图/流程问题优先）：相关符号源码 + 调用路径 + 影响面一次返回。args: {query: 符号名/文件/自然语言}".into()
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "required": ["query"],
            "properties": {"query": {"type": "string", "description": "符号名、文件名或自然语言问题"}}
        })
    }

    async fn execute(&self, args: Value) -> CsResult<String> {
        let query = args
            .get("query")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| CsError::new(USER_INPUT, "explore 缺少 query 参数"))?
            .to_string();
        let out = self
            .engine
            .call("explore", serde_json::json!({"query": query}))
            .await?;
        Ok(format!("[codegraph explore] {query}\n{out}"))
    }
}

/// files：列出索引内文件结构（可 glob 过滤）。
pub struct FilesTool {
    engine: Arc<CodegraphEngine>,
}

impl FilesTool {
    pub fn new(engine: Arc<CodegraphEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait::async_trait]
impl Tool for FilesTool {
    fn name(&self) -> &'static str {
        "files"
    }

    fn description(&self) -> String {
        "列出已索引的文件结构（可按 glob 过滤）。args: {filter?: glob}".into()
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {"filter": {"type": "string", "description": "glob，如 src/**"}}
        })
    }

    async fn execute(&self, args: Value) -> CsResult<String> {
        let mut a = serde_json::Map::new();
        if let Some(f) = args.get("filter").and_then(Value::as_str) {
            a.insert("filter".into(), Value::String(f.to_string()));
        }
        let out = self.engine.call("files", Value::Object(a)).await?;
        Ok(format!("[codegraph files]\n{out}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lost_artifact_step_decides_retry_then_fail() {
        // P007 R1.6 回归：产物就绪直接 serve；缺失先重试，达上限后判失败而非裸 serve
        assert_eq!(lost_step(true, 0), LostStep::Proceed);
        assert_eq!(lost_step(false, 0), LostStep::Retry);
        assert_eq!(lost_step(false, LOST_ARTIFACT_RETRIES - 1), LostStep::Retry);
        assert_eq!(lost_step(false, LOST_ARTIFACT_RETRIES), LostStep::Fail);
    }

    #[tokio::test]
    async fn missing_binary_is_structured_error() {
        let dir = tempfile::tempdir().unwrap();
        let err = CodegraphEngine::start_with_bin(dir.path(), "/nonexistent/cg-test", false)
            .await
            .unwrap_err();
        assert_eq!(err.code, INDEX_NOT_AVAILABLE);
    }

    #[test]
    fn symbol_args_validation() {
        let err = symbol_args(&serde_json::json!({}), "callers").unwrap_err();
        assert_eq!(err.code, USER_INPUT);
        let ok = symbol_args(
            &serde_json::json!({"symbol": "main", "limit": 5, "depth": 2}),
            "callers",
        )
        .unwrap();
        assert_eq!(ok["symbol"], "main");
        assert_eq!(ok["limit"], 5);
        assert_eq!(ok["depth"], 2);
    }
}
