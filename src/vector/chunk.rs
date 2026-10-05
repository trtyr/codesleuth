//! 切块（P003 E1a · Q2 已定）：codegraph 符号表为边界真源 + 三层规则。
//!
//! 层1 符号主块：顶层符号（装饰器/签名/docstring/函数体），embedding 前加面包屑。
//! 层2 超大滑窗：超 MAX_CHUNK_CHARS 的符号体内按 SUB_WINDOW_LINES 切子块，重拼面包屑。
//! 层3 兜底：非源码文件（README 标题节 / 其他按窗口）+ codegraph 覆盖不到的 leftover 行。

use crate::errors::{CsError, CsResult, INDEX_NOT_AVAILABLE};
use rusqlite::OptionalExtension;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

/// 块字符上限（≈1500 token）。
pub const MAX_CHUNK_CHARS: usize = 6000;
/// 超大符号的子窗口行数。
pub const SUB_WINDOW_LINES: usize = 100;
/// 子窗口重叠行数。
pub const SUB_OVERLAP_LINES: usize = 15;
/// 兜底窗口行数。
pub const FALLBACK_WINDOW_LINES: usize = 100;
/// 兜底窗口重叠行数。
pub const FALLBACK_OVERLAP_LINES: usize = 20;
/// 作为主块保留的节点 kind（file=容器丢弃、import/variable 落 leftover）。
pub const KEPT_KINDS: &[&str] = &[
    "function",
    "method",
    "struct",
    "class",
    "interface",
    "impl",
    "trait",
];
/// 索引排除目录。
pub const SKIP_DIRS: &[&str] = &[
    ".git",
    ".codegraph",
    "target",
    "node_modules",
    "dist",
    "__pycache__",
    ".venv",
];
/// 索引排除扩展名。
pub const SKIP_EXTS: &[&str] = &[
    "lock", "png", "jpg", "jpeg", "gif", "ico", "pdf", "zip", "bin", "woff", "woff2",
];
/// 文件大小上限（超过跳过）。
pub const MAX_FILE_BYTES: u64 = 400_000;

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolRow {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub language: String,
    pub docstring: Option<String>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
    pub symbol: String,
    pub kind: String,
    pub language: String,
    /// 白拿层：面包屑 + 签名 + docstring（不含函数体）——复合体组装的原料。
    pub header: String,
    pub text: String,
    pub text_hash: String,
}

pub fn sha256_hex(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// 查询符号的关系上下文（callers / callees，节点名去重封顶 8）。db 不可用返回错误（调用方降级）。
pub fn relations_for_symbol(
    db_path: &Path,
    file_path: &str,
    symbol: &str,
) -> CsResult<(Vec<String>, Vec<String>)> {
    let conn =
        rusqlite::Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("codegraph 库打开失败: {e}")))?;
    let node_id: Option<String> = conn
        .query_row(
            "SELECT id FROM nodes WHERE file_path = ?1 AND name = ?2 ORDER BY start_line LIMIT 1",
            rusqlite::params![file_path, symbol],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("节点查询失败: {e}")))?;
    let Some(id) = node_id else {
        return Ok((Vec::new(), Vec::new()));
    };
    let mut callers = Vec::new();
    {
        let mut st = conn
            .prepare(
                "SELECT DISTINCT n.name FROM edges e JOIN nodes n ON e.source = n.id
                 WHERE e.target = ?1 LIMIT 8",
            )
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("callers 查询失败: {e}")))?;
        let rows = st
            .query_map([&id], |r| r.get::<_, String>(0))
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("callers 遍历失败: {e}")))?;
        for r in rows {
            callers.push(r.map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("{e}")))?);
        }
    }
    let mut callees = Vec::new();
    {
        let mut st = conn
            .prepare(
                "SELECT DISTINCT n.name FROM edges e JOIN nodes n ON e.target = n.id
                 WHERE e.source = ?1 LIMIT 8",
            )
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("callees 查询失败: {e}")))?;
        let rows = st
            .query_map([&id], |r| r.get::<_, String>(0))
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("callees 遍历失败: {e}")))?;
        for r in rows {
            callees.push(r.map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("{e}")))?);
        }
    }
    Ok((callers, callees))
}

/// 从 codegraph SQLite（只读）读全部节点。
pub fn symbols_from_codegraph(db_path: &Path) -> CsResult<Vec<SymbolRow>> {
    let conn =
        rusqlite::Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| {
                CsError::new(
                    INDEX_NOT_AVAILABLE,
                    format!("codegraph 库打开失败 {}: {e}", db_path.display()),
                )
            })?;
    let mut stmt = conn
        .prepare(
            "SELECT name, kind, file_path, start_line, end_line, language, docstring, signature FROM nodes",
        )
        .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("nodes 表查询失败: {e}")))?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SymbolRow {
                name: r.get::<_, String>(0)?,
                kind: r.get::<_, String>(1)?,
                file_path: r.get::<_, String>(2)?,
                start_line: r.get::<_, i64>(3)?.max(1) as usize,
                end_line: r.get::<_, i64>(4)?.max(1) as usize,
                language: r.get::<_, String>(5)?,
                docstring: r.get::<_, Option<String>>(6)?,
                signature: r.get::<_, Option<String>>(7)?,
            })
        })
        .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("nodes 表遍历失败: {e}")))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub(crate) fn read_lines(path: &Path) -> Option<Vec<String>> {
    let meta = std::fs::metadata(path).ok()?;
    if meta.len() > MAX_FILE_BYTES {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    Some(content.lines().map(|l| l.to_string()).collect())
}

pub(crate) fn breadcrumb(file: &str, symbol: &str, kind: &str) -> String {
    format!("{file} › {symbol}（{kind}）")
}

/// chunk 构造（内部；字段多系切块产物固有属性）。
#[allow(clippy::too_many_arguments)]
fn make_chunk(
    file: &str,
    line_start: usize,
    line_end: usize,
    symbol: &str,
    kind: &str,
    language: &str,
    header: String,
    text: String,
) -> Chunk {
    let text_hash = sha256_hex(&text);
    Chunk {
        file: file.to_string(),
        line_start,
        line_end,
        symbol: symbol.to_string(),
        kind: kind.to_string(),
        language: language.to_string(),
        header,
        text,
        text_hash,
    }
}

/// 白拿层组装：面包屑 + 签名 + docstring。返回 (header, header 副本)。
fn symbol_parts(sym: &SymbolRow) -> (String, String) {
    let mut parts = vec![breadcrumb(&sym.file_path, &sym.name, &sym.kind)];
    if let Some(sig) = &sym.signature {
        parts.push(sig.clone());
    }
    if let Some(doc) = &sym.docstring {
        parts.push(doc.clone());
    }
    let header = parts.join("\n");
    (header.clone(), header)
}

/// 符号主块文本：header + 函数体。
fn symbol_text(sym: &SymbolRow, body: &str) -> String {
    let (header, _) = symbol_parts(sym);
    format!("{header}\n{body}")
}

/// 超大符号体切子窗，每窗重拼面包屑。
fn sub_window_chunks(sym: &SymbolRow, body_lines: &[String]) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < body_lines.len() {
        let end = (i + SUB_WINDOW_LINES).min(body_lines.len());
        let text = format!(
            "{}\n{}",
            breadcrumb(&sym.file_path, &sym.name, &sym.kind),
            body_lines[i..end].join("\n")
        );
        out.push((i, end, text));
        if end == body_lines.len() {
            break;
        }
        i = end.saturating_sub(SUB_OVERLAP_LINES);
    }
    out
}

/// 是否注释行（docstring 上提用）。
fn is_comment_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("///")
        || t.starts_with("//")
        || t.starts_with("#")
        || t.starts_with("*")
        || t.starts_with("/*")
        || t.starts_with("\"\"\"")
}

/// 主块 + leftover + 兜底的编排。nodes 全量（含 file/import/variable），内部自行分层。
pub fn plan_chunks(repo_root: &Path, nodes: &[SymbolRow]) -> CsResult<Vec<Chunk>> {
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut covered: BTreeMap<String, Vec<(usize, usize)>> = BTreeMap::new();
    let mut codegraph_files: HashSet<String> = HashSet::new();

    // 按文件分组
    let mut by_file: BTreeMap<String, Vec<&SymbolRow>> = BTreeMap::new();
    for n in nodes {
        by_file.entry(n.file_path.clone()).or_default().push(n);
        if n.kind == "file" {
            codegraph_files.insert(n.file_path.clone());
        }
    }

    for (file, syms) in &by_file {
        let abs = repo_root.join(file);
        let Some(lines) = read_lines(&abs) else {
            continue;
        };
        let lang = syms.first().map(|s| s.language.clone()).unwrap_or_default();

        // 容器丢弃 + 文档行上提
        let mut kept: Vec<(&SymbolRow, usize, usize)> = Vec::new(); // (sym, eff_start, eff_end)
        for sym in syms
            .iter()
            .filter(|s| KEPT_KINDS.contains(&s.kind.as_str()))
        {
            let start = sym.start_line.max(1).min(lines.len().max(1));
            let end = sym.end_line.min(lines.len()).max(start);
            if kept.iter().any(|(_, ks, ke)| start >= *ks && end <= *ke) {
                continue; // 容器/被包含
            }
            kept.retain(|(_, ks, ke)| !(ks >= &start && ke <= &end));
            // docstring/注释行上提（最多 20 行）
            let mut eff = start;
            let mut walked = 0;
            while eff > 1 && walked < 20 {
                let prev = &lines[eff - 2];
                if prev.trim().is_empty() || !is_comment_line(prev) {
                    break;
                }
                eff -= 1;
                walked += 1;
            }
            kept.push((sym, eff, end));
        }
        kept.sort_by_key(|(_, s, _)| *s);

        for (sym, eff_start, end) in &kept {
            let body = lines[eff_start.saturating_sub(1)..*end].join("\n");
            let full = symbol_text(sym, &body);
            let (header, _) = symbol_parts(sym);
            if full.len() <= MAX_CHUNK_CHARS {
                chunks.push(make_chunk(
                    file,
                    *eff_start,
                    *end,
                    &sym.name,
                    &sym.kind,
                    &lang,
                    header.clone(),
                    full,
                ));
            } else {
                let body_lines: Vec<String> = lines[eff_start.saturating_sub(1)..*end].to_vec();
                for (i, e, text) in sub_window_chunks(sym, &body_lines) {
                    chunks.push(make_chunk(
                        file,
                        eff_start + i,
                        eff_start + e - 1,
                        &sym.name,
                        &sym.kind,
                        &lang,
                        header.clone(),
                        text,
                    ));
                }
            }
            covered
                .entry(file.clone())
                .or_default()
                .push((*eff_start, *end));
        }

        // leftover：codegraph 已知文件里未被覆盖的行（≥3 非空行才算）
        let spans = covered.entry(file.clone()).or_default();
        spans.sort();
        let mut merged: Vec<(usize, usize)> = Vec::new();
        for (s, e) in spans.iter().copied() {
            match merged.last_mut() {
                Some(last) if s <= last.1.saturating_add(1) => last.1 = last.1.max(e),
                _ => merged.push((s, e)),
            }
        }
        let mut gaps: Vec<(usize, usize)> = Vec::new();
        let mut cursor = 1;
        for (s, e) in merged.iter().copied() {
            if s > cursor {
                gaps.push((cursor, s - 1));
            }
            cursor = cursor.max(e + 1);
        }
        if lines.len() >= cursor {
            gaps.push((cursor, lines.len()));
        }
        for (gs, ge) in gaps {
            let slice = &lines[gs.saturating_sub(1)..ge];
            if slice.iter().filter(|l| !l.trim().is_empty()).count() < 2 {
                continue;
            }
            let header = breadcrumb(file, "<leftover>", "leftover");
            let text = format!("{header}\n{}", slice.join("\n"));
            chunks.push(make_chunk(
                file,
                gs,
                ge,
                "<leftover>",
                "leftover",
                &lang,
                header,
                text,
            ));
        }
    }

    // 兜底：磁盘上存在但 codegraph 未覆盖的文本文件
    let walk_root = repo_root.to_path_buf();
    let mut stack = vec![walk_root.clone()];
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if p.is_dir() {
                if !SKIP_DIRS.contains(&name.as_str()) {
                    stack.push(p);
                }
            } else if let Some(ext) = p.extension().map(|x| x.to_string_lossy().to_string()) {
                if !SKIP_EXTS.contains(&ext.as_str()) {
                    files.push(p);
                }
            } else {
                files.push(p);
            }
        }
    }
    for p in files {
        let Ok(rel) = p.strip_prefix(&walk_root) else {
            continue;
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        if codegraph_files.contains(&rel) || covered.contains_key(&rel) {
            continue; // codegraph 已覆盖
        }
        let Some(lines) = read_lines(&p) else {
            continue;
        };
        if lines.is_empty() {
            continue; // 空文件不切块（E3 x-ray 实战：空 md 触发 fallback 切片越界）
        }
        let is_md = rel.ends_with(".md");
        let mut sections: Vec<(usize, usize)> = Vec::new();
        if is_md {
            let mut start = 0;
            for (i, l) in lines.iter().enumerate() {
                if i > 0 && l.starts_with('#') {
                    if i - start > 0 {
                        sections.push((start, i - 1));
                    }
                    start = i;
                }
            }
            sections.push((start, lines.len().saturating_sub(1)));
        } else {
            let mut i = 0;
            while i < lines.len() {
                let end = (i + FALLBACK_WINDOW_LINES).min(lines.len());
                sections.push((i, end - 1));
                if end == lines.len() {
                    break;
                }
                i = end.saturating_sub(FALLBACK_OVERLAP_LINES);
            }
        }
        let lang = if is_md {
            "markdown".to_string()
        } else {
            "text".to_string()
        };
        for (s, e) in sections {
            let body = lines[s..=e].join("\n");
            if body.trim().is_empty() {
                continue;
            }
            let header = breadcrumb(&rel, "<fallback>", "fallback");
            let text = format!("{header}\n{body}");
            chunks.push(make_chunk(
                &rel,
                s + 1,
                e + 1,
                "<fallback>",
                "fallback",
                &lang,
                header,
                text,
            ));
        }
    }

    chunks.sort_by(|a, b| {
        (&a.file, a.line_start, &a.symbol).cmp(&(&b.file, b.line_start, &b.symbol))
    });
    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn write_file(dir: &Path, rel: &str, content: &str) -> PathBuf {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, content).unwrap();
        p
    }

    fn sym(name: &str, kind: &str, file: &str, s: usize, e: usize, lang: &str) -> SymbolRow {
        SymbolRow {
            name: name.into(),
            kind: kind.into(),
            file_path: file.into(),
            start_line: s,
            end_line: e,
            language: lang.into(),
            docstring: None,
            signature: None,
        }
    }

    #[test]
    fn symbol_chunks_with_breadcrumb_and_container_drop() {
        let dir = tempfile::tempdir().unwrap();
        let src = "use std::io;\n\nconst MAX: u32 = 3;\n\nfn core() {\n    body\n}\n\nfn helper() {\n    h\n}\n";
        write_file(dir.path(), "src/lib.rs", src);
        // file 节点（容器，应被丢）+ 两个函数
        let nodes = vec![
            sym("lib.rs", "file", "src/lib.rs", 1, 9, "rust"),
            sym("core", "function", "src/lib.rs", 5, 7, "rust"),
            sym("helper", "function", "src/lib.rs", 9, 11, "rust"),
        ];
        let chunks = plan_chunks(dir.path(), &nodes).unwrap();
        let main: Vec<_> = chunks.iter().filter(|c| c.symbol != "<leftover>").collect();
        assert_eq!(main.len(), 2, "file 容器节点必须被丢: {chunks:?}");
        assert!(main[0].text.contains("src/lib.rs › core（function）"));
        assert!(main[0].text.contains("body"));
        assert_eq!(main[0].text_hash, sha256_hex(&main[0].text));
        // leftover：1-4 行（import + MAX）未覆盖 → leftover 块
        let leftover: Vec<_> = chunks.iter().filter(|c| c.kind == "leftover").collect();
        assert!(
            !leftover.is_empty(),
            "应有 leftover 块覆盖 MAX 行: {chunks:?}"
        );
        assert!(leftover[0].text.contains("MAX"));
    }

    #[test]
    fn oversize_symbol_splits_into_subwindows_with_breadcrumb() {
        let dir = tempfile::tempdir().unwrap();
        // 行宽 ~45 字符 × 300 行 ≈ 13.5k 字符 > MAX_CHUNK_CHARS(6000)，必触发滑窗
        let body: String = (1..=300)
            .map(|i| format!("line {i} = {i} with some padding to grow the size quickly\n"))
            .collect();
        let src = format!("fn big() {{\n{body}}}\n");
        write_file(dir.path(), "src/big.rs", &src);
        let nodes = vec![sym("big", "function", "src/big.rs", 1, 302, "rust")];
        let chunks = plan_chunks(dir.path(), &nodes).unwrap();
        let main: Vec<_> = chunks.iter().filter(|c| c.symbol == "big").collect();
        assert!(main.len() >= 3, "超大符号应切多块: {}", main.len());
        assert!(
            main.iter()
                .all(|c| c.text.contains("src/big.rs › big（function）"))
        );
        assert!(main.iter().all(|c| c.text.len() <= MAX_CHUNK_CHARS + 200));
    }

    #[test]
    fn fallback_markdown_header_split_for_uncovered_files() {
        let dir = tempfile::tempdir().unwrap();
        write_file(
            dir.path(),
            "README.md",
            "# Title\nintro\n\n## 使用\nusage text\n\n## 架构\narch text\n",
        );
        let chunks = plan_chunks(dir.path(), &[]).unwrap();
        let md: Vec<_> = chunks.iter().filter(|c| c.file == "README.md").collect();
        assert!(md.len() >= 2, "README 应按标题切节: {chunks:?}");
        assert!(md.iter().any(|c| c.text.contains("usage text")));
    }

    #[test]
    fn docstring_lines_above_symbol_are_absorbed() {
        let dir = tempfile::tempdir().unwrap();
        let src = "/// 装饰器：重试逻辑。\n/// 更多说明。\ndef retry():\n    pass\n";
        write_file(dir.path(), "src/mod.py", src);
        let nodes = vec![sym("retry", "function", "src/mod.py", 3, 4, "python")];
        let chunks = plan_chunks(dir.path(), &nodes).unwrap();
        let c = chunks.iter().find(|c| c.symbol == "retry").unwrap();
        assert!(
            c.text.contains("装饰器：重试逻辑。"),
            "docstring 应上提吸收: {}",
            c.text
        );
        assert_eq!(c.line_start, 1);
    }

    #[test]
    fn codegraph_sqlite_adapter_reads_nodes() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("codegraph.db");
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes (id TEXT PRIMARY KEY, kind TEXT NOT NULL, name TEXT NOT NULL, qualified_name TEXT NOT NULL, file_path TEXT NOT NULL, language TEXT NOT NULL, start_line INTEGER NOT NULL, end_line INTEGER NOT NULL, start_column INTEGER DEFAULT 0, end_column INTEGER DEFAULT 0, docstring TEXT, signature TEXT, visibility TEXT, is_exported INTEGER DEFAULT 0, is_async INTEGER DEFAULT 0, is_static INTEGER DEFAULT 0, is_abstract INTEGER DEFAULT 0, decorators TEXT, type_parameters TEXT, return_type TEXT, updated_at INTEGER NOT NULL);
             INSERT INTO nodes (id, kind, name, qualified_name, file_path, language, start_line, end_line, updated_at) VALUES ('n1', 'function', 'retry', 'retry', 'src/a.py', 'python', 3, 10, 1);",
        )
        .unwrap();
        let rows = symbols_from_codegraph(&db).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "retry");
        assert_eq!(rows[0].start_line, 3);
        // 不存在的库 → 结构化错误
        let err = symbols_from_codegraph(&dir.path().join("nope.db")).unwrap_err();
        assert_eq!(err.code, crate::errors::INDEX_NOT_AVAILABLE);
    }
}
