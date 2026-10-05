//! 召回引擎（P003 E1b · 设计稿 §3）：SQLite 真相 + 会话内 HNSW 影子图 + 三段式查询。
//!
//! 三段式：① HNSW k×3 超采（ef_search=64）→ ② 候选精确重算余弦 → ③ 墓碑过滤取 top-K。
//! <1000 向量走暴力（该量级 HNSW 无收益且更精确）；墓碑超 20% 触发 compact。

use crate::errors::{CsError, CsResult, INDEX_NOT_AVAILABLE};
use crate::vector::chunk::Chunk;
use crate::vector::embed::{EmbedClient, instruct_query};
use crate::vector::store::{VectorStore, cosine};
use hnsw_rs::prelude::*;
use std::collections::HashSet;

pub const HNSW_M: usize = 16;
pub const HNSW_MAX_LAYER: usize = 16;
pub const HNSW_EF_CONSTRUCTION: usize = 200;
pub const HNSW_EF_SEARCH: usize = 64;
/// 查询超采倍数（ANN 近似 → 精确复算前的候选池扩容）。
pub const OVERFETCH_FACTOR: usize = 3;
/// 低于此规模直接暴力（HNSW 无收益且更精确）。
pub const MIN_HNSW_SIZE: usize = 100_000; // 暖启动零负担（2026-10-05 用户硬要求）：≤10 万块直接精确暴力（查询亚秒级），免会话启动 HNSW 重建分钟级成本；HNSW 只留给巨型仓
/// 墓碑占比超过此值应 compact。
pub const TOMBSTONE_REBUILD_RATIO: f32 = 0.2;

/// 召回引擎：持真相快照（rows）+ 活跃集合（alive）+ 可选 HNSW 影子图。
pub struct RecallEngine {
    embed: EmbedClient,
    rows: Vec<(Chunk, Vec<f32>)>,
    alive: HashSet<usize>,
    hnsw: Option<Hnsw<'static, f32, DistCosine>>,
}

impl RecallEngine {
    /// 从已填充的索引库构建。meta 中的 model/维度与 embed 客户端不一致时降级暴力并省略 HNSW。
    pub fn open(store: VectorStore, embed: EmbedClient) -> CsResult<Self> {
        let rows = store.load_all()?;
        let model_ok = store.get_meta("model")?.as_deref() == Some(embed.model.as_str());
        let dim = rows.first().map(|(_, v)| v.len());
        let dim_ok = dim.map(|d| d as u32 == embed.dimensions).unwrap_or(true);
        let mut alive = HashSet::new();
        for i in 0..rows.len() {
            alive.insert(i);
        }
        let mut engine = Self {
            embed,
            rows,
            alive,
            hnsw: None,
        };
        if engine.rows.len() >= MIN_HNSW_SIZE && model_ok && dim_ok {
            engine.build_hnsw();
        }
        Ok(engine)
    }

    fn build_hnsw(&mut self) {
        let n = self.rows.len();
        let hnsw = Hnsw::new(HNSW_M, n, HNSW_MAX_LAYER, HNSW_EF_CONSTRUCTION, DistCosine);
        let datas: Vec<(&[f32], usize)> = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, (_, v))| (v.as_slice(), i))
            .collect();
        hnsw.parallel_insert_slice(&datas);
        self.hnsw = Some(hnsw);
    }

    /// 墓碑一个行号（更新/删除的旧块）。
    pub fn tombstone(&mut self, row: usize) {
        self.alive.remove(&row);
    }

    pub fn tombstone_count(&self) -> usize {
        self.rows.len() - self.alive.len()
    }

    /// 墓碑超阈值 → 需要压缩（丢死行、重编号、重建图）。
    pub fn needs_compact(&self) -> bool {
        let dead = self.rows.len() - self.alive.len();
        !self.rows.is_empty() && (dead as f32 / self.rows.len() as f32) > TOMBSTONE_REBUILD_RATIO
    }

    /// 压缩：丢死行重编号，重建图。
    pub fn compact(&mut self) {
        let mut new_rows = Vec::with_capacity(self.alive.len());
        for i in 0..self.rows.len() {
            if self.alive.contains(&i) {
                new_rows.push(self.rows[i].clone());
            }
        }
        self.rows = new_rows;
        self.alive = (0..self.rows.len()).collect();
        self.hnsw = None;
        if self.rows.len() >= MIN_HNSW_SIZE {
            self.build_hnsw();
        }
    }

    /// 按向量召回（同步）：返回 (row 序号, 余弦相似度)，相似度降序。
    pub fn recall_by_vector(&self, qv: &[f32], k: usize) -> Vec<(usize, f32)> {
        if let Some(hnsw) = &self.hnsw {
            let knbn = (k * OVERFETCH_FACTOR).min(self.alive.len().max(1));
            let mut scored: Vec<(usize, f32)> = hnsw
                .search(qv, knbn, HNSW_EF_SEARCH)
                .into_iter()
                .filter(|n| self.alive.contains(&n.d_id))
                .map(|n| {
                    let row = n.d_id;
                    (row, cosine(&self.rows[row].1, qv))
                })
                .collect();
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            scored.truncate(k);
            scored
        } else {
            let mut scored: Vec<(usize, f32)> = self
                .alive
                .iter()
                .map(|&i| (i, cosine(&self.rows[i].1, qv)))
                .collect();
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            scored.truncate(k);
            scored
        }
    }

    /// 文本查询（异步）：指令前缀嵌入 → 三段式召回。查询侧走 Qwen3 指令感知前缀。
    pub async fn recall(&self, query: &str, k: usize) -> CsResult<Vec<(Chunk, f32)>> {
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
    fn store_with(n: usize, dim: usize) -> (tempfile::TempDir, VectorStore, Vec<usize>) {
        let dir = tempfile::tempdir().unwrap();
        let store = VectorStore::open(&dir.path().join("v.db")).unwrap();
        let mut specials = Vec::new();
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
            store.upsert_chunk(&chunk, &v).unwrap();
            if i % 7 == 3 {
                specials.push(i);
            }
        }
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
    fn warm_start_brute_respects_alive() {
        // 2026-10-05 用户硬要求「暖启动零负担」：千级向量走精确暴力（MIN_HNSW_SIZE=100k），
        // 会话启动零 HNSW 重建成本；墓碑语义在暴力路径下同样成立。
        let (_dir, mut engine) = engine_for(16, 1200);
        assert!(
            engine.hnsw.is_none(),
            "千级向量应走精确暴力（暖启动零负担）"
        );
        let qv = hot_query(16, 9);
        let hits = engine.recall_by_vector(&qv, 5);
        assert!(!hits.is_empty());
        assert!(hits.iter().all(|(i, _)| engine.alive.contains(i)));
        assert!(
            hits.iter().all(|(_, s)| (*s - 1.0).abs() < 1e-4),
            "方向匹配应近似 1.0: {hits:?}"
        );
        // 墓碑 top-1 后不再出现
        let top = hits[0].0;
        engine.tombstone(top);
        let hits2 = engine.recall_by_vector(&qv, 5);
        assert!(hits2.iter().all(|(i, _)| *i != top));
        assert_eq!(engine.tombstone_count(), 1);
        assert!(!engine.needs_compact());
    }

    #[test]
    fn compact_renumbers_and_rebuilds() {
        let (_dir, mut engine) = engine_for(16, 1200);
        for i in 0..1000 {
            engine.tombstone(i);
        }
        assert!(engine.needs_compact());
        engine.compact();
        assert_eq!(engine.rows.len(), 200);
        assert_eq!(engine.tombstone_count(), 0);
        let qv = hot_query(16, 1005); // 存活行：1005 未被墓碑（0..1000 已碑）
        let hits = engine.recall_by_vector(&qv, 3);
        assert!(!hits.is_empty());
        assert!(
            hits.iter().all(|(_, s)| (*s - 1.0).abs() < 1e-4),
            "{hits:?}"
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
    fn meta_mismatch_skips_hnsw_but_brute_still_works() {
        let (_dir, store, _) = store_with(1200, 8);
        store.set_meta("model", "other-model").unwrap();
        let embed = EmbedClient::new("https://x.example", "k", "test-model", 8);
        let engine = RecallEngine::open(store, embed).unwrap();
        assert!(engine.hnsw.is_none(), "模型不一致不得建图");
        let qv = hot_query(8, 5);
        let hits = engine.recall_by_vector(&qv, 3);
        assert!(!hits.is_empty(), "模型不一致降级暴力，检索仍须可用");
    }
}
