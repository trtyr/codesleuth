//! 模糊检索层（fff-search 封装，参考 dmtrKovalenko/fff）：
//! find_files = frecency 模糊路径搜索；grep = plain/regex/fuzzy 三态内容检索。
//! 注意：禁用 fff 的 `zlob` feature（需要 zig），纯 Rust fallback 保证 cargo install 可移植。

use crate::errors::{CsError, CsResult, INDEX_BUILD_FAILED, USER_INPUT};
use crate::fence::Fence;
use crate::tools::Tool;
use async_trait::async_trait;
use fff_search::{
    FFFMode, FilePicker, FilePickerOptions, FuzzySearchOptions, GrepMode, GrepSearchOptions,
    PaginationArgs, QueryParser,
};
use serde_json::Value;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// 会话级模糊检索引擎：一个 picker 服务整个会话（watch=false，同步收集，无后台线程）。
pub struct FuzzyEngine {
    picker: FilePicker,
    fence: Arc<Fence>,
}

impl FuzzyEngine {
    pub fn new(root: &Path) -> CsResult<Self> {
        let fence = Fence::new(root)?;
        let mut picker = FilePicker::new(FilePickerOptions {
            base_path: fence.root().to_string_lossy().into_owned(),
            watch: false,
            mode: FFFMode::Ai,
            ..Default::default()
        })
        .map_err(|e| CsError::new(INDEX_BUILD_FAILED, format!("fff 索引初始化失败: {e}")))?;
        picker
            .collect_files()
            .map_err(|e| CsError::new(INDEX_BUILD_FAILED, format!("fff 文件收集失败: {e}")))?;
        Ok(Self {
            picker,
            fence: Arc::new(fence),
        })
    }

    pub fn fence(&self) -> &Fence {
        &self.fence
    }
}

/// find_files：模糊路径搜索（层进协议第 2 层「顺藤摸点」的定位手段）。
pub struct FileFinderTool {
    engine: Arc<FuzzyEngine>,
}

impl FileFinderTool {
    pub fn new(engine: Arc<FuzzyEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl Tool for FileFinderTool {
    fn name(&self) -> &'static str {
        "find_files"
    }

    fn description(&self) -> String {
        "按名字模糊找文件（frecency 排序）。args: {query: 文件名/路径片段, limit?: 数量(默认20)}"
            .into()
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "required": ["query"],
            "properties": {
                "query": {"type": "string", "description": "文件名或路径片段（支持约束语法）"},
                "limit": {"type": "integer", "description": "返回条数（默认 20，上限 100）"}
            }
        })
    }

    async fn execute(&self, args: Value) -> CsResult<String> {
        let query_str = args
            .get("query")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| CsError::new(USER_INPUT, "find_files 缺少 query 参数"))?
            .to_string();
        let limit = args
            .get("limit")
            .and_then(Value::as_u64)
            .unwrap_or(20)
            .clamp(1, 100) as usize;

        let query = QueryParser::default().parse(&query_str);
        let opts = FuzzySearchOptions {
            max_threads: 0,
            pagination: PaginationArgs { offset: 0, limit },
            ..Default::default()
        };
        let result = self.engine.picker.fuzzy_search(&query, None, opts);

        let mut out = format!(
            "[find_files] \"{query_str}\" · 命中 {}/{}\n",
            result.total_matched, result.total_files
        );
        for (i, item) in result.items.iter().enumerate() {
            out.push_str(&format!(
                "{}. {}\n",
                i + 1,
                item.relative_path(&self.engine.picker)
            ));
        }
        if result.total_matched > result.items.len() {
            out.push_str("…（已截断，可提高 limit）\n");
        }
        Ok(out)
    }
}

/// grep：内容检索（plain/regex/fuzzy 三态，零命中自动降级由 fff 处理）。
pub struct GrepTool {
    engine: Arc<FuzzyEngine>,
}

impl GrepTool {
    pub fn new(engine: Arc<FuzzyEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &'static str {
        "grep"
    }

    fn description(&self) -> String {
        "在仓库内检索内容。args: {pattern: 检索串, mode?: plain(默认)|regex|fuzzy, limit?: 命中数上限(默认50,上限200)}".into()
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "required": ["pattern"],
            "properties": {
                "pattern": {"type": "string", "description": "检索内容"},
                "mode": {"type": "string", "enum": ["plain", "regex", "fuzzy"], "description": "默认 plain"},
                "limit": {"type": "integer", "description": "命中数上限（默认 50，上限 200）"}
            }
        })
    }

    async fn execute(&self, args: Value) -> CsResult<String> {
        let pattern = args
            .get("pattern")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| CsError::new(USER_INPUT, "grep 缺少 pattern 参数"))?
            .to_string();
        let mode_str = args.get("mode").and_then(Value::as_str).unwrap_or("plain");
        let mode = match mode_str {
            "regex" => GrepMode::Regex,
            "fuzzy" => GrepMode::Fuzzy,
            _ => GrepMode::PlainText,
        };
        let limit = args
            .get("limit")
            .and_then(Value::as_u64)
            .unwrap_or(50)
            .clamp(1, 200) as usize;

        let query = QueryParser::default().parse(&pattern);
        let options = GrepSearchOptions {
            max_file_size: 1_048_576,
            max_matches_per_file: 10,
            smart_case: true,
            casing: None,
            file_offset: 0,
            page_limit: limit,
            mode,
            time_budget_ms: 5000,
            enforce_time_budget: false,
            before_context: 0,
            after_context: 0,
            classify_definitions: true,
            trim_whitespace: false,
            abort_signal: None,
        };
        let result = self.engine.picker.grep(&query, &options);

        let mut out = format!(
            "[grep] \"{pattern}\" ({mode_str}) · {}/{} 文件命中 · 已扫描 {} 文件\n",
            result.files_with_matches, result.filtered_file_count, result.total_files_searched
        );
        let rel: Mutex<Vec<String>> = Mutex::new(Vec::with_capacity(result.files.len()));
        for f in &result.files {
            rel.lock()
                .unwrap()
                .push(f.relative_path(&self.engine.picker));
        }
        let rel = rel.into_inner().unwrap();
        for m in &result.matches {
            out.push_str(&format!(
                "{}:{}: {}\n",
                rel[m.file_index], m.line_number, m.line_content
            ));
        }
        if result.next_file_offset > 0 {
            out.push_str(&format!(
                "…还有更多文件未扫（file_offset={}）\n",
                result.next_file_offset
            ));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_for(dir: &Path) -> Arc<FuzzyEngine> {
        Arc::new(FuzzyEngine::new(dir).unwrap())
    }

    async fn run(tool: &dyn Tool, args: Value) -> CsResult<String> {
        tool.execute(args).await
    }

    #[tokio::test]
    async fn find_files_fuzzy_matches_name() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/retry_logic.rs"), "fn alpha() {}\n").unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
        let engine = engine_for(dir.path());
        let t = FileFinderTool::new(engine.clone());
        let out = run(&t, serde_json::json!({"query": "retry"}))
            .await
            .unwrap();
        assert!(out.contains("retry_logic.rs"), "out = {out}");
        let out = run(&t, serde_json::json!({"query": "main"})).await.unwrap();
        assert!(out.contains("main.rs"));
    }

    #[tokio::test]
    async fn grep_plain_finds_lines() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn alpha() {}\nlet x = 1;\n").unwrap();
        let engine = engine_for(dir.path());
        let t = GrepTool::new(engine);
        let out = run(&t, serde_json::json!({"pattern": "fn alpha"}))
            .await
            .unwrap();
        assert!(out.contains("a.rs:1:"), "out = {out}");
    }

    #[tokio::test]
    async fn missing_args_are_tool_errors() {
        let dir = tempfile::tempdir().unwrap();
        let engine = engine_for(dir.path());
        let f = FileFinderTool::new(engine.clone());
        let g = GrepTool::new(engine);
        assert!(run(&f, serde_json::json!({})).await.is_err());
        assert!(run(&g, serde_json::json!({})).await.is_err());
    }
}
