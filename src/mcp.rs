//! 最小 MCP over stdio 客户端（D011）——只为 codesleuth 自己服务：
//! spawn server → newline-delimited JSON-RPC（initialize 握手 / tools_list / tools_call）→ Drop 回收。
//! 顺序请求-响应（单会话单 agent），服务端主动通知一律忽略。
//! 锁用 tokio::sync::Mutex（guard 可跨 await 且 Send）。

use crate::errors::{CsError, CsResult, INDEX_NOT_AVAILABLE, INDEX_TIMEOUT};
use serde_json::{Value, json};
use std::io::ErrorKind;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(90);

/// P007 R2.10：MCP 单行响应长度上限（JSON-RPC 响应不可能合理超过此值）。
const MAX_LINE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub struct McpClient {
    child: Mutex<tokio::process::Child>,
    stdin: Mutex<tokio::process::ChildStdin>,
    reader: Mutex<BufReader<tokio::process::ChildStdout>>,
    next_id: AtomicU64,
    timeout: Duration,
}

/// 构造 JSON-RPC 请求行（纯函数，可测）。
pub fn build_request(id: u64, method: &str, params: &Value) -> String {
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string()
}

/// 从响应中取出 id（通知无 id → None）。
pub fn response_id(v: &Value) -> Option<u64> {
    v.get("id").and_then(Value::as_u64)
}

/// tools/call 响应 → 拼接 text 内容。
/// P007 R3.16：错误码语义归位——工具调用期失败不再是 INDEX_BUILD_FAILED（那是
/// 构建期错误码，误导排查方向）：isError/空内容 → INDEX_NOT_AVAILABLE；
/// 空内容（符号不存在/无结果）是合法查询结果而非故障，返回说明性文本零误差。
pub fn extract_tool_text(resp: &Value) -> CsResult<String> {
    let text = content_text(resp);
    if resp["result"]["isError"].as_bool() == Some(true) {
        return Err(CsError::new(
            INDEX_NOT_AVAILABLE,
            format!("codegraph 工具错误: {text}"),
        ));
    }
    if text.trim().is_empty() {
        // 空内容 = 查询无结果（符号不存在/索引未就绪）：诚实回显给模型自纠，
        // 不当错误抛（Err 会烧 no_progress 且误导为故障）
        return Ok("（codegraph 返回空内容——符号不存在或索引未就绪；换符号名或先 explore）".into());
    }
    Ok(text)
}

fn content_text(resp: &Value) -> String {
    resp["result"]["content"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

impl McpClient {
    pub async fn spawn(bin: &str, args: &[&str], cwd: &Path) -> CsResult<Self> {
        let mut cmd = tokio::process::Command::new(bin);
        cmd.args(args)
            .current_dir(cwd)
            .env("CODEGRAPH_TELEMETRY", "0")
            .env("DO_NOT_TRACK", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                CsError::new(INDEX_NOT_AVAILABLE, "codegraph CLI 未安装").with_hint(
                    "npm i -g @colbymchenry/codegraph，或 curl -fsSL https://raw.githubusercontent.com/colbymchenry/codegraph/main/install.sh | sh",
                )
            } else {
                CsError::new(INDEX_NOT_AVAILABLE, format!("codegraph 启动失败: {e}"))
            }
        })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| CsError::new(INDEX_NOT_AVAILABLE, "MCP server 无 stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| CsError::new(INDEX_NOT_AVAILABLE, "MCP server 无 stdout"))?;
        Ok(Self {
            child: Mutex::new(child),
            stdin: Mutex::new(stdin),
            reader: Mutex::new(BufReader::new(stdout)),
            next_id: AtomicU64::new(1),
            timeout: DEFAULT_TIMEOUT,
        })
    }

    /// initialize 握手 + initialized 通知。返回服务器响应（含自带 usage guidance，Phase 4 可用）。
    pub async fn initialize(&self) -> CsResult<Value> {
        let params = json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "codesleuth", "version": env!("CARGO_PKG_VERSION")}
        });
        let resp = self.request("initialize", &params).await?;
        self.notify("notifications/initialized").await?;
        Ok(resp)
    }

    // P007 R4.7：list_tools 已删——工具集在 cli.rs 静态注册，从不向 server 查询；
    // 初始化返回的 usage guidance 已在 initialize 处消费

    pub async fn call_tool(&self, name: &str, args: Value) -> CsResult<String> {
        let resp = self
            .request("tools/call", &json!({"name": name, "arguments": args}))
            .await?;
        extract_tool_text(&resp)
    }

    async fn write_line(&self, line: &str) -> CsResult<()> {
        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("MCP 写入失败: {e}")))?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("MCP 写入失败: {e}")))?;
        stdin
            .flush()
            .await
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("MCP flush 失败: {e}")))?;
        Ok(())
    }

    async fn request(&self, method: &str, params: &Value) -> CsResult<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.write_line(&build_request(id, method, params)).await?;

        let deadline = tokio::time::Instant::now() + self.timeout;
        loop {
            let mut buf = String::new();
            let n = tokio::time::timeout_at(deadline, async {
                self.reader.lock().await.read_line(&mut buf).await
            })
            .await
            .map_err(|_| {
                CsError::new(
                    INDEX_TIMEOUT,
                    format!("MCP {method} 超时（{}s）", self.timeout.as_secs()),
                )
            })?
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("MCP 读取失败: {e}")))?;
            if n == 0 {
                return Err(CsError::new(
                    INDEX_NOT_AVAILABLE,
                    "MCP server 提前关闭了连接",
                ));
            }
            // P007 R2.10：单行长度上限——无换行的超长输出会无限累积内存直到超时；
            // 超限即报错（server 输出格式失控属于结构故障，不该靠 90s 超时兑底）
            if buf.len() > MAX_LINE_BYTES {
                return Err(CsError::new(
                    INDEX_NOT_AVAILABLE,
                    format!(
                        "MCP 单行响应超过 {}MB 上限，server 输出异常",
                        MAX_LINE_BYTES / 1024 / 1024
                    ),
                ));
            }
            let Ok(v) = serde_json::from_str::<Value>(buf.trim()) else {
                continue; // 非法行跳过
            };
            if response_id(&v) == Some(id) {
                if let Some(err) = v.get("error") {
                    // P007 R3.16：请求级协议错误归 INDEX_NOT_AVAILABLE（非构建期）
                    return Err(CsError::new(
                        INDEX_NOT_AVAILABLE,
                        format!("MCP error: {err}"),
                    ));
                }
                return Ok(v);
            }
            // 服务端通知 / 其他响应：忽略
        }
    }

    async fn notify(&self, method: &str) -> CsResult<()> {
        self.write_line(&json!({"jsonrpc": "2.0", "method": method}).to_string())
            .await
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        // 生命周期回收（D011）：会话结束即回收 server 进程。
        // P007 R2.15：try_lock 失败静默跳过会泄漏孤儿进程且无任何痕迹——
        // 改 blocking_lock（Drop 同步上下文，持锁方在 await 点挂起时最多等
        // 到其完成；正常路径锁空闲无开销），泄漏窗口消除。
        // 注：blocking_lock 在异步 runtime 线程上 panic 风险仅在锁被永久持有时，
        // 当前 child 锁只在 Drop 内触碰，无此场景。
        let mut child = self.child.blocking_lock();
        let _ = child.start_kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_line_shape() {
        let line = build_request(7, "tools/call", &json!({"name": "x"}));
        let v: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["id"], 7);
        assert_eq!(v["method"], "tools/call");
        assert_eq!(v["params"]["name"], "x");
    }

    #[test]
    fn response_id_and_content_extraction() {
        let resp = json!({
            "jsonrpc": "2.0", "id": 3,
            "result": {"content": [{"type": "text", "text": "hello"}, {"type": "text", "text": "world"}], "isError": false}
        });
        assert_eq!(response_id(&resp), Some(3));
        assert_eq!(extract_tool_text(&resp).unwrap(), "hello\nworld");

        let err_resp = json!({
            "jsonrpc": "2.0", "id": 4,
            "result": {"content": [{"type": "text", "text": "boom"}], "isError": true}
        });
        assert!(extract_tool_text(&err_resp).is_err());

        let notif = json!({"jsonrpc": "2.0", "method": "notifications/initialized"});
        assert_eq!(response_id(&notif), None);
    }
}
