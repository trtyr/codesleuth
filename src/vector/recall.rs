//! 召回引擎（P003 E1b）：SQLite 真相快照 + 全量精确余弦暴力召回。
//!
//! P007 R4.1 大扫除：HNSW 影子图 + 墓碑/压缩整套机制删除——MIN_HNSW_SIZE=100_000
//! 的「暖启动零负担」拍板意味着现实所有仓库都走暴力路径，影子图是永不激活的
//! 投机通用化（overdesign 审查裁决）。召回语义：全量精确余弦，结果确定性排序。

use crate::errors::{CsError, CsResult, INDEX_NOT_AVAILABLE};
use crate::vector::chunk::Chunk;
use crate::vector::embed::{EmbedClient, instruct_query};
use crate::vector::store::{VectorStore, cosine};

/// 召回引擎：一次性真相快照（open 后只读；GC 发生在下次构建的事务里）。
pub struct RecallEngine {
    embed: EmbedClient,
    rows: Vec<(Chunk, Vec<f32>)>,
    /// P007 R2.13：meta（model/dim）与当前嵌入客户端是否一致——
    /// 不一致时 zip 截断余弦会产生无意义相似度，召回必须拒绝而非硬算。
    meta_ok: bool,
}

impl RecallEngine {
    /// 从已填充的索引库构建。
    pub fn open(store: VectorStore, embed: EmbedClient) -> CsResult<Self> {
        let rows = store.load_all()?;
        let model_ok = store.get_meta("model")?.as_deref() == Some(embed.model.as_str());
        // P005 R6.1：全行维度校验（原只看第一行——首行正常、后续行损坏时会静默漏检）
        let dim_ok = rows.iter().all(|(_, v)| v.len() as u32 == embed.dimensions);
        Ok(Self {
            embed,
            rows,
            meta_ok: model_ok && dim_ok,
        })
    }

    /// 按向量召回（同步）：返回 (row 序号, 余弦相似度)，相似度降序。
    pub fn recall_by_vector(&self, qv: &[f32], k: usize) -> Vec<(usize, f32)> {
        let mut scored: Vec<(usize, f32)> = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, (_, v))| (i, cosine(v, qv)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    /// 文本查询（异步）：指令前缀嵌入 → 精确余弦召回。查询侧走 Qwen3 指令感知前缀。
    pub async fn recall(&self, query: &str, k: usize) -> CsResult<Vec<(Chunk, f32)>> {
        // P007 R2.13：meta 不一致时拒绝召回——跨模型/跨维度 zip 截断余弦是无意义分数，
        // 误导 agent；诚实报错并指向重建
        if !self.meta_ok {
            return Err(CsError::new(
                INDEX_NOT_AVAILABLE,
                "向量索引 meta 与当前嵌入模型/维度不一致，召回结果无语义",
            )
            .with_hint("重建索引后重试：codesleuth index --vector（或 run --fresh-index）"));
        }
        let qv = self.embed.embed(&[instruct_query(query)]).await?;
        let qv = qv
            .into_iter()
            .next()
            .ok_or_else(|| CsError::new(INDEX_NOT_AVAILABLE, "嵌入返回空向量"))?;
        Ok(self
            .recall_by_vector(&qv, k)
            .into_iter()
            .map(|(i, s)| (self.rows[i].0.clone(), s))
            .collect())
    }

    /// 召回结果格式化（注入模板：K≤10，相似度暴露，「可能无关」标注）。
    pub fn format_recall_block(hits: &[(Chunk, f32)], k: usize) -> String {
        let mut out = String::from(
            "[语义召回 · 起步线索]\n以下是与任务语义相似的代码位置，由向量检索给出，可能无关，仅供起步参考：\n",
        );
        for (i, (c, s)) in hits.iter().take(k).enumerate() {
            out.push_str(&format!(
                "{}. {}:{}-{} {}（相似度 {:.2}）\n",
                i + 1,
                c.file,
                c.line_start,
                c.line_end,
                c.symbol,
                s
            ));
        }
        out.push_str("（召回内容未经验证；作为证据使用前必须 read 原文。）\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 方向唯一化：v[i%dim]=1.0 + v[(7i+3)%dim]=0.5，确保每行方向不同（gcd(7,dim)=1 时）。
    /// 经公开写入口 commit_build 落库（R4.8 后无单写包装）。
    fn store_with(n: usize, dim: usize) -> (tempfile::TempDir, VectorStore, Vec<usize>) {
        let dir = tempfile::tempdir().unwrap();
        let store = VectorStore::open(&dir.path().join("v.db")).unwrap();
        let mut specials = Vec::new();
        let mut updates: Vec<(Chunk, Vec<f32>)> = Vec::new();
        for i in 0..n {
            let mut v = vec![0.0f32; dim];
            v[i % dim] = 1.0;
            v[(i * 7 + 3) % dim] += 0.5;
            let chunk = Chunk {
                file: format!("src/f{}.rs", i),
                line_start: 1,
                line_end: 10,
                symbol: format!("sym_{i}"),
                kind: "function".into(),
                language: "rust".into(),
                header: format!("header {i}"),
                text: format!("text {i}"),
                text_hash: format!("h{i}"),
            };
            updates.push((chunk, v));
            if i % 7 == 3 {
                specials.push(i);
            }
        }
        store
            .commit_build(&[], &updates, "test-model", dim as u32, "Raw")
            .unwrap();
        store.set_meta("model", "test-model").unwrap();
        (dir, store, specials)
    }

    fn engine_for(dim: usize, n: usize) -> (tempfile::TempDir, RecallEngine) {
        let (dir, store, _) = store_with(n, dim);
        let embed = EmbedClient::new("https://x.example", "k", "test-model", dim as u32);
        (dir, RecallEngine::open(store, embed).unwrap())
    }

    fn hot_query(dim: usize, row: usize) -> Vec<f32> {
        // 与 store_with 第 row 行同方向的查询向量
        let mut v = vec![0.0f32; dim];
        v[row % dim] = 1.0;
        v[(row * 7 + 3) % dim] += 0.5;
        v
    }

    #[test]
    fn brute_path_finds_exact_match() {
        let (_dir, engine) = engine_for(8, 6); // 全部方向互异
        let qv = hot_query(8, 5);
        let hits = engine.recall_by_vector(&qv, 3);
        assert_eq!(hits[0].0, 5, "top-1 应命中 row5: {hits:?}");
        assert!((hits[0].1 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn large_index_still_exact_brute() {
        // 2026-10-05 用户拍板「暖启动零负担」：千级向量全量精确暴力，确定性结果
        let (_dir, engine) = engine_for(16, 1200);
        let qv = hot_query(16, 9);
        let hits = engine.recall_by_vector(&qv, 5);
        assert!(!hits.is_empty());
        assert!(
            hits.iter().all(|(_, s)| (*s - 1.0).abs() < 1e-4),
            "方向匹配应近似 1.0: {hits:?}"
        );
    }

    #[test]
    fn recall_block_format_has_disclaimer() {
        let (_dir, engine) = engine_for(8, 6);
        let qv = hot_query(8, 3);
        let hits = engine.recall_by_vector(&qv, 3);
        let mapped: Vec<(Chunk, f32)> = hits
            .into_iter()
            .map(|(i, s)| (engine.rows[i].0.clone(), s))
            .collect();
        let block = RecallEngine::format_recall_block(&mapped, 10);
        assert!(block.contains("[语义召回 · 起步线索]"));
        assert!(block.contains("可能无关"));
        assert!(block.contains("必须 read 原文"));
        assert!(block.contains("相似度"));
    }

    #[test]
    fn meta_mismatch_refuses_recall() {
        // P007 R2.13：meta 不一致时召回拒绝（旧行为：降级暴力硬算无意义相似度）
        let (_dir, store, _) = store_with(1200, 8);
        store.set_meta("model", "other-model").unwrap();
        let embed = EmbedClient::new("https://x.example", "k", "test-model", 8);
        let engine = RecallEngine::open(store, embed).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let err = rt
            .block_on(engine.recall("query", 3))
            .expect_err("meta 不一致必须拒绝召回");
        assert_eq!(err.code, INDEX_NOT_AVAILABLE);
    }
}
