//! 审计 JSONL（D003.2）+ 证据账本（D008）。
//! 审计文件 = 全量原文恢复源（D008 recall 钻取的落点）；账本 = 跨压缩存活的结构化发现。

use crate::errors::{CsError, CsResult, INTERNAL};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 会话结束后追加一行宿主级事件（如 write_check）：seq = 现有最大 seq + 1，保持单调不重不漏。
pub fn append_line(path: &Path, kind: &str, payload: &serde_json::Value) -> CsResult<u64> {
    let max_seq = std::fs::read_to_string(path)
        .map_err(|e| CsError::new(INTERNAL, format!("读审计失败 {path:?}: {e}")))?
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter_map(|v| v.get("seq").and_then(serde_json::Value::as_u64))
        .max()
        .unwrap_or(0);
    let seq = max_seq + 1;
    // 与 record 同规：payload 先并入、顶层后写（顶层胜出，防键冲突）
    let mut row = BTreeMap::new();
    if let serde_json::Value::Object(m) = payload {
        for (k, v) in m {
            row.insert(k.clone(), v.clone());
        }
    }
    row.insert("seq".into(), serde_json::json!(seq));
    row.insert("ts_ms".into(), serde_json::json!(now_ms()));
    row.insert("kind".into(), serde_json::json!(kind));
    let mut f = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|e| CsError::new(INTERNAL, format!("打开审计失败 {path:?}: {e}")))?;
    writeln!(
        f,
        "{}",
        serde_json::Value::Object(row.into_iter().collect())
    )
    .map_err(|e| CsError::new(INTERNAL, format!("写审计失败: {e}")))?;
    Ok(seq)
}

/// 会话 ID：纳秒时间 + pid 的 hex。
pub fn new_session_id() -> String {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}-{:x}", t, std::process::id())
}

/// 审计 JSONL：每行 = { seq, ts_ms, kind, ...payload }，按 seq 单调。
pub struct Audit {
    path: PathBuf,
    file: Mutex<File>,
    seq: AtomicU64,
}

impl Audit {
    pub fn create(state_dir: &Path, session_id: &str) -> CsResult<Self> {
        let dir = state_dir.join("audit");
        std::fs::create_dir_all(&dir).map_err(|e| {
            CsError::new(INTERNAL, format!("创建审计目录失败: {e}"))
                .with_hint(format!("状态目录: {}", dir.display()))
        })?;
        let path = dir.join(format!("{session_id}.jsonl"));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| CsError::new(INTERNAL, format!("打开审计文件失败: {e}")))?;
        Ok(Self {
            path,
            file: Mutex::new(file),
            seq: AtomicU64::new(0),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 当前最大 seq（压缩时标记审计范围用）。
    pub fn last_seq(&self) -> u64 {
        self.seq.load(Ordering::SeqCst)
    }

    /// 读回 seq ∈ [from, to] 的审计原文（D008 recall 钻取：按 seq 升序，不重不漏）。
    pub fn read_range(&self, from: u64, to: u64) -> CsResult<Vec<(u64, serde_json::Value)>> {
        let content = std::fs::read_to_string(&self.path)
            .map_err(|e| CsError::new(INTERNAL, format!("审计读取失败: {e}")))?;
        let mut out: Vec<(u64, serde_json::Value)> = Vec::new();
        for line in content.lines() {
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            let s = v
                .get("seq")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            if s >= from && s <= to {
                out.push((s, v));
            }
        }
        out.sort_by_key(|(s, _)| *s);
        Ok(out)
    }

    /// 追加一行，返回全局序号（账本引用它回溯原文）。
    pub fn record(&self, kind: &str, payload: &serde_json::Value) -> CsResult<u64> {
        // 先合入 payload，再写入顶层控制字段——payload 键不得覆盖 seq/ts_ms/kind
        let mut line: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        if let Some(map) = payload.as_object() {
            for (k, v) in map {
                line.insert(k.clone(), v.clone());
            }
        }
        let n = self.seq.fetch_add(1, Ordering::SeqCst) + 1;
        line.insert("seq".into(), serde_json::json!(n));
        line.insert("ts_ms".into(), serde_json::json!(now_ms()));
        line.insert("kind".into(), serde_json::json!(kind));
        let mut f = self.file.lock().expect("audit lock");
        writeln!(
            f,
            "{}",
            serde_json::to_string(&line)
                .map_err(|e| CsError::new(INTERNAL, format!("审计序列化失败: {e}")))?
        )
        .map_err(|e| CsError::new(INTERNAL, format!("审计写入失败: {e}")))?;
        f.flush().ok();
        Ok(n)
    }
}

// ---------------- 证据账本 ----------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LedgerEntry {
    /// 12-hex 短 ID（审计/上下文引用锚点）。
    pub id: String,
    /// finding | dead_end | lead
    pub kind: String,
    pub text: String,
    /// 证据引用（file:line 等）。
    pub refs: Vec<String>,
    /// 产生该条目的审计行 seq。
    pub created_seq: u64,
}

/// 证据账本：跨压缩存活的持久结构（D008）。
pub struct Ledger {
    path: PathBuf,
    entries: Vec<LedgerEntry>,
}

impl Ledger {
    pub fn load(path: PathBuf) -> Self {
        let entries = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice::<Vec<LedgerEntry>>(&b).ok())
            .unwrap_or_default();
        Self { path, entries }
    }

    pub fn add(
        &mut self,
        kind: &str,
        text: &str,
        refs: Vec<String>,
        created_seq: u64,
    ) -> &LedgerEntry {
        let mut h = sha2::Sha256::new();
        h.update(text.as_bytes());
        h.update(created_seq.to_le_bytes());
        h.update((self.entries.len() as u64).to_le_bytes());
        let digest = h.finalize();
        let id: String = digest[..6].iter().map(|b| format!("{b:02x}")).collect();
        self.entries.push(LedgerEntry {
            id,
            kind: kind.to_string(),
            text: text.to_string(),
            refs,
            created_seq,
        });
        self.entries.last().expect("just pushed")
    }

    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    pub fn save(&self) -> CsResult<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CsError::new(INTERNAL, format!("创建账本目录失败: {e}")))?;
        }
        let bytes = serde_json::to_vec_pretty(&self.entries)
            .map_err(|e| CsError::new(INTERNAL, format!("账本序列化失败: {e}")))?;
        std::fs::write(&self.path, bytes)
            .map_err(|e| CsError::new(INTERNAL, format!("账本写入失败: {e}")))?;
        Ok(())
    }

    /// 注入上下文的紧凑渲染（D008 结构段之一）。
    pub fn render(&self) -> String {
        if self.entries.is_empty() {
            return "证据账本：（空）".to_string();
        }
        let mut out = format!("证据账本（{} 条）：\n", self.entries.len());
        for e in &self.entries {
            let refs = if e.refs.is_empty() {
                String::new()
            } else {
                format!(" [{}]", e.refs.join(", "))
            };
            out.push_str(&format!("- #{} [{}] {}{}\n", e.id, e.kind, e.text, refs));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_records_monotonic_lines() {
        let dir = tempfile::tempdir().unwrap();
        let audit = Audit::create(dir.path(), "s1").unwrap();
        let a = audit
            .record("llm", &serde_json::json!({"model": "m", "total_tokens": 7}))
            .unwrap();
        let b = audit
            .record("tool", &serde_json::json!({"name": "echo"}))
            .unwrap();
        assert_eq!(a, 1);
        assert_eq!(b, 2);
        let content = std::fs::read_to_string(audit.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(first["kind"], "llm");
        assert_eq!(first["total_tokens"], 7);
        assert!(first["ts_ms"].as_u64().unwrap() > 0);
    }

    #[test]
    fn ledger_roundtrip_and_render() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ledger.json");
        let mut ledger = Ledger::load(path.clone());
        let e = ledger.add(
            "finding",
            "重试逻辑在 retry.rs",
            vec!["src/retry.rs:10-40".into()],
            3,
        );
        assert_eq!(e.id.len(), 12);
        ledger.save().unwrap();
        let loaded = Ledger::load(path);
        assert_eq!(loaded.entries().len(), 1);
        assert_eq!(loaded.entries()[0].kind, "finding");
        let rendered = loaded.render();
        assert!(rendered.contains("重试逻辑在 retry.rs"));
        assert!(rendered.contains(&format!("#{}", loaded.entries()[0].id)));
    }
}
