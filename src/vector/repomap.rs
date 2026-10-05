//! repo map 预算化注入（P003 E2 · 设计稿 §2）：codegraph 符号 + 边表度数中心度排序，
//! token 预算（字符预算 ≈ 6k token × 4）封顶贪心装填，小仓全图、大仓头部。
//! 注入位置 = system prompt 尾部「[repo map]」段（稳定事实性内容 + 查证指引）。

use crate::errors::{CsError, CsResult, INDEX_NOT_AVAILABLE};
use std::collections::HashMap;
use std::path::Path;

/// 默认字符预算（≈6k token）。
pub const DEFAULT_BUDGET_CHARS: usize = 24_000;
/// 单符号行预算上限（防止超长 qualified name 吃光预算）。
pub const MAX_LINE_CHARS: usize = 200;

pub struct MapSymbol {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub start_line: i64,
}

/// 从 codegraph SQLite 读符号（排除 file/import 等容器型节点）与边表度数。
pub fn repo_map_inputs(db_path: &Path) -> CsResult<(Vec<MapSymbol>, HashMap<String, usize>)> {
    let conn =
        rusqlite::Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("codegraph 库打开失败: {e}")))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, kind, file_path, start_line FROM nodes
             WHERE kind IN ('function','method','struct','class','interface','impl','trait')",
        )
        .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("nodes 查询失败: {e}")))?;
    let symbols: Vec<MapSymbol> = stmt
        .query_map([], |r| {
            Ok(MapSymbol {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: r.get(2)?,
                file_path: r.get(3)?,
                start_line: r.get(4)?,
            })
        })
        .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("nodes 遍历失败: {e}")))?
        .filter_map(|r| r.ok())
        .collect();
    let mut degrees: HashMap<String, usize> = HashMap::new();
    let mut est = conn
        .prepare("SELECT source, target FROM edges")
        .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("edges 查询失败: {e}")))?;
    let rows = est
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("edges 遍历失败: {e}")))?;
    for row in rows {
        let (s, t) = row.map_err(|e| CsError::new(INDEX_NOT_AVAILABLE, format!("{e}")))?;
        *degrees.entry(s).or_default() += 1;
        *degrees.entry(t).or_default() += 1;
    }
    Ok((symbols, degrees))
}

/// 任务相关导航图（2026-10-04 用户立项）：召回命中优先 → 一跳邻居 → 度数填充。
/// seeds = 召回命中的符号名（相似度序）；neighbors = 命中符号的 callers/callees 名集合。
/// 空 seeds + 空 neighbors 时与 build_repo_map 等价。◈=召回命中 ◇=一跳邻居。
pub fn build_task_map(
    symbols: &[MapSymbol],
    degrees: &HashMap<String, usize>,
    budget_chars: usize,
    seeds: &[String],
    neighbors: &std::collections::HashSet<String>,
) -> String {
    let seed_rank: HashMap<&str, usize> = seeds
        .iter()
        .enumerate()
        .map(|(i, s)| (s.as_str(), i))
        .collect();
    let mut entries: Vec<&MapSymbol> = symbols.iter().collect();
    entries.sort_by(|a, b| {
        let ra = seed_rank
            .get(a.name.as_str())
            .copied()
            .unwrap_or(usize::MAX);
        let rb = seed_rank
            .get(b.name.as_str())
            .copied()
            .unwrap_or(usize::MAX);
        let da = degrees.get(&a.id).copied().unwrap_or(0);
        let db = degrees.get(&b.id).copied().unwrap_or(0);
        ra.cmp(&rb)
            .then_with(|| {
                neighbors
                    .contains(&b.name)
                    .cmp(&neighbors.contains(&a.name))
            })
            .then_with(|| db.cmp(&da))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.file_path.cmp(&b.file_path))
    });
    let mut out = String::new();
    let mut used = 0usize;
    let mut included = 0usize;
    for s in &entries {
        let deg = degrees.get(&s.id).copied().unwrap_or(0);
        let tag = if seed_rank.contains_key(s.name.as_str()) {
            " ◈"
        } else if neighbors.contains(&s.name) {
            " ◇"
        } else {
            ""
        };
        let line = format!(
            "{}:{} {}（{}，度 {}）{}\n",
            s.file_path, s.start_line, s.name, s.kind, deg, tag
        );
        let line_len = line.len().min(MAX_LINE_CHARS);
        if used + line_len > budget_chars {
            break;
        }
        out.push_str(&line);
        used += line_len;
        included += 1;
    }
    if included < entries.len() {
        out.push_str(&format!(
            "…共 {} 符号，其余未列入（详情用 explore / files 查）\n",
            entries.len()
        ));
    }
    out
}

/// 生成 repo map 文本（预算内贪心装填，确定性排序：度数降序 → 名称升序）。
pub fn build_repo_map(
    symbols: &[MapSymbol],
    degrees: &HashMap<String, usize>,
    budget_chars: usize,
) -> String {
    let mut entries: Vec<&MapSymbol> = symbols.iter().collect();
    entries.sort_by(|a, b| {
        let da = degrees.get(&a.id).copied().unwrap_or(0);
        let db = degrees.get(&b.id).copied().unwrap_or(0);
        db.cmp(&da)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.file_path.cmp(&b.file_path))
    });
    let mut out = String::new();
    let mut used = 0usize;
    let mut included = 0usize;
    for s in &entries {
        let deg = degrees.get(&s.id).copied().unwrap_or(0);
        let line = format!(
            "{}:{} {}（{}，度 {}）\n",
            s.file_path, s.start_line, s.name, s.kind, deg
        );
        let line_len = line.len().min(MAX_LINE_CHARS);
        if used + line_len > budget_chars {
            break;
        }
        out.push_str(&line);
        used += line_len;
        included += 1;
    }
    if included < entries.len() {
        out.push_str(&format!(
            "…共 {} 符号，其余未列入（详情用 explore / files 查）\n",
            entries.len()
        ));
    }
    if out.is_empty() {
        out.push_str("（仓库无符号索引）\n");
    }
    out
}

/// system prompt 注入段（含查证指引）。
pub fn wrap_repo_section(map: &str) -> String {
    format!("[repo map]\n{map}\n结构详情勿凭此图推断，用 explore / callers / callees 查证。")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sym(id: &str, name: &str, file: &str, line: i64) -> MapSymbol {
        MapSymbol {
            id: id.into(),
            name: name.into(),
            kind: "function".into(),
            file_path: file.into(),
            start_line: line,
        }
    }

    #[test]
    fn task_map_seeds_first_then_neighbors_then_degree() {
        let symbols = vec![
            sym("a", "hot_target", "src/a.rs", 10),
            sym("b", "its_caller", "src/b.rs", 5),
            sym("c", "unrelated_big", "src/c.rs", 1),
        ];
        let mut degrees = HashMap::new();
        degrees.insert("c".to_string(), 99);
        degrees.insert("a".to_string(), 3);
        degrees.insert("b".to_string(), 1);
        let mut neighbors = std::collections::HashSet::new();
        neighbors.insert("its_caller".to_string());
        let map = build_task_map(
            &symbols,
            &degrees,
            10_000,
            &["hot_target".to_string()],
            &neighbors,
        );
        let lines: Vec<&str> = map.lines().collect();
        assert!(
            lines[0].contains("hot_target") && lines[0].contains("◈"),
            "种子第一行带◈: {map}"
        );
        assert!(
            lines[1].contains("its_caller") && lines[1].contains("◇"),
            "邻居第二行带◇: {map}"
        );
        assert!(lines[2].contains("unrelated_big"), "度数填充殿后: {map}");
    }

    #[test]
    fn task_map_empty_seeds_equals_global() {
        let symbols = vec![
            sym("a", "zeta", "src/a.rs", 1),
            sym("b", "alpha", "src/b.rs", 1),
        ];
        let mut degrees = HashMap::new();
        degrees.insert("b".to_string(), 7);
        let empty = std::collections::HashSet::new();
        assert_eq!(
            build_task_map(&symbols, &degrees, 10_000, &[], &empty),
            build_repo_map(&symbols, &degrees, 10_000)
        );
    }

    #[test]
    fn ranked_by_degree_desc_then_name() {
        let symbols = vec![
            sym("a", "zeta", "src/a.rs", 1),
            sym("b", "alpha", "src/b.rs", 1),
            sym("c", "mid", "src/c.rs", 1),
        ];
        let mut degrees = HashMap::new();
        degrees.insert("b".to_string(), 5);
        degrees.insert("a".to_string(), 2);
        // c 度数 0
        let map = build_repo_map(&symbols, &degrees, 10_000);
        let lines: Vec<&str> = map.lines().collect();
        assert!(lines[0].contains("alpha"), "高度数排前: {map}");
        assert!(lines[1].contains("zeta"));
        assert!(lines[2].contains("mid"));
    }

    #[test]
    fn budget_cap_and_overflow_note() {
        let symbols: Vec<MapSymbol> = (0..100)
            .map(|i| {
                sym(
                    &format!("id{i}"),
                    &format!("sym_{i:03}"),
                    &format!("src/f{i}.rs"),
                    1,
                )
            })
            .collect();
        let map = build_repo_map(&symbols, &HashMap::new(), 2_000);
        assert!(map.contains("其余未列入"));
        assert!(map.len() < 2_000 + 300);
    }

    #[test]
    fn small_repo_full_coverage_no_note() {
        let symbols = vec![sym("a", "only_one", "src/a.rs", 1)];
        let map = build_repo_map(&symbols, &HashMap::new(), 10_000);
        assert!(!map.contains("其余未列入"));
        assert!(map.contains("only_one"));
    }

    #[test]
    fn wrap_section_has_verification_hint() {
        let s = wrap_repo_section("src/a.rs:1 foo（function，度 3）");
        assert!(s.starts_with("[repo map]"));
        assert!(s.contains("explore"));
    }

    #[test]
    fn repo_map_inputs_read_real_codegraph() {
        let db = std::path::Path::new("tests/fixtures/fixture-rs/.codegraph/codegraph.db");
        if !db.exists() {
            return; // CI 无库时跳过
        }
        let (symbols, degrees) = repo_map_inputs(db).unwrap();
        assert!(!symbols.is_empty());
        assert!(symbols.iter().any(|s| s.name == "retry_with_backoff"));
        let _ = degrees;
    }
}
