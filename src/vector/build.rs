//! 向量索引构建（P003 E1b → 精简版）：切块 → 组装（无 LLM 描述）→ 嵌入 → SQLite 落地。
//!
//! 描述层已按用户裁决移除（2026-10-04）：大仓首建的 80% 时间花在逐块 LLM 描述上，
//! 生产系统（Cursor 等）不做索引期 LLM 改写——模态对齐交给代码原生嵌入模型或热点富化。
//! 增量：text_hash 未变的 chunk 整体复用（向量零开销）。

use crate::errors::{CsError, CsResult, REPO_NOT_FOUND};
use crate::vector::chunk::{Chunk, plan_chunks};
use crate::vector::compose::{EmbedMode, compose_input};
use crate::vector::embed::EmbedClient;
use crate::vector::store::{VectorStore, chunk_key, fingerprint, index_path};
use dunce::canonicalize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BuildReport {
    pub chunks_total: usize,
    pub embedded: usize,
    pub reused: usize,
    pub gc_removed: usize,
    pub index_path: PathBuf,
}

/// 构建向量索引。
pub async fn build_vector_index(
    repo_root: &Path,
    state_dir: &Path,
    embed: EmbedClient,
    mode: EmbedMode,
) -> CsResult<BuildReport> {
    let repo_abs = canonicalize(repo_root)
        .map_err(|e| CsError::new(REPO_NOT_FOUND, format!("仓库路径无效: {e}")))?;
    let fp = fingerprint(&repo_abs);
    let db_path = repo_abs.join(".codegraph").join("codegraph.db");
    let nodes = crate::vector::chunk::symbols_from_codegraph(&db_path).unwrap_or_default();
    let chunks = plan_chunks(&repo_abs, &nodes)?;

    let store = VectorStore::open(&index_path(state_dir, &fp))?;
    let existing = store.existing_hashes()?;
    // 模型/维度/模式任一变更 → 旧向量全部作废（嵌入输入变了，复用即投毒）
    let stored_model = store.get_meta("model")?;
    let stored_dim = store.get_meta("dim")?;
    let stored_mode = store.get_meta("mode")?;
    let mode_str = format!("{mode:?}");
    let stale_index = stored_model.as_deref() != Some(embed.model.as_str())
        || stored_dim.as_deref() != Some(&embed.dimensions.to_string())
        || stored_mode.as_deref() != Some(mode_str.as_str());

    // GC（用户拍板 2026-10-04）：当前块集合之外的旧块连向量带描述一起清（索引学会忘记死数据）。
    // P005 R3（bugs 审查 P1）：这里只算键不删——删除挪进 commit_build 的提交事务，
    // 消灭「旧块已删、新块写一半」的失败窗口。
    let current_keys: std::collections::HashSet<String> = chunks.iter().map(chunk_key).collect();
    let gc_keys: Vec<String> = existing
        .keys()
        .filter(|k| !current_keys.contains(*k))
        .cloned()
        .collect();

    let mut pending: Vec<Chunk> = Vec::new();
    let mut reused = 0usize;
    for c in &chunks {
        let key = chunk_key(c);
        if !stale_index && existing.get(&key) == Some(&c.text_hash) {
            reused += 1;
            continue;
        }
        pending.push(c.clone());
    }
    tracing::info!("待嵌入 {} 块（复用 {}）", pending.len(), reused);

    // 组装 + 嵌入（网络调用在事务外：失败时索引零改动）
    let mut inputs: Vec<(Chunk, String)> = Vec::with_capacity(pending.len());
    for c in &pending {
        let input = compose_input(c, mode);
        inputs.push((c.clone(), input));
    }
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

    let path = index_path(state_dir, &fp);
    Ok(BuildReport {
        chunks_total: chunks.len(),
        embedded: inputs.len(),
        reused,
        gc_removed,
        index_path: path,
    })
}
