//! 仓库级索引引导锁（D014）：graph 引导（init / index --force）与向量构建共用
//! 一把跨进程文件锁 `<repo>/.codesleuth/boot.lock`，把两层「check-then-act」索引
//! 引导在进程间串行化。
//!
//! 语义（D014 用户拍板「输者死不降级」）：
//! - 锁空闲立即拿到 = [`BootLock::Won`]，调用方正常执行引导；
//! - 锁被占 → 轮询等待 → 拿到 = [`BootLock::Lost`]（对手刚完成引导）→ 调用方必须
//!   以 CS4016 INDEX_LOCKED 判负退出，禁止降级——残缺工具面的侦察照样烧 token、
//!   产出劣质报告还伪装成功，重跑成本 ≈ 0（索引已就绪）；
//! - 等待超过 timeout = 同码 CS4016 退出。
//!
//! 实现：`std::fs::File::try_lock`（flock，Rust 1.89 稳定）。进程崩溃由内核放锁，
//! 无陈旧锁问题；锁文件常驻 `.codesleuth/`（writeguard 豁免目录，D011 式工具元数据）。
//! 同步阻塞轮询只发生在锁竞争路径（上限 timeout），引导期短任务可接受。

use crate::errors::{CsError, CsResult, INDEX_LOCKED, REPO_NOT_READABLE};
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::time::{Duration, Instant};

/// 轮询间隔。
const POLL_INTERVAL: Duration = Duration::from_millis(500);
/// 等锁上限，与 run_cli 的 INDEX_TIMEOUT（300s）对齐。
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

/// acquire 的三态结果。
#[derive(Debug)]
pub enum BootLock {
    /// 锁空闲直接拿到：正常执行引导。
    Won(BootLockGuard),
    /// 等待后拿到：对手刚完成引导，调用方必须判负退出（CS4016），不得降级。
    Lost,
}

/// 引导锁守卫：持有期间独占 `<repo>/.codesleuth/boot.lock`。
///
/// 引导段结束即释放（Drop unlock）——锁只罩 init/build，不跨 LLM 调用、不罩 serve。
#[derive(Debug)]
pub struct BootLockGuard {
    // 保存文件句柄：Drop 时 unlock；句柄消失锁随之释放（双保险）。
    file: File,
}

impl Drop for BootLockGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

/// 获取仓库引导锁。
pub fn acquire(repo_root: &Path, timeout: Duration) -> CsResult<BootLock> {
    let dir = repo_root.join(".codesleuth");
    std::fs::create_dir_all(&dir).map_err(|e| {
        CsError::new(
            REPO_NOT_READABLE,
            format!("无法创建引导锁目录 {}: {e}", dir.display()),
        )
    })?;
    let path = dir.join("boot.lock");
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| {
            CsError::new(
                REPO_NOT_READABLE,
                format!("无法打开引导锁 {}: {e}", path.display()),
            )
        })?;

    // 首次尝试：立即拿到 = 赢家，无竞争。
    match file.try_lock() {
        Ok(()) => return Ok(BootLock::Won(BootLockGuard { file })),
        Err(std::fs::TryLockError::Error(e)) => {
            return Err(CsError::new(INDEX_LOCKED, format!("引导锁获取失败: {e}")));
        }
        Err(std::fs::TryLockError::WouldBlock) => {}
    }

    // 竞争路径：轮询等待。拿到即判负——对手刚完成引导，本进程的引导已无必要，
    // 任务带着残缺工具面继续只会烧 token，故交由调用方以 INDEX_LOCKED 退出。
    let deadline = Instant::now() + timeout;
    loop {
        if Instant::now() >= deadline {
            return Err(CsError::new(
                INDEX_LOCKED,
                format!("等待索引引导锁超时（{}s）", timeout.as_secs()),
            )
            .with_hint("另一进程长时间持有引导锁（可能正在建大仓索引）；确认其结束后重跑本命令"));
        }
        std::thread::sleep(POLL_INTERVAL);
        match file.try_lock() {
            Ok(()) => return Ok(BootLock::Lost),
            Err(std::fs::TryLockError::Error(e)) => {
                return Err(CsError::new(INDEX_LOCKED, format!("引导锁获取失败: {e}")));
            }
            Err(std::fs::TryLockError::WouldBlock) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 空闲锁首次获取必 Won；释放后可重新 Won（无竞争 = 赢家语义）。
    #[test]
    fn fresh_lock_is_won_and_reacquirable_after_release() {
        let dir = tempfile::tempdir().unwrap();
        {
            let BootLock::Won(_g) = acquire(dir.path(), DEFAULT_TIMEOUT).unwrap() else {
                panic!("fresh lock must be won immediately");
            };
        } // guard drop → unlock
        match acquire(dir.path(), DEFAULT_TIMEOUT).unwrap() {
            BootLock::Won(_) => {}
            BootLock::Lost => panic!("no competitor: must be Won"),
        }
    }

    /// 持锁期间第二个获取者必须等待；锁释放后它拿到 = Lost（判负语义）。
    #[test]
    fn second_acquirer_waits_then_reports_lost() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let BootLock::Won(g1) = acquire(dir.path(), DEFAULT_TIMEOUT).unwrap() else {
            panic!("first must win");
        };
        let h = std::thread::spawn(move || match acquire(&root, DEFAULT_TIMEOUT).unwrap() {
            BootLock::Lost => true,
            BootLock::Won(_) => false,
        });
        // 等待者首试失败（锁被占）后进入 500ms 轮询；600ms 后放锁，
        // 其下一次轮询（~1s）拿到 → Lost。轮询间隔 500ms 给了足够裕度。
        std::thread::sleep(Duration::from_millis(600));
        drop(g1);
        assert!(h.join().unwrap(), "waiter must report Lost, not Won");
    }

    /// 等锁超时 → CS4016 结构化错误（不 panic、不静默）。
    #[test]
    fn wait_timeout_is_index_locked() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let BootLock::Won(g1) = acquire(dir.path(), DEFAULT_TIMEOUT).unwrap() else {
            panic!("first must win");
        };
        let h = std::thread::spawn(move || {
            let err = acquire(&root, Duration::from_millis(300)).unwrap_err();
            assert_eq!(err.code, INDEX_LOCKED);
        });
        // 超时 300ms < 轮询间隔 500ms：等待者必然在第一次轮询前撞死线。
        std::thread::sleep(Duration::from_millis(800));
        drop(g1);
        h.join().unwrap();
    }
}
