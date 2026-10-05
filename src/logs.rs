//! 会话日志落盘（P005 R7.4）：stderr 人读 + 文件归档双轨，同一过滤同一格式。
//! 每会话一文件 `~/.codesleuth/logs/<session_id>.log`，单代轮转（超限 → `.old`，与审计同款）。
//! --json 模式的 stdout 纪律不受影响——文件与 stderr 都是过程通道。

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// 单代轮转阈值（与审计 64MB 同款）。
pub const LOG_ROTATE_BYTES: u64 = 64 * 1024 * 1024;

/// 会话日志文件句柄（线程安全，供 tracing 的 MakeWriter 使用）。
#[derive(Clone)]
pub struct SessionLog {
    inner: Arc<Mutex<File>>,
    path: PathBuf,
}

impl SessionLog {
    /// 打开会话日志；已存在且超限则先轮转（覆盖旧 `.old`，单代保留）。
    pub fn open(path: &Path, max_bytes: u64) -> std::io::Result<Self> {
        if let Ok(meta) = std::fs::metadata(path)
            && meta.len() > max_bytes
        {
            let old = path.with_extension("log.old");
            // 轮转失败不阻断（退化为继续追加，warn 由调用方日志可见）
            let _ = std::fs::rename(path, &old);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(file)),
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Write for SessionLog {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .flush()
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for SessionLog {
    type Writer = SessionLog;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// P005 R7.4：超限轮转 → 旧内容进 `.old`，新文件继续写。
    #[test]
    fn open_rotates_oversized_log() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.log");
        std::fs::write(&path, vec![b'x'; 128]).unwrap();

        let mut log = SessionLog::open(&path, 64).unwrap();
        write!(log, "fresh").unwrap();

        let old = std::fs::read(dir.path().join("s.log.old")).unwrap();
        assert_eq!(old, vec![b'x'; 128], "旧内容应整体轮转到 .old");
        assert_eq!(std::fs::read(&path).unwrap(), b"fresh", "新文件只含新内容");
    }

    /// P005 R7.4：未超限不轮转，追加语义成立。
    #[test]
    fn open_keeps_small_log_and_appends() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.log");
        std::fs::write(&path, "keep").unwrap();

        let mut log = SessionLog::open(&path, 64).unwrap();
        write!(log, "+more").unwrap();

        assert!(!dir.path().join("s.log.old").exists());
        assert_eq!(std::fs::read(&path).unwrap(), b"keep+more");
    }

    /// P005 R7.4：session span 内的事件自动携带 session_id（结构化串线）。
    #[test]
    fn session_id_flows_into_log_line() {
        // 捕获型 writer（tracing-subscriber 自带 TestWriter 只打印不缓存，无法断言）
        #[derive(Clone, Default)]
        struct Capture(Arc<Mutex<Vec<u8>>>);
        impl std::io::Write for Capture {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Capture {
            type Writer = Capture;
            fn make_writer(&'a self) -> Self::Writer {
                self.clone()
            }
        }

        let writer = Capture::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(writer.clone())
            .with_ansi(false)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("session", session_id = "abc123");
            let _g = span.enter();
            tracing::warn!("degraded probe");
        });
        let out = String::from_utf8(writer.0.lock().unwrap().clone()).unwrap();
        // fmt 全格式对字符串字段加引号：session{session_id="abc123"}
        assert!(
            out.contains("session_id=\"abc123\""),
            "session_id 未串线: {out}"
        );
        assert!(out.contains("degraded probe"));
    }
}
