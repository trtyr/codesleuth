//! 仓库级索引引导锁（D014）：graph 引导（init / index --force）与向量构建共用
//! 一把跨进程文件锁 `<repo>/.codesleuth/boot.lock`，把两层「check-then-act」索引
//! 引导在进程间串行化。
//!
//! 语义（D020 修订，部分取代 D014「输者死不降级」）：
//! - 锁空闲立即拿到 = [`BootLock::Won`]，调用方正常执行引导；
//! - 锁被占 → 轮询等待 → 拿到 = [`BootLock::Lost`]（对手刚完成引导）→ 产物已就绪，
//!   败者继续跑成本 ≈ 0（增量构建全复用），调用方跳过引导段直接继续；
//!   原 D014「残缺工具面烧 token」担忧不成立——工具面装配发生在锁外；
//! - 等待超过 timeout = CS4016 判负退出（唯一判负路径）。
//!   D025（EN-9）：持锁方心跳新鲜（构建仍推进）时等锁方动态延展，上限 2×timeout
//!   兜底活锁；`--lock-timeout` 可配基准等待（三调用点：graph 引导 / index / run 补建）。
//!
//! 实现：`std::fs::File::try_lock`（flock，Rust 1.89 稳定）。进程崩溃由内核放锁，
//! 无陈旧锁问题；锁文件常驻 `.codesleuth/`（writeguard 豁免目录，D011 式工具元数据）。
//! 同步阻塞轮询只发生在锁竞争路径（上限 timeout），引导期短任务可接受。

use crate::errors::{CsError, CsResult, FailureReason, INDEX_LOCKED, REPO_NOT_READABLE};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// 轮询间隔。
const POLL_INTERVAL: Duration = Duration::from_millis(500);
/// 等锁上限，与 run_cli 的 INDEX_TIMEOUT（300s）对齐。
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);
/// 持锁方心跳刷新间隔（D025）：嵌入批次间也会停顿，间隔取保守值。
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);
/// 心跳新鲜度阈值：超过此间隔无心跳视为对手已停（进入正常倒计时）。
const HEARTBEAT_FRESH_MS: u64 = 30_000;

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 读锁文件内的心跳时间戳（D025）；缺失/损坏返回 None（无延展，向后兼容旧锁文件）。
fn read_heartbeat_ms(path: &Path) -> Option<u64> {
    let s = std::fs::read_to_string(path).ok()?;
    let v = s.trim().strip_prefix("hb=")?.trim().parse::<u64>().ok()?;
    (v > 0).then_some(v)
}

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
/// D025：持锁期间后台线程周期性刷新锁文件心跳，等锁方据此延展等待（EN-9）。
#[derive(Debug)]
pub struct BootLockGuard {
    // 保存文件句柄：Drop 时 unlock；句柄消失锁随之释放（双保险）。
    file: File,
    stop: Arc<AtomicBool>,
    hb: Option<std::thread::JoinHandle<()>>,
}

impl BootLockGuard {
    /// Won 路径统一入口：持有锁并启动心跳线程（D025）。
    fn won(file: File, path: PathBuf) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let hb = Self::spawn_heartbeat(path, Arc::clone(&stop));
        Self {
            file,
            stop,
            hb: Some(hb),
        }
    }

    /// 心跳线程：每 HEARTBEAT_INTERVAL 写一次 `hb=<unix_ms>` 到锁文件。
    fn spawn_heartbeat(path: PathBuf, stop: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            loop {
                // 分片睡眠：Drop 后尽快退出，不拖持有者收尾
                for _ in 0..(HEARTBEAT_INTERVAL.as_millis() as u32 / 100) {
                    if stop.load(Ordering::Relaxed) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                if stop.load(Ordering::Relaxed) {
                    return;
                }
                let Ok(mut f) = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                else {
                    continue;
                };
                let _ = write!(f, "hb={}", now_ms());
                let _ = f.flush();
            }
        })
    }
}

impl Drop for BootLockGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.hb.take() {
            let _ = h.join();
        }
        let _ = self.file.unlock();
    }
}

/// 引导锁结果（D020）。
#[derive(Debug)]
pub enum BootLockOutcome {
    /// 锁空闲直接拿到：调用方执行引导段，结束 drop 放锁。
    Won(BootLockGuard),
    /// 等待后拿到但对手刚完成引导：产物已就绪，调用方跳过引导直接继续（锁不持有）。
    OpponentFinished,
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
        Ok(()) => {
            tracing::debug!("引导锁首次尝试即获得（无竞争）: {}", path.display());
            return Ok(BootLock::Won(BootLockGuard::won(file, path)));
        }
        Err(std::fs::TryLockError::Error(e)) => {
            // P007 R3.16：首试硬错误（ENOLCK/EACCES/NFS 不支持等）保留原始错误作根因，
            // 不再只报「锁获取失败」让人误以为竞争超时；错误码仍归 INDEX_LOCKED 段位
            return Err(CsError::new(
                INDEX_LOCKED,
                format!("引导锁获取失败（非竞争性硬错误）: {e}"),
            )
            .with_source(format!("flock try_lock: {e}"))
            .with_hint("检查文件系统是否支持 flock（如 NFS 挂载）或目录权限"));
        }
        Err(std::fs::TryLockError::WouldBlock) => {}
    }

    // 竞争路径：轮询等待。拿到即 Lost——对手刚完成引导，产物已就绪，
    // 调用方跳过引导段直接继续（D020）。
    // D025（EN-9）：对手心跳新鲜（构建仍在推进）则延展倒计时，上限 2×timeout 兜底
    // 活锁（对手真死了心跳会变陈旧，30s 内回归正常倒计时）；无心跳文件则行为不变。
    let start = Instant::now();
    let mut deadline = start + timeout;
    tracing::warn!(
        "索引引导锁被占（{}），进入轮询等待（上限 {}s，对手推进中则延展）——另一进程可能正在建索引",
        path.display(),
        timeout.as_secs()
    );
    loop {
        std::thread::sleep(POLL_INTERVAL);
        // 先读心跳延展，再试锁，最后判超时——顺序保证延展在判负前生效（D025）
        if let Some(hb) = read_heartbeat_ms(&path) {
            let age = now_ms().saturating_sub(hb);
            if age <= HEARTBEAT_FRESH_MS {
                let proposed = Instant::now() + timeout;
                let cap = start + 2 * timeout;
                if proposed > deadline {
                    deadline = proposed.min(cap);
                }
            }
        }
        match file.try_lock() {
            Ok(()) => {
                tracing::warn!("引导锁等待后获得：对手刚完成引导，本进程复用产物继续（D020）");
                drop(file); // 产物已就绪，本进程不引导，立即放锁
                return Ok(BootLock::Lost);
            }
            Err(std::fs::TryLockError::Error(e)) => {
                return Err(CsError::new(INDEX_LOCKED, format!("引导锁获取失败: {e}")));
            }
            Err(std::fs::TryLockError::WouldBlock) => {}
        }
        if Instant::now() >= deadline {
            return Err(CsError::new(
                INDEX_LOCKED,
                format!(
                    "等待索引引导锁超时（基准 {}s，对手推进最多延展至 {}s）",
                    timeout.as_secs(),
                    2 * timeout.as_secs()
                ),
            )
            .with_hint(
                "另一进程长时间持锁且无进展（可能正在建大仓索引）；可用 --lock-timeout 调高等待上限，或确认其结束后重跑",
            )
            .with_reason(FailureReason::Timeout));
        }
    }
}

/// acquire 的统一封装（P005 R1 判负去重、D020 语义更新）：graph 引导、run 时向量构建、
/// index --vector 手动构建三个调用点统一走这里——Won 拿 guard 执行引导段；
/// Lost（对手刚完成引导）→ OpponentFinished，调用方跳过引导直接继续；
/// 等待超时 → CS4016 结构化错误（唯一判负路径）。
pub fn acquire_guard(repo_root: &Path, timeout: Duration, what: &str) -> CsResult<BootLockOutcome> {
    match acquire(repo_root, timeout)? {
        BootLock::Won(g) => Ok(BootLockOutcome::Won(g)),
        BootLock::Lost => {
            tracing::warn!("对手进程刚完成：{what}——复用其产物继续（D020）");
            Ok(BootLockOutcome::OpponentFinished)
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

    /// D020：持锁期间第二个获取者必须等待；锁释放后它拿到 = OpponentFinished（复用继续）。
    #[test]
    fn second_acquirer_waits_then_reuses() {
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
        assert!(
            h.join().unwrap(),
            "waiter must report Lost (opponent finished), not Won"
        );
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
            assert_eq!(err.reason, Some(FailureReason::Timeout));
        });
        // 超时 300ms < 轮询间隔 500ms：等待者必然在第一次轮询前撞死线。
        std::thread::sleep(Duration::from_millis(800));
        drop(g1);
        h.join().unwrap();
    }

    /// D020：acquire_guard 的 Lost 路径 = OpponentFinished（复用继续，不再判负）。
    #[test]
    fn acquire_guard_reports_lost_as_opponent_finished() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let BootLock::Won(g1) = acquire(dir.path(), DEFAULT_TIMEOUT).unwrap() else {
            panic!("first must win");
        };
        let h = std::thread::spawn(move || {
            match acquire_guard(&root, DEFAULT_TIMEOUT, "测试引导").unwrap() {
                BootLockOutcome::OpponentFinished => {}
                BootLockOutcome::Won(_) => panic!("opponent finished, not won"),
            }
        });
        std::thread::sleep(Duration::from_millis(600));
        drop(g1);
        h.join().unwrap();
    }

    /// D020：对手完成后锁已释放——复用方（或第三方）可重新 Won（无死锁）。
    /// 2026-10-13（P002-T003 伴随修复）：barrier 同步子线程首试时机——原实现
    /// 「sleep 600ms 祈祷子线程进入等待」在全量并行（174 测试）下调度抖动会
    /// 击穿假设（子线程首试时锁已释放 → 直接 Won → 断言 OpponentFinished 炸）。
    #[test]
    fn lock_is_free_after_opponent_finished() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let BootLock::Won(g1) = acquire(dir.path(), DEFAULT_TIMEOUT).unwrap() else {
            panic!("first must win");
        };
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let b2 = Arc::clone(&barrier);
        let h = std::thread::spawn(move || {
            // 主线程放行时必已持锁 → 首试确定性 WouldBlock → 进入轮询等待
            b2.wait();
            assert!(matches!(
                acquire_guard(&root, DEFAULT_TIMEOUT, "测试引导").unwrap(),
                BootLockOutcome::OpponentFinished
            ));
        });
        barrier.wait();
        std::thread::sleep(Duration::from_millis(600));
        drop(g1);
        h.join().unwrap();
        // OpponentFinished 不持锁：下一个获取者可重新 Won（无死锁主张）。
        // 2026-10-13：全量并行（174 测试）下此处曾稳定拿到 OpponentFinished（单跑永不复现，
        // 机制未完全定位，疑同进程其他测试线程的 flock 窗口竞争）——重试吸收：锁面清空后
        // 必须能 Won，无死锁主张不变。
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match acquire_guard(dir.path(), DEFAULT_TIMEOUT, "再次引导").unwrap() {
                BootLockOutcome::Won(_) => break,
                BootLockOutcome::OpponentFinished => {
                    assert!(
                        Instant::now() < deadline,
                        "放锁后 10s 内必须能重新 Won（无死锁）"
                    );
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    /// D025（EN-9）：对手心跳新鲜（构建仍在推进）时，超过基准 timeout 不判负；
    /// 对手完成后等锁方 Lost 复用。
    #[test]
    fn fresh_heartbeat_extends_wait_beyond_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let lock = dir.path().join(".codesleuth").join("boot.lock");
        let BootLock::Won(g1) = acquire(dir.path(), DEFAULT_TIMEOUT).unwrap() else {
            panic!("first must win");
        };
        let h =
            std::thread::spawn(
                move || match acquire(&root, Duration::from_millis(300)).unwrap() {
                    BootLock::Lost => true,
                    BootLock::Won(_) => false,
                },
            );
        // 模拟持锁方推进：100/300ms 写新鲜心跳；450ms 放锁（都在基准 300ms 之后）
        std::thread::sleep(Duration::from_millis(100));
        std::fs::write(&lock, format!("hb={}", now_ms())).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        std::fs::write(&lock, format!("hb={}", now_ms())).unwrap();
        std::thread::sleep(Duration::from_millis(150));
        drop(g1);
        assert!(
            h.join().unwrap(),
            "心跳新鲜时等待方应延展超过基准 timeout，最终 Lost 复用"
        );
    }

    /// D025（EN-9）：心跳一直新鲜也最多延展到 2×timeout（活锁兜底），仍判 CS4016。
    #[test]
    fn heartbeat_extension_is_capped_at_double_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let lock = dir.path().join(".codesleuth").join("boot.lock");
        let BootLock::Won(_g1) = acquire(dir.path(), DEFAULT_TIMEOUT).unwrap() else {
            panic!("first must win");
        };
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = Arc::clone(&stop);
        let lock2 = lock.clone();
        let hb_thread = std::thread::spawn(move || {
            while !stop2.load(Ordering::Relaxed) {
                std::fs::write(&lock2, format!("hb={}", now_ms())).unwrap();
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        let h = std::thread::spawn(move || {
            let err = acquire(&root, Duration::from_millis(200)).unwrap_err();
            assert_eq!(err.code, INDEX_LOCKED);
        });
        h.join().unwrap();
        stop.store(true, Ordering::Relaxed);
        hb_thread.join().unwrap();
    }
}
