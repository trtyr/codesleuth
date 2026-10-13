//! 向量索引构建（P003 E1b → 精简版）：切块 → 组装（无 LLM 描述）→ 嵌入 → SQLite 落地。
//!
//! 描述层已按用户裁决移除（2026-10-04）：大仓首建的 80% 时间花在逐块 LLM 描述上，
//! 生产系统（Cursor 等）不做索引期 LLM 改写——模态对齐交给代码原生嵌入模型或热点富化。
//! 增量：text_hash 未变的 chunk 整体复用（向量零开销）。
//! EN-14/D030：git 脏检查快道——HEAD 未变 + 工作区 clean（.codegraph 除外）秒退，
//! 根治无变化仓的全仓枚举空扫（ai-guard 实证 22 分钟零嵌入）。

use crate::errors::{CsError, CsResult, REPO_NOT_FOUND};
use crate::vector::chunk::{Chunk, plan_chunks};
use crate::vector::compose::{EmbedMode, compose_input};
use crate::vector::embed::EmbedClient;
use crate::vector::store::{VectorStore, chunk_key, fingerprint, index_path};
use dunce::canonicalize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct BuildReport {
    pub chunks_total: usize,
    pub embedded: usize,
    pub reused: usize,
    pub gc_removed: usize,
    pub index_path: PathBuf,
    /// EN-14/D030：git 快道命中（HEAD 未变 + 工作区 clean），本次零扫描零嵌入
    pub skipped: bool,
}

/// git 可判定状态（EN-14/D030）。
/// None = 非 git 仓 / git 命令缺失或失败 / unborn 分支——一律跳过快道走原路径
/// （快道是优化不是正确性依赖，Q003 拍板）。
pub(crate) struct GitState {
    head: String,
    clean: bool,
}

fn read_git_state(repo: &Path) -> Option<GitState> {
    let head_out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !head_out.status.success() {
        return None; // 非 git 仓 / unborn 分支
    }
    let head = String::from_utf8_lossy(&head_out.stdout).trim().to_string();
    if head.is_empty() {
        return None;
    }
    // .codegraph/ 是本工具自建符号真源，不构成「仓库有变化」——
    // pathspec exclude 让 git 自己过滤，不做输出字符串解析
    let st = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["status", "--porcelain", "--", ".", ":(exclude).codegraph"])
        .output()
        .ok()?;
    if !st.status.success() {
        return None;
    }
    let clean = String::from_utf8_lossy(&st.stdout).trim().is_empty();
    Some(GitState { head, clean })
}

/// 构建向量索引。
pub async fn build_vector_index(
    repo_root: &Path,
    state_dir: &Path,
    embed: EmbedClient,
    mode: EmbedMode,
    // D023（EN-6）：config [index].exclude glob，透传 plan_chunks 排除链
    exclude_globs: &[String],
) -> CsResult<BuildReport> {
    let repo_abs = canonicalize(repo_root)
        .map_err(|e| CsError::new(REPO_NOT_FOUND, format!("仓库路径无效: {e}")))?;
    let fp = fingerprint(&repo_abs);
    let idx = index_path(state_dir, &fp);
    let store = VectorStore::open(&idx)?;
    // 模型/维度/模式任一变更 → 旧向量全部作废（嵌入输入变了，复用即投毒）
    let stored_model = store.get_meta("model")?;
    let stored_dim = store.get_meta("dim")?;
    let stored_mode = store.get_meta("mode")?;
    let mode_str = format!("{mode:?}");
    let stale_index = stored_model.as_deref() != Some(embed.model.as_str())
        || stored_dim.as_deref() != Some(&embed.dimensions.to_string())
        || stored_mode.as_deref() != Some(mode_str.as_str());

    // EN-14/D030：git 脏检查快道——在一切扫描之前。HEAD 仅作短路判据，
    // 绝不回流指纹键控（D024 语义不变）；stale 一票否决（换模型必须重嵌）。
    if !stale_index
        && let Some(gs) = read_git_state(&repo_abs)
        && store.get_meta("git_head")?.as_deref() == Some(gs.head.as_str())
        && gs.clean
    {
        tracing::info!(
            component = "vector_build",
            "git 脏检查快道命中：HEAD 未变且工作区 clean，nothing to embed"
        );
        return Ok(BuildReport {
            chunks_total: 0,
            embedded: 0,
            reused: 0,
            gc_removed: 0,
            index_path: idx,
            skipped: true,
        });
    }

    let db_path = repo_abs.join(".codegraph").join("codegraph.db");
    // P007 R3.19：codegraph 读取失败不再静默——符号边界真源失效退化为全文
    // text 切块是显著质量降级，至少 warn 一次
    let nodes = match crate::vector::chunk::symbols_from_codegraph(&db_path) {
        Ok(n) => n,
        Err(e) => {
            tracing::warn!(
                component = "vector_build",
                error = %e,
                "codegraph 符号读取失败，退化全文 text 切块（符号边界失效）"
            );
            Default::default()
        }
    };
    let chunks = plan_chunks(&repo_abs, &nodes, exclude_globs)?;

    let existing = store.existing_hashes()?;

    // GC（用户拍板 2026-10-04）：当前块集合之外的旧块连向量带描述一起清（索引学会忘记死数据）。
    // P005 R3（bugs 审查 P1）：这里只算键不删——删除挪进 commit_build 的提交事务，
    // 消灭「旧块已删、新块写一半」的失败窗口。
    let current_keys: std::collections::HashSet<String> = chunks.iter().map(chunk_key).collect();
    let gc_keys: Vec<String> = existing
        .keys()
        .filter(|k| !current_keys.contains(*k))
        .cloned()
        .collect();

    let mut inputs: Vec<(Chunk, String)> = Vec::with_capacity(chunks.len());
    let mut reused = 0usize;
    for c in &chunks {
        let key = chunk_key(c);
        // P007 R2.12：复用判定改为对「组装后的嵌入输入」做 hash——旧实现在 raw text 上
        // 做 sha256，Composite 模式下子窗块的 text 不含 header（签名/docstring），
        // 仅改签名时 text_hash 不变 → 旧向量被错误复用（增量复用投毒）。
        // Raw 模式 compose=text，语义不变。
        let input = compose_input(c, mode);
        let hash = crate::vector::chunk::sha256_hex(&input);
        if !stale_index && existing.get(&key) == Some(&hash) {
            reused += 1;
            continue;
        }
        inputs.push((c.clone(), input));
    }
    tracing::info!("待嵌入 {} 块（复用 {}）", inputs.len(), reused);
    let texts: Vec<String> = inputs.iter().map(|(_, i)| i.clone()).collect();
    let vectors = embed.embed(&texts).await?;
    let updates: Vec<(Chunk, Vec<f32>)> = inputs
        .iter()
        .zip(vectors.iter())
        .map(|((c, _), v)| (c.clone(), v.clone()))
        .collect();

    // 原子落库（P005 R3）：GC 删除 + 全部 upsert + meta 同一事务，失败整体回滚
    let gc_removed = store.commit_build(
        &gc_keys,
        &updates,
        &embed.model,
        embed.dimensions,
        &mode_str,
    )?;
    if gc_removed > 0 {
        tracing::info!("GC: 清理 {} 个失效块", gc_removed);
    }

    // EN-14/D030：构建成功记录 HEAD（下次快道判据）。commit 事务外 best-effort：
    // 写失败只损失下一次快道命中（退化为原增量路径），正确性无损
    if let Some(gs) = read_git_state(&repo_abs) {
        store.set_meta("git_head", &gs.head)?;
    }

    Ok(BuildReport {
        chunks_total: chunks.len(),
        embedded: inputs.len(),
        reused,
        gc_removed,
        index_path: idx,
        skipped: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// git 命令包装（ci 环境已装 git；commit 场景用 -c 注入身份）
    fn git(repo: &Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .expect("git 命令执行失败（ci 环境需装 git）");
        assert!(
            out.status.success(),
            "git {args:?} 失败: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn init_repo_with_commit(dir: &Path) {
        git(dir, &["init", "-q"]);
        git(
            dir,
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "--allow-empty",
                "-q",
                "-m",
                "init",
            ],
        );
    }

    #[test]
    fn git_state_clean_repo_detects_head() {
        let dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(dir.path());
        let gs = read_git_state(dir.path()).expect("git 仓应可判定");
        assert!(!gs.head.is_empty());
        assert!(gs.clean, "仅空 commit 的仓应为 clean");
    }

    #[test]
    fn git_state_detects_dirty_worktree() {
        let dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(dir.path());
        std::fs::write(dir.path().join("a.txt"), "hi").unwrap();
        let gs = read_git_state(dir.path()).unwrap();
        assert!(!gs.clean, "新增未跟踪文件应为 dirty");
    }

    #[test]
    fn git_state_excludes_codegraph_dir() {
        // .codegraph/ 是自建符号真源：其变更不构成仓库变化（否则构建自己污染快道）
        let dir = tempfile::tempdir().unwrap();
        init_repo_with_commit(dir.path());
        std::fs::create_dir_all(dir.path().join(".codegraph")).unwrap();
        std::fs::write(dir.path().join(".codegraph/codegraph.db"), "x").unwrap();
        let gs = read_git_state(dir.path()).unwrap();
        assert!(gs.clean, ".codegraph 变更应被 pathspec 排除");
    }

    #[test]
    fn git_state_none_for_non_git_and_unborn() {
        let dir = tempfile::tempdir().unwrap();
        assert!(
            read_git_state(dir.path()).is_none(),
            "非 git 仓 → None（跳过快道走原路径）"
        );
        let dir2 = tempfile::tempdir().unwrap();
        git(dir2.path(), &["init", "-q"]); // unborn：init 后无 commit
        assert!(read_git_state(dir2.path()).is_none(), "unborn 分支 → None");
    }

    #[tokio::test]
    async fn fast_forward_hits_without_embed_call() {
        // 索引 meta 与 git HEAD 全对齐 + clean → 秒退；embed 指向死端口，
        // 快道命中则绝不触网（触网即连接拒绝 → 测试失败）
        let repo = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        init_repo_with_commit(repo.path());
        let repo_abs = canonicalize(repo.path()).unwrap();
        let fp = fingerprint(&repo_abs);
        let idx = index_path(state.path(), &fp);
        {
            let s = VectorStore::open(&idx).unwrap();
            s.set_meta("model", "mock-model").unwrap();
            s.set_meta("dim", "4").unwrap();
            s.set_meta("mode", "Raw").unwrap();
            let gs = read_git_state(&repo_abs).unwrap();
            s.set_meta("git_head", &gs.head).unwrap();
        }
        let embed = EmbedClient::new("http://127.0.0.1:1", "k", "mock-model", 4);
        let report = build_vector_index(repo.path(), state.path(), embed, EmbedMode::Raw, &[])
            .await
            .expect("快道命中应成功返回");
        assert!(report.skipped, "HEAD 未变 + clean 应秒退");
        assert_eq!(report.embedded, 0);
    }

    #[tokio::test]
    async fn fast_forward_misses_when_head_differs() {
        // git_head 存的是旧值（模拟上游有新 commit）→ 快道 miss 走原路径
        // → embed 触死端口报错（证明未秒退、原增量路径语义不变）
        let repo = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        init_repo_with_commit(repo.path());
        let repo_abs = canonicalize(repo.path()).unwrap();
        let fp = fingerprint(&repo_abs);
        let idx = index_path(state.path(), &fp);
        {
            let s = VectorStore::open(&idx).unwrap();
            s.set_meta("model", "mock-model").unwrap();
            s.set_meta("dim", "4").unwrap();
            s.set_meta("mode", "Raw").unwrap();
            s.set_meta("git_head", "0000000000000000000000000000000000000000")
                .unwrap();
        }
        let embed = EmbedClient::new("http://127.0.0.1:1", "k", "mock-model", 4);
        let report = build_vector_index(repo.path(), state.path(), embed, EmbedMode::Raw, &[])
            .await
            .expect("HEAD 变化走原路径：空仓零嵌入正常完成");
        assert!(
            !report.skipped,
            "HEAD 变化应走原路径，不得秒退（skipped=false）"
        );
    }
}
