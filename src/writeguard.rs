//! 零写入自证（D1 修复，retest-r1.md）：聚合指纹在活仓库（并发开发 / 构建产物）必然误报。
//! 升级为 per-file manifest + 写面归因：codesleuth 在目标仓内的可写面为空集，
//! `.codegraph/**` 是 D011 豁免的工具元数据（快照期直接跳过）；
//! 其余任何差异 = 「不可归因变更（并发/外部）」，如实上报——不冒领，也不瞒报。

use crate::errors::{CsError, CsResult, REPO_NOT_READABLE};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

const SKIP_DIRS: &[&str] = &[
    ".git",
    ".codegraph",
    ".codesleuth",
    "target",
    "node_modules",
    "dist",
    "__pycache__",
    ".venv",
];

/// 快照全文哈希的文件大小上限：超过则指纹化（大小 + 首 64KB）。
/// 非代码仓（知识库/素材库）动辄上 GB 资产，全量哈希会把会话开场拖到分钟级。
const SNAPSHOT_FULL_HASH_MAX: u64 = 1024 * 1024;

/// 相对路径 → sha256。BTreeMap 保证 diff 输出顺序稳定。
pub type Manifest = BTreeMap<String, [u8; 32]>;

pub fn snapshot(root: &Path) -> CsResult<Manifest> {
    if !root.is_dir() {
        return Err(CsError::new(
            REPO_NOT_READABLE,
            format!("快照根不存在: {}", root.display()),
        ));
    }
    let mut out = Manifest::new();
    walk(root, root, &mut out)?;
    Ok(out)
}

fn walk(dir: &Path, root: &Path, out: &mut Manifest) -> CsResult<()> {
    let rd = std::fs::read_dir(dir).map_err(|e| {
        CsError::new(
            REPO_NOT_READABLE,
            format!("快照失败 {}: {e}", dir.display()),
        )
    })?;
    let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let name = e.file_name().to_string_lossy().to_string();
        let p = e.path();
        // P005 R4.3：symlink 一律跳过——symlink_metadata 不跟随：目录链接防成环递归与越界遍历，
        // 文件链接防哈希读出仓外；悬空链接同样读不到，一并跳过（如实不完整，不伪造）
        let Ok(ft) = std::fs::symlink_metadata(&p) else {
            continue;
        };
        if ft.file_type().is_symlink() {
            tracing::debug!("快照跳过符号链接 {p:?}");
            continue;
        }
        if ft.is_dir() {
            if SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            walk(&p, root, out)?;
        } else if ft.is_file() {
            let rel = p
                .strip_prefix(root)
                // P005 R5.3：唯一生产 expect 改错误传播——walk 逻辑上不可达，但形式上不崩溃
                .map_err(|e| CsError::new(REPO_NOT_READABLE, format!("快照路径越界 {p:?}: {e}")))?
                .to_string_lossy()
                .replace('\\', "/");
            let mut h = Sha256::new();
            let mut f = std::fs::File::open(&p)
                .map_err(|e| CsError::new(REPO_NOT_READABLE, format!("读取失败 {p:?}: {e}")))?;
            // 大文件指纹化（QA 马拉松 FINDING-008）：>1MB 资产（图片/字体/媒体）只摘
            // 「大小 + 首 64KB」指纹，不全量哈希——知识库/素材库快照从分钟级降到秒级；
            // 改动检测灵敏度对小文本文件（代码/笔记）毫发无损。
            let size = ft.len();
            if size > SNAPSHOT_FULL_HASH_MAX {
                use std::io::Read;
                h.update(size.to_le_bytes());
                let mut head = vec![0u8; 64 * 1024];
                let n = match f.read(&mut head) {
                    Ok(n) => n,
                    Err(e) => {
                        // >1MB 文件短读/读失败属异常（P004 T3）：灵敏度降级必须可见
                        tracing::warn!("指纹首 64KB 读取失败 {p:?}: {e}（按 0 字节计）");
                        0
                    }
                };
                h.update(&head[..n]);
            } else {
                std::io::copy(&mut f, &mut h)
                    .map_err(|e| CsError::new(REPO_NOT_READABLE, format!("哈希失败 {p:?}: {e}")))?;
            }
            out.insert(rel, h.finalize().into());
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChangeKind {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Change {
    pub path: String,
    pub kind: ChangeKind,
}

pub fn diff(before: &Manifest, after: &Manifest) -> Vec<Change> {
    let mut out = Vec::new();
    for (p, h) in after {
        match before.get(p) {
            None => out.push(Change {
                path: p.clone(),
                kind: ChangeKind::Added,
            }),
            Some(b) if b != h => out.push(Change {
                path: p.clone(),
                kind: ChangeKind::Modified,
            }),
            _ => {}
        }
    }
    for p in before.keys() {
        if !after.contains_key(p) {
            out.push(Change {
                path: p.clone(),
                kind: ChangeKind::Removed,
            });
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_change_yields_empty_diff() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn a() {}\n").unwrap();
        let m1 = snapshot(dir.path()).unwrap();
        let m2 = snapshot(dir.path()).unwrap();
        assert!(diff(&m1, &m2).is_empty());
        assert_eq!(m1.len(), 1);
    }

    #[test]
    fn modify_add_remove_detected() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "v1").unwrap();
        let m1 = snapshot(dir.path()).unwrap();
        std::fs::write(dir.path().join("a.rs"), "v2").unwrap();
        std::fs::write(dir.path().join("b.rs"), "new").unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("sub/c.rs"), "c").unwrap();
        let m2 = snapshot(dir.path()).unwrap();
        let changes = diff(&m1, &m2);
        assert_eq!(changes.len(), 3, "{changes:?}");
        assert!(changes.contains(&Change {
            path: "a.rs".into(),
            kind: ChangeKind::Modified
        }));
        assert!(changes.contains(&Change {
            path: "b.rs".into(),
            kind: ChangeKind::Added
        }));
        assert!(changes.contains(&Change {
            path: "sub/c.rs".into(),
            kind: ChangeKind::Added
        }));
        // 反向：删除
        std::fs::remove_file(dir.path().join("a.rs")).unwrap();
        let m3 = snapshot(dir.path()).unwrap();
        let rm = diff(&m2, &m3);
        assert_eq!(
            rm,
            vec![Change {
                path: "a.rs".into(),
                kind: ChangeKind::Removed
            }]
        );
    }

    #[test]
    #[cfg(unix)]
    fn symlink_entries_are_skipped_no_hang() {
        use std::os::unix::fs::symlink;
        // P005 R4.3：符号链接（含目录成环）一律跳过——不递归、不进 manifest、不哈希出仓内容
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("f.txt"), "x").unwrap();
        symlink(&sub, dir.path().join("loop")).unwrap();
        symlink(sub.join("f.txt"), dir.path().join("alias.txt")).unwrap();
        let m = snapshot(dir.path()).unwrap();
        assert!(m.contains_key("sub/f.txt"));
        assert!(!m.keys().any(|k| k.starts_with("loop")));
        assert!(!m.contains_key("alias.txt"));
    }

    #[test]
    fn dot_codegraph_is_excluded_from_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let cg = dir.path().join(".codegraph");
        std::fs::create_dir(&cg).unwrap();
        std::fs::write(cg.join("index.db"), "v1").unwrap();
        std::fs::write(dir.path().join("src.rs"), "s").unwrap();
        let m1 = snapshot(dir.path()).unwrap();
        assert_eq!(m1.len(), 1, ".codegraph 不应入册: {m1:?}");
        std::fs::write(cg.join("index.db"), "v2-grew").unwrap(); // codegraph 索引增长
        let m2 = snapshot(dir.path()).unwrap();
        assert!(diff(&m1, &m2).is_empty(), "D011 豁免面变更必须归零");
    }

    #[test]
    fn write_check_appends_with_monotonic_seq() {
        use crate::audit::{Audit, append_line, new_session_id};
        let dir = tempfile::tempdir().unwrap();
        let a = Audit::create(dir.path(), &new_session_id()).unwrap();
        a.record("tool_call", &serde_json::json!({"name": "x"}))
            .unwrap();
        a.record("tool_result", &serde_json::json!({"seq": 1}))
            .unwrap(); // 键冲突陷阱：顶层 seq 必须胜出
        let s = append_line(
            a.path(),
            "write_check",
            &serde_json::json!({"files_snapshotted": 7, "unattributed_changes": 0}),
        )
        .unwrap();
        assert_eq!(s, 3);
        let rows: Vec<serde_json::Value> = std::fs::read_to_string(a.path())
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        let seqs: Vec<u64> = rows.iter().filter_map(|v| v["seq"].as_u64()).collect();
        assert_eq!(seqs, vec![1, 2, 3], "seq 必须单调不重");
        assert_eq!(
            rows[1]["seq"].as_u64(),
            Some(2),
            "payload 的 seq 键不得覆盖顶层"
        );
        assert_eq!(rows[2]["kind"], "write_check");
        assert_eq!(rows[2]["files_snapshotted"].as_u64(), Some(7));
    }
}
