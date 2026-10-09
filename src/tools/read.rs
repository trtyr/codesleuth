//! 强读工具（topics/read-tool-design.md MVP 六特性）：
//! 行号+行哈希锚点 / offset-limit 分页 / 二进制探测 / 编码消毒 / 诚实反馈 / 结构化输出头。

use crate::errors::{CsError, CsResult, REPO_NOT_READABLE, USER_INPUT};
use crate::fence::Fence;
use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;
use sha2::Digest;
use std::sync::Arc;

const DEFAULT_LIMIT: usize = 200;
const MAX_LIMIT: usize = 2000;

pub struct ReadTool {
    fence: Arc<Fence>,
}

impl ReadTool {
    pub fn new(fence: Arc<Fence>) -> Self {
        Self { fence }
    }
}

/// 行哈希锚点（12-bit，3 位 hex）：行内容一变锚点大概率失效——
/// P007 R3.26 口径修正：12-bit 空间仅 4096 种值，不同行碰撞概率 ≈1/4096，
/// 单个 200 行窗口内碰撞概率约百分之几；锚点是提示性校验，非强保证，
/// 引用准确性仍以行内容为准。
fn line_hash(line: &str) -> u16 {
    let d = sha2::Sha256::digest(line.as_bytes());
    (((d[0] as u16) << 4) | ((d[1] as u16) >> 4)) & 0x0FFF
}

/// 二进制启发：NUL 字节或控制字符占比 > 10%（UTF-8 高位字节不算，中文文件不受影响）。
fn looks_binary(head: &[u8]) -> bool {
    if head.contains(&0) {
        return true;
    }
    let n = head.len().max(1);
    let weird = head
        .iter()
        .filter(|&&b| b < 9 || (b > 13 && b < 32))
        .count();
    weird * 100 / n > 10
}

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &'static str {
        "read"
    }

    fn description(&self) -> String {
        "读取仓库内文件（唯一的事实来源）。返回行号+锚点。args: {path: 仓库内相对路径, offset?: 起始行(1-based), limit?: 行数(默认200,上限2000)}".into()
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": {"type": "string", "description": "仓库内相对路径"},
                "offset": {"type": "integer", "description": "起始行（1-based，默认 1）"},
                "limit": {"type": "integer", "description": "行数（默认 200，上限 2000）"}
            }
        })
    }

    async fn execute(&self, args: Value) -> CsResult<String> {
        let path_arg = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CsError::new(USER_INPUT, "read 缺少 path 参数")
                    .with_hint(r#"{"path": "src/main.rs"}"#)
            })?
            .to_string();
        // P007 R3.8：非法/越限参数不再纯静默归一——实际生效值在输出头本就可见
        // （显示 {offset}-{end} 行），此处补一行归一说明，诚实可纠
        let raw_offset = args.get("offset").and_then(Value::as_u64);
        let raw_limit = args.get("limit").and_then(Value::as_u64);
        let offset = raw_offset.unwrap_or(1).max(1) as usize;
        let limit = raw_limit
            .unwrap_or(DEFAULT_LIMIT as u64)
            .clamp(1, MAX_LIMIT as u64) as usize;
        let mut normalize_note = String::new();
        if raw_offset.is_none() && args.get("offset").is_some() {
            normalize_note.push_str("；offset 非法已归一为 1");
        }
        if let Some(l) = raw_limit
            && (l < 1 || l > MAX_LIMIT as u64)
        {
            normalize_note.push_str(&format!("；limit {l} 越限已钳到 {limit}"));
        }

        let resolved = self.fence.resolve(&path_arg)?;
        if !resolved.is_file() {
            return Err(CsError::new(
                REPO_NOT_READABLE,
                format!("{path_arg} 不是普通文件（可能是目录）"),
            ));
        }
        let bytes = std::fs::read(&resolved)
            .map_err(|e| CsError::new(REPO_NOT_READABLE, format!("读取失败: {e}")))?;

        // 特性 3：二进制探测——不倾倒垃圾
        let head_len = bytes.len().min(8192);
        if looks_binary(&bytes[..head_len]) {
            return Ok(format!(
                "[read] {path_arg}：二进制文件（{} bytes），不倾倒内容。可用工具查看其结构，而非读取原文。",
                bytes.len()
            ));
        }

        // 特性 4：编码消毒（lossy UTF-8 + 声明）
        let text = String::from_utf8_lossy(&bytes);
        let lossy = text.contains('\u{FFFD}');
        let total = text.lines().count();

        // 特性 5：诚实反馈
        if total == 0 {
            return Ok(format!("[read] {path_arg}：空文件（0 行）。"));
        }
        if offset > total {
            return Ok(format!(
                "[read] {path_arg}：offset {offset} 超出范围，文件共 {total} 行。"
            ));
        }

        let end = (offset.saturating_sub(1) + limit).min(total);
        let mut out = format!(
            "[read] {path_arg} · 共 {total} 行 · 显示 {offset}-{end} 行{}{normalize_note}\n",
            if lossy {
                " · 含非 UTF-8 字节（已用 U+FFFD 替换）"
            } else {
                ""
            }
        );
        for (idx, line) in text
            .lines()
            .enumerate()
            .skip(offset - 1)
            .take(end - offset + 1)
        {
            out.push_str(&format!("{}:{:03x}|{}\n", idx + 1, line_hash(line), line));
        }
        // 特性 2：截断必须告知总行数与续读 offset
        if end < total {
            out.push_str(&format!(
                "…未完：还有 {} 行。续读 offset={}\n",
                total - end,
                end + 1
            ));
        }
        Ok(out)
    }
}

/// P007 R1.2：判别 read 输出是否真实交付了文件行内容。
/// 二进制不倾倒 / 空文件 / offset 越界三种诚实声明返回 Ok 但未读出任何行——
/// 这些路径不得进入证据库（「必须真实读过」门禁，R1.2）。判别特征与上方
/// 成功分支的输出头（`· 共 N 行 · 显示 x-y 行`）同文件维护，防漂移。
pub fn delivered_lines(output: &str) -> bool {
    output.contains("· 共 ") && output.contains("行 · 显示 ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn delivered_lines_discriminates_honest_empty_reads() {
        // P007 R1.2 回归：三种诚实声明（二进制/空文件/越界）均未读出行内容，
        // 不得入证据库；只有成功分支的头格式判真
        assert!(!delivered_lines(
            "[read] a.bin：二进制文件（123 bytes），不倾倒内容。可用工具查看其结构，而非读取原文。"
        ));
        assert!(!delivered_lines("[read] empty.md：空文件（0 行）。"));
        assert!(!delivered_lines(
            "[read] big.rs：offset 99 超出范围，文件共 5 行。"
        ));
        assert!(delivered_lines(
            "[read] big.rs · 共 500 行 · 显示 2-4 行\n2:abc|line 2\n"
        ));
    }

    fn tool_for(dir: &Path) -> ReadTool {
        ReadTool::new(Arc::new(Fence::new(dir).unwrap()))
    }

    #[tokio::test]
    async fn anchors_and_pagination_with_next_offset() {
        let dir = tempfile::tempdir().unwrap();
        let body: String = (1..=500).map(|i| format!("line {i}\n")).collect();
        std::fs::write(dir.path().join("big.rs"), &body).unwrap();
        let out = tool_for(dir.path())
            .execute(serde_json::json!({"path": "big.rs", "offset": 2, "limit": 3}))
            .await
            .unwrap();
        assert!(out.contains("共 500 行 · 显示 2-4 行"));
        assert!(out.contains("2:"));
        assert!(out.contains("4:"));
        assert!(!out.contains("5:"));
        assert!(out.contains("续读 offset=5"));
    }

    #[tokio::test]
    async fn binary_refuses_to_dump() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("blob.bin"), b"\x00\x01\x02binary\x00").unwrap();
        let out = tool_for(dir.path())
            .execute(serde_json::json!({"path": "blob.bin"}))
            .await
            .unwrap();
        assert!(out.contains("二进制文件"));
        assert!(!out.contains("0:"));
    }

    #[tokio::test]
    async fn lossy_utf8_declared() {
        let dir = tempfile::tempdir().unwrap();
        let mut bytes = "中文行\n".as_bytes().to_vec();
        bytes.push(0xFF); // 非 UTF-8 字节
        std::fs::write(dir.path().join("mixed.txt"), &bytes).unwrap();
        let out = tool_for(dir.path())
            .execute(serde_json::json!({"path": "mixed.txt"}))
            .await
            .unwrap();
        assert!(out.contains("U+FFFD"));
        assert!(out.contains("中文行"));
    }

    #[tokio::test]
    async fn empty_and_out_of_range_are_honest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("e.txt"), "").unwrap();
        std::fs::write(dir.path().join("s.txt"), "one\n").unwrap();
        let t = tool_for(dir.path());
        let out = t
            .execute(serde_json::json!({"path": "e.txt"}))
            .await
            .unwrap();
        assert!(out.contains("空文件"));
        let out = t
            .execute(serde_json::json!({"path": "s.txt", "offset": 99}))
            .await
            .unwrap();
        assert!(out.contains("超出范围"));
        assert!(out.contains("共 1 行"));
    }

    #[tokio::test]
    async fn missing_path_arg_is_tool_error() {
        let dir = tempfile::tempdir().unwrap();
        let err = tool_for(dir.path())
            .execute(serde_json::json!({}))
            .await
            .unwrap_err();
        assert_eq!(err.code, USER_INPUT);
    }
}
