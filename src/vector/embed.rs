//! 嵌入客户端（P003 E1a）：Qwen/Qwen3-Embedding-8B via OpenAI 兼容 `/embeddings`。
//!
//! 约定（scoring/检索设计稿 §1 + Q1 决策）：
//! - 维度默认 1024（Matryoshka 降维，E0 实验可调 4096 对比）
//! - 批量输入（batch_size 默认 64）
//! - 查询侧指令前缀（Qwen3 指令感知：query 加 Instruct，doc 侧不加）

use crate::errors::{CsError, CsResult, INDEX_EMBED_FAILED};
use serde_json::Value;

/// 查询侧指令前缀（Qwen3 指令感知官方用法；文档侧不加）。
pub const QUERY_INSTRUCT: &str =
    "Instruct: 给定代码库的自然语言问题，检索能回答它的代码片段\nQuery: ";

/// 嵌入批并发（2026-10-04 用户拍板加速；P007 R3.19 注释同步：当前值 4，
/// 网关速率限制口径；嵌入调用轻于 LLM 生成）。
pub const EMBED_CONCURRENCY: usize = 4;

pub fn instruct_query(query: &str) -> String {
    format!("{QUERY_INSTRUCT}{query}")
}

/// 构造 /embeddings 请求体（纯函数，单测断言形状）。
pub fn build_request(model: &str, texts: &[String], dimensions: u32) -> Value {
    serde_json::json!({
        "model": model,
        "input": texts,
        "dimensions": dimensions,
        "encoding_format": "float",
    })
}

/// 把 texts 按批大小切成 (start, end) 左闭右开区间。
pub fn split_batches(total: usize, batch_size: usize) -> Vec<(usize, usize)> {
    let bs = batch_size.max(1);
    (0..total)
        .step_by(bs)
        .map(|s| (s, (s + bs).min(total)))
        .collect()
}

#[derive(Clone)]
pub struct EmbedClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    pub model: String,
    pub dimensions: u32,
    batch_size: usize,
}

impl EmbedClient {
    pub fn new(base_url: &str, api_key: &str, model: &str, dimensions: u32) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            dimensions,
            batch_size: 64,
        }
    }

    pub fn with_batch_size(mut self, n: usize) -> Self {
        self.batch_size = n.max(1);
        self
    }

    /// 批量嵌入：按 batch_size 分批，工人池并发（EMBED_CONCURRENCY），按批序号还原顺序。
    pub async fn embed(&self, texts: &[String]) -> CsResult<Vec<Vec<f32>>> {
        use std::collections::HashMap;
        use std::sync::Arc;
        use tokio::sync::Semaphore;
        // 内容去重（2026-10-05 用户立项）：相同文本只嵌一次，向量按哈希回填——Go 系重复样板仓库省 10-30% 调用
        let mut unique: Vec<String> = Vec::with_capacity(texts.len());
        let mut index_by_hash: HashMap<String, usize> = HashMap::new();
        let mut slots: Vec<usize> = Vec::with_capacity(texts.len());
        for t in texts {
            let h = crate::vector::store::hash_of(t);
            match index_by_hash.get(&h) {
                Some(i) => slots.push(*i),
                None => {
                    slots.push(unique.len());
                    index_by_hash.insert(h, unique.len());
                    unique.push(t.clone());
                }
            }
        }
        let texts: &[String] = &unique;
        let batches = split_batches(texts.len(), self.batch_size);
        let total = batches.len();
        let sem = Arc::new(Semaphore::new(EMBED_CONCURRENCY));
        let mut set = tokio::task::JoinSet::new();
        for (bi, (start, end)) in batches.into_iter().enumerate() {
            let permit = Arc::clone(&sem)
                .acquire_owned()
                .await
                .map_err(|e| CsError::new(INDEX_EMBED_FAILED, format!("嵌入信号量失败: {e}")))?;
            let this = self.clone();
            let batch: Vec<String> = texts[start..end].to_vec();
            set.spawn(async move {
                let _guard = permit;
                // 瞬时故障兜底：失败后 2s 重试一次（E3-R1 engram 解码错误教训）
                let r = match this.embed_one(&batch).await {
                    Ok(v) => Ok(v),
                    Err(_) => {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        this.embed_one(&batch).await
                    }
                };
                (bi, r)
            });
        }
        let mut parts: Vec<(usize, CsResult<Vec<Vec<f32>>>)> = Vec::new();
        let mut done = 0usize;
        while let Some(res) = set.join_next().await {
            match res {
                Ok(tuple) => {
                    done += 1;
                    if done.is_multiple_of(25) || done == total {
                        tracing::info!("嵌入进度 {}/{} 批", done, total);
                    }
                    parts.push(tuple);
                }
                Err(e) => {
                    return Err(CsError::new(
                        INDEX_EMBED_FAILED,
                        format!("嵌入任务中止: {e}"),
                    ));
                }
            }
        }
        parts.sort_by_key(|(bi, _)| *bi);
        let mut unique_vectors: Vec<Vec<f32>> = Vec::with_capacity(unique.len());
        for (_, r) in parts {
            unique_vectors.extend(r?);
        }
        // 按去重前的槽位回填
        let mut out: Vec<Vec<f32>> = Vec::with_capacity(slots.len());
        for slot in &slots {
            out.push(unique_vectors[*slot].clone());
        }
        Ok(out)
    }

    /// 单批嵌入请求（工人任务体）。
    async fn embed_one(&self, batch: &[String]) -> CsResult<Vec<Vec<f32>>> {
        let url = format!("{}/embeddings", self.base_url);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&build_request(&self.model, batch, self.dimensions))
            .send()
            .await
            .map_err(|e| {
                CsError::new(INDEX_EMBED_FAILED, format!("embeddings 请求失败: {e}"))
                    .with_retryable(true)
            })?;
        let status = resp.status();
        // P007 R3.16：先取 status——非 2xx 且错误体非 JSON（网关 HTML 502 页）时，
        // 旧实现 json() 先失败把 HTTP 状态码和 retryable 分类全部吞掉
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let retryable = status.as_u16() == 429 || status.is_server_error();
            return Err(CsError::new(
                INDEX_EMBED_FAILED,
                format!(
                    "embeddings HTTP {status}: {}",
                    crate::llm::safe_prefix(&body, 200)
                ),
            )
            .with_retryable(retryable));
        }
        let value: Value = resp.json().await.map_err(|e| {
            CsError::new(INDEX_EMBED_FAILED, format!("embeddings 响应解析失败: {e}"))
        })?;
        let mut data: Vec<(usize, Vec<f32>)> = value["data"]
            .as_array()
            .ok_or_else(|| CsError::new(INDEX_EMBED_FAILED, "embeddings 响应缺 data 数组"))?
            .iter()
            .filter_map(|d| {
                let idx = d.get("index")?.as_u64()? as usize;
                let vec = d
                    .get("embedding")?
                    .as_array()?
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect::<Vec<f32>>();
                Some((idx, vec))
            })
            .collect();
        data.sort_by_key(|(idx, _)| *idx);
        if data.len() != batch.len() {
            return Err(CsError::new(
                INDEX_EMBED_FAILED,
                format!("embeddings 返回 {} 条，请求 {} 条", data.len(), batch.len()),
            ));
        }
        // P005 R4.2：错位即拒绝（内核见 validate_index_alignment）
        validate_index_alignment(&data)?;
        Ok(data.into_iter().map(|(_, v)| v).collect())
    }
}

/// P005 R4.2（bugs 审查 P2）：响应 index 与请求位次必须严格 0-based 对齐——
/// 数量校验抓不住「1-based 序号/乱序补齐」，向量错位会静默投毒；错位即拒绝。
fn validate_index_alignment(data: &[(usize, Vec<f32>)]) -> CsResult<()> {
    for (i, (idx, _)) in data.iter().enumerate() {
        if *idx != i {
            return Err(CsError::new(
                INDEX_EMBED_FAILED,
                format!(
                    "embeddings 返回 index {idx} 与期望位置 {i} 错位（网关非 0-based 序号？），拒绝写入防投毒"
                ),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_alignment_rejects_nonzero_base() {
        // P005 R4.2：1-based 序号/乱序补齐 → 错位拒绝，不静默投毒
        let bad = vec![(1usize, vec![0.1f32]), (2, vec![0.2])];
        assert!(validate_index_alignment(&bad).is_err());
        let gap = vec![(0usize, vec![0.1]), (2, vec![0.2])];
        assert!(validate_index_alignment(&gap).is_err());
        let ok = vec![(0usize, vec![0.1]), (1, vec![0.2])];
        assert!(validate_index_alignment(&ok).is_ok());
        assert!(validate_index_alignment(&[]).is_ok());
    }

    #[test]
    fn build_request_shape() {
        let req = build_request(
            "Qwen/Qwen3-Embedding-8B",
            &["a".to_string(), "b".to_string()],
            1024,
        );
        assert_eq!(req["model"], "Qwen/Qwen3-Embedding-8B");
        assert_eq!(req["dimensions"], 1024);
        assert_eq!(req["encoding_format"], "float");
        assert_eq!(req["input"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn instruct_prefix_wraps_query() {
        let q = instruct_query("请求失败怎么自动再试");
        assert!(q.starts_with("Instruct: "));
        assert!(q.ends_with("请求失败怎么自动再试"));
    }

    #[test]
    fn split_batches_covers_all() {
        assert!(split_batches(0, 32).is_empty());
        assert_eq!(split_batches(7, 32), vec![(0, 7)]);
        assert_eq!(split_batches(64, 32), vec![(0, 32), (32, 64)]);
        assert_eq!(split_batches(33, 32), vec![(0, 32), (32, 33)]);
    }
}
