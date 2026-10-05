//! 路径围栏（D009 只读边界第二层）：所有工具的目标路径必须解析到 repo root 之内。
//! symlink 逃逸一律拒绝；越界 = CS3003 结构化报错。

use crate::errors::{CsError, CsResult, FENCE_DENIED, REPO_NOT_FOUND};
use std::path::{Path, PathBuf};

pub struct Fence {
    root: PathBuf,
}

impl Fence {
    pub fn new(root: &Path) -> CsResult<Self> {
        let root = dunce::canonicalize(root).map_err(|e| {
            CsError::new(
                REPO_NOT_FOUND,
                format!("仓库根目录不可达: {}", root.display()),
            )
            .with_source(format!("canonicalize: {e}"))
        })?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 解析相对（或绝对）路径到围栏内的绝对路径。
    /// 防御纵深：①词汇层预检（.. / 绝对路径越界直接拒，不泄露目标库外的存在性）
    /// ②canonicalize 解 symlink 后复检。
    pub fn resolve(&self, rel: &str) -> CsResult<PathBuf> {
        let candidate = self.root.join(rel);
        let candidate_abs = if candidate.is_absolute() {
            candidate.clone()
        } else {
            std::env::current_dir().unwrap_or_default().join(&candidate)
        };
        if !starts_with_root(&normalize(&candidate_abs), &self.root) {
            return Err(
                CsError::new(FENCE_DENIED, format!("路径逃逸出仓库围栏: {rel}"))
                    .with_hint("只允许访问仓库内的文件"),
            );
        }
        let resolved = dunce::canonicalize(&candidate).map_err(|e| {
            CsError::new(REPO_NOT_FOUND, format!("路径不存在: {rel}"))
                .with_hint("用 find_files 先定位真实路径")
                .with_source(format!("canonicalize: {e}"))
        })?;
        if !starts_with_root(&resolved, &self.root) {
            return Err(
                CsError::new(FENCE_DENIED, format!("路径逃逸出仓库围栏: {rel}"))
                    .with_hint("只允许访问仓库内的文件"),
            );
        }
        Ok(resolved)
    }
}

/// 词汇层路径归一（不解 symlink）：折叠 . 与 ..。
fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// 围栏前缀判定（P005 R6.1）：Windows 文件系统大小写不敏感，逐组件忽略 ASCII 大小写比较，
/// 其余平台严格比较——避免合法路径在 Windows 被大小写差异误杀。
fn starts_with_root(p: &Path, root: &Path) -> bool {
    p.components().zip(root.components()).all(|(a, b)| {
        #[cfg(windows)]
        {
            a.as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
        }
        #[cfg(not(windows))]
        {
            a.as_os_str() == b.as_os_str()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_inside_and_resolves() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/a.rs"), "fn a() {}\n").unwrap();
        let fence = Fence::new(dir.path()).unwrap();
        let p = fence.resolve("src/a.rs").unwrap();
        assert!(p.ends_with("src/a.rs"));
    }

    #[test]
    fn symlink_escape_denied() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let secret = outside.path().join("secret.txt");
        std::fs::write(&secret, "top secret").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&secret, dir.path().join("leak.txt")).unwrap();
        let fence = Fence::new(dir.path()).unwrap();
        let err = fence.resolve("leak.txt").unwrap_err();
        assert_eq!(err.code, FENCE_DENIED);
        assert_eq!(err.exit_code(), 4);
    }

    #[test]
    fn absolute_outside_denied() {
        let dir = tempfile::tempdir().unwrap();
        let fence = Fence::new(dir.path()).unwrap();
        let err = fence.resolve("/etc/hosts").unwrap_err();
        assert_eq!(err.code, FENCE_DENIED);
    }

    #[test]
    fn nonexistent_is_repo_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let fence = Fence::new(dir.path()).unwrap();
        let err = fence.resolve("nope.rs").unwrap_err();
        assert_eq!(err.code, REPO_NOT_FOUND);
    }
}
