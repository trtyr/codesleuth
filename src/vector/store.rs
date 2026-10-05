//! 向量索引存储（P003 E1a · Q3 已定）：SQLite 单文件 + BLOB 向量，Rust 侧检索。
//!
//! 真相原则：vectors.db 是嵌入索引的唯一持久真相；HNSW 图是会话内影子（E1b）。
//! 增量：text_hash 未变的 chunk 直接复用旧向量（embed 调用零开销）。

use crate::errors::{CsError, CsResult, INDEX_NOT_AVAILABLE};
use crate::vector::chunk::{Chunk, sha256_hex};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// chunk 唯一键（file + symbol + line_start）。
pub fn chunk_key(c: &Chunk) -> String {
    format!("{}\u{1}{}\u{1}{}", c.file, c.symbol, c.line_start)
}

/// 仓库指纹：sha256(绝对路径 + git HEAD) 前 16 hex。索引按此键控隔离。
pub fn fingerprint(repo_abs: &Path) -> String {
    let mut h = Sha256::new();
    h.update(repo_abs.to_string_lossy().as_bytes());
    if let Ok(head) = std::fs::read(repo_abs.join(".git").join("HEAD")) {
        h.update(&head);
    }
    let d = h.finalize();
    d.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn blob_to_vec(b: &[u8]) -> Vec<f32> {
    b.as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect()
}

pub struct VectorStore {
    conn: Connection,
}

impl VectorStore {
    /// 打开（不存在则创建）索引库，确保 schema。
    pub fn open(path: &Path) -> CsResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("创建索引目录失败: {e}")))?;
        }
        let conn = Connection::open(path)
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("打开索引库失败: {e}")))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("WAL 设置失败: {e}")))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY,
                file TEXT NOT NULL,
                line_start INTEGER NOT NULL,
                line_end INTEGER NOT NULL,
                symbol TEXT NOT NULL,
                kind TEXT NOT NULL,
                language TEXT NOT NULL,
                text_hash TEXT NOT NULL,
                embedding BLOB NOT NULL,
                dim INTEGER NOT NULL
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_chunks_key ON chunks(file, symbol, line_start);
            CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS descriptions (
                key TEXT PRIMARY KEY,
                text_hash TEXT NOT NULL,
                description TEXT NOT NULL
            );",
        )
        .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("建表失败: {e}")))?;
        Ok(Self { conn })
    }

    pub fn get_meta(&self, key: &str) -> CsResult<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM meta WHERE key = ?1")
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("meta 查询失败: {e}")))?;
        let mut rows = stmt
            .query_map([key], |r| r.get::<_, String>(0))
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("meta 遍历失败: {e}")))?;
        rows.next()
            .transpose()
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("meta 读取失败: {e}")))
    }

    pub fn set_meta(&self, key: &str, value: &str) -> CsResult<()> {
        self.conn
            .execute(
                "INSERT INTO meta (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [key, value],
            )
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("meta 写入失败: {e}")))?;
        Ok(())
    }



    /// 垃圾回收（用户拍板 2026-10-04）：删除当前块集合之外的失效块——向量与描述一起清，
    /// 索引学会忘记死数据（陈旧向量会指向已不存在的行号，污染召回）。
    pub fn remove_stale(&self, keys: &[String]) -> CsResult<usize> {
        let mut n = 0usize;
        for k in keys {
            let parts: Vec<&str> = k.split('\u{1}').collect();
            if parts.len() != 3 {
                continue;
            }
            let line_start = parts[2].parse::<i64>().unwrap_or(0);
            self.conn
                .execute(
                    "DELETE FROM chunks WHERE file = ?1 AND symbol = ?2 AND line_start = ?3",
                    rusqlite::params![parts[0], parts[1], line_start],
                )
                .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("GC 删除块失败: {e}")))?;
            self.conn
                .execute("DELETE FROM descriptions WHERE key = ?1", [k])
                .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("GC 删除描述失败: {e}")))?;
            n += 1;
        }
        Ok(n)
    }

    /// 已存 chunk 的 key → text_hash（增量重建的复用依据）。
    pub fn existing_hashes(&self) -> CsResult<HashMap<String, String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT file, symbol, line_start, text_hash FROM chunks")
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("chunks 查询失败: {e}")))?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    format!(
                        "{}\u{1}{}\u{1}{}",
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?
                    ),
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("chunks 遍历失败: {e}")))?;
        let mut map = HashMap::new();
        for row in rows {
            let (k, h) = row.map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("{e}")))?;
            map.insert(k, h);
        }
        Ok(map)
    }

    /// 写入/覆盖一个 chunk 及其向量。
    pub fn upsert_chunk(&self, chunk: &Chunk, vector: &[f32]) -> CsResult<()> {
        self.conn
            .execute(
                "INSERT INTO chunks (file, line_start, line_end, symbol, kind, language, text_hash, embedding, dim)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(file, symbol, line_start) DO UPDATE SET
                   line_end = excluded.line_end, kind = excluded.kind, language = excluded.language,
                   text_hash = excluded.text_hash, embedding = excluded.embedding, dim = excluded.dim",
                rusqlite::params![
                    chunk.file,
                    chunk.line_start as i64,
                    chunk.line_end as i64,
                    chunk.symbol,
                    chunk.kind,
                    chunk.language,
                    chunk.text_hash,
                    vec_to_blob(vector),
                    vector.len() as i64,
                ],
            )
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("chunk 写入失败: {e}")))?;
        Ok(())
    }

    /// 批量落库（单事务）：5 万块的首次构建从逐条自动提交的分钟级降到秒级。
    pub fn upsert_chunks(&self, items: &[(Chunk, Vec<f32>)]) -> CsResult<usize> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("事务开启失败: {e}")))?;
        for (chunk, vector) in items {
            tx.execute(
                "INSERT INTO chunks (file, line_start, line_end, symbol, kind, language, text_hash, embedding, dim)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(file, symbol, line_start) DO UPDATE SET
                   line_end = excluded.line_end, kind = excluded.kind, language = excluded.language,
                   text_hash = excluded.text_hash, embedding = excluded.embedding, dim = excluded.dim",
                rusqlite::params![
                    chunk.file,
                    chunk.line_start as i64,
                    chunk.line_end as i64,
                    chunk.symbol,
                    chunk.kind,
                    chunk.language,
                    chunk.text_hash,
                    vec_to_blob(vector),
                    vector.len() as i64,
                ],
            )
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("chunk 批量写入失败: {e}")))?;
        }
        tx.commit()
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("事务提交失败: {e}")))?;
        Ok(items.len())
    }

    pub fn count(&self) -> CsResult<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0))
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("计数失败: {e}")))
    }

    /// 全量加载（重建 HNSW 影子图的原料）。
    pub fn load_all(&self) -> CsResult<Vec<(Chunk, Vec<f32>)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT file, line_start, line_end, symbol, kind, language, text_hash, embedding FROM chunks ORDER BY id",
            )
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("chunks 遍历失败: {e}")))?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)? as usize,
                    r.get::<_, i64>(2)? as usize,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, Vec<u8>>(7)?,
                ))
            })
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("chunks 遍历失败: {e}")))?;
        let mut out = Vec::new();
        for row in rows {
            let (file, ls, le, symbol, kind, language, text_hash, blob) =
                row.map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("{e}")))?;
            let vector = blob_to_vec(&blob);
            let header = crate::vector::chunk::breadcrumb(&file, &symbol, &kind);
            let text = String::new(); // 载入检索时不携带原文（原文指针即可）
            out.push((
                Chunk {
                    file,
                    line_start: ls,
                    line_end: le,
                    symbol,
                    kind,
                    language,
                    header,
                    text,
                    text_hash,
                },
                vector,
            ));
        }
        Ok(out)
    }
}

/// 余弦相似度（归一化由调用方负责时亦适用：先做点积/模长）。
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let (mut dot, mut na, mut nb) = (0.0f32, 0.0f32, 0.0f32);
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}

/// 索引文件标准路径：状态目录/indexes/<fingerprint>/vectors.db。
/// 项目本地索引目录（2026-10-05 用户拍板：数据库迁到项目里跟着走，不再集中放全局状态目录）。
/// .codesleuth/ 与 .codegraph/ 同待遇（writeguard SKIP_DIRS 豁免，D011 式工具元数据）。
pub fn project_index_dir(repo_abs: &Path) -> std::path::PathBuf {
    repo_abs.join(".codesleuth")
}

pub fn index_path(state_dir: &Path, fp: &str) -> std::path::PathBuf {
    state_dir.join("indexes").join(fp).join("vectors.db")
}

/// 便捷：内容哈希（与 chunk.text_hash 同口径）。
pub fn hash_of(text: &str) -> String {
    sha256_hex(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_chunk(file: &str, symbol: &str, ls: usize, body: &str) -> Chunk {
        let text = format!("{file} › {symbol}\n{body}");
        let text_hash = sha256_hex(&text);
        Chunk {
            file: file.into(),
            line_start: ls,
            line_end: ls + 5,
            symbol: symbol.into(),
            kind: "function".into(),
            language: "rust".into(),
            header: "header".into(),
            text,
            text_hash,
        }
    }

    #[test]
    fn remove_stale_deletes_vectors_and_descriptions() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("vectors.db");
        let store = VectorStore::open(&db).unwrap();
        let c1 = mk_chunk("a.rs", "keep_me", 1, "fn keep() {}");
        let c2 = mk_chunk("a.rs", "dead_fn", 20, "fn dead() {}");
        store.upsert_chunk(&c1, &[0.5f32, 0.5]).unwrap();
        store.upsert_chunk(&c2, &[0.25f32, 0.75]).unwrap();
        let k2 = chunk_key(&c2);
        assert_eq!(store.count().unwrap(), 2);

        let removed = store.remove_stale(std::slice::from_ref(&k2)).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(store.count().unwrap(), 1);
        // 幂等：再删一次不报错也不多删
        assert_eq!(store.remove_stale(&[k2]).unwrap(), 1);
        assert_eq!(store.count().unwrap(), 1);
    }

    #[test]
    fn roundtrip_and_incremental() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("vectors.db");
        let store = VectorStore::open(&db).unwrap();

        let c1 = mk_chunk("src/a.rs", "alpha", 1, "alpha body");
        let c2 = mk_chunk("src/a.rs", "beta", 10, "beta body");
        let v = vec![0.1f32, 0.2, 0.3];
        store.upsert_chunk(&c1, &v).unwrap();
        store.upsert_chunk(&c2, &v).unwrap();
        assert_eq!(store.count().unwrap(), 2);

        // 增量：已有 hash 集合命中 → 复用，不再嵌入
        let hashes = store.existing_hashes().unwrap();
        assert_eq!(hashes.get(&chunk_key(&c1)).unwrap(), &c1.text_hash);

        // 覆盖写（同 key 更新）
        let v2 = vec![0.4f32, 0.5, 0.6];
        store.upsert_chunk(&c1, &v2).unwrap();
        assert_eq!(store.count().unwrap(), 2);

        let all = store.load_all().unwrap();
        let alpha = all.iter().find(|(c, _)| c.symbol == "alpha").unwrap();
        assert_eq!(alpha.1, vec![0.4, 0.5, 0.6]);
        assert_eq!(alpha.0.text_hash, c1.text_hash);
    }

    #[test]
    fn meta_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = VectorStore::open(&dir.path().join("v.db")).unwrap();
        assert_eq!(store.get_meta("model").unwrap(), None);
        store.set_meta("model", "Qwen/Qwen3-Embedding-8B").unwrap();
        assert_eq!(
            store.get_meta("model").unwrap().as_deref(),
            Some("Qwen/Qwen3-Embedding-8B")
        );
    }

    #[test]
    fn cosine_basics() {
        assert!((cosine(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 1e-6);
        assert!(cosine(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-6);
        assert_eq!(cosine(&[], &[]), 0.0);
    }

    #[test]
    fn fingerprint_deterministic_and_path_sensitive() {
        let a = fingerprint(Path::new("/tmp/repo-a"));
        let a2 = fingerprint(Path::new("/tmp/repo-a"));
        let b = fingerprint(Path::new("/tmp/repo-b"));
        assert_eq!(a, a2);
        assert_ne!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn blob_roundtrip_preserves_floats() {
        let v = vec![0.25f32, -1.5, 3.25e-4];
        assert_eq!(blob_to_vec(&vec_to_blob(&v)), v);
    }
}
