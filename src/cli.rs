//! CLI 命令面（topics/cli-and-output.md）。
//! 退出码：0 就绪 · 1 用法 · 2 配置/凭据 · 3 上游 LLM · 4 目标库 · 5 索引 · 6 内部。

use crate::config::{self, CliOverrides};
use crate::errors::{
    CONFIG_INVALID, CONFIG_MISSING, CsError, CsResult, INDEX_NOT_AVAILABLE, INTERNAL,
    REPO_NOT_FOUND, USER_INPUT, report_error,
};
use crate::{audit, fence, harness, llm, tools, vector};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(
    name = "codesleuth",
    version,
    about = "只读代码侦察 Agent —— 进代码库找东西，带证据回来"
)]
pub struct Cli {
    /// 侦察任务（自然语言），如 "重试逻辑在哪"
    pub task: Option<String>,
    /// 目标仓库根目录
    #[arg(long, value_name = "PATH")]
    pub repo: Option<PathBuf>,
    /// 限定检索范围（glob，可多次）
    #[arg(long, value_name = "GLOB")]
    pub focus: Vec<String>,
    /// 输出机器可读 JSON 报告（stdout 仅 JSON）
    #[arg(long)]
    pub json: bool,
    /// 将报告落盘到文件
    #[arg(long, value_name = "FILE")]
    pub out: Option<PathBuf>,
    /// 覆盖 LLM 模型
    #[arg(long, value_name = "ID")]
    pub model: Option<String>,
    /// 覆盖 LLM base url
    #[arg(long, value_name = "URL")]
    pub base_url: Option<String>,
    /// 忽略 stale 索引，强制重建
    #[arg(long)]
    pub fresh_index: bool,
    /// 启用向量召回暖启动 + vector_search 工具（P003）
    #[arg(long)]
    pub vector: bool,
    /// 启用 repo map 预算化注入（P003 E2）
    #[arg(long)]
    pub repo_map: bool,
    /// 日志详细度（-v info / -vv debug）
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// 查看 / 设置配置
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// 索引管理
    Index {
        /// 目标仓库路径
        path: PathBuf,
        /// 强制重建索引
        #[arg(long)]
        rebuild: bool,
        /// 构建向量索引（嵌入 + 关系感知描述，P003）
        #[arg(long)]
        vector: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// 打印生效配置（或单个键）
    Get {
        /// 键，如 llm.model；缺省打印全部
        key: Option<String>,
    },
    /// 写入全局配置
    Set {
        /// 键（llm.base_url | llm.api_key | llm.model | vector.* | behavior.*）
        key: String,
        /// 值
        value: String,
    },
    /// 打印配置文件路径
    Path,
}

impl Cli {
    pub fn run(self) -> i32 {
        let overrides = CliOverrides {
            base_url: self.base_url.clone(),
            model: self.model.clone(),
        };
        match self.command {
            Some(Command::Config { action }) => run_config(action),
            Some(Command::Index {
                path,
                rebuild,
                vector,
            }) => {
                if vector {
                    match Self::run_index_vector(&overrides, &path) {
                        Ok(()) => 0,
                        Err(e) => {
                            report_error(&e);
                            e.exit_code()
                        }
                    }
                } else {
                    let e = CsError::new(
                        INDEX_NOT_AVAILABLE,
                        format!("结构索引由 codegraph 按需自建：{}", path.display()),
                    )
                    .with_hint(if rebuild {
                        "--rebuild 已记录：run 时可用 --fresh-index 触发".to_string()
                    } else {
                        "向量索引请加 --vector".to_string()
                    });
                    report_error(&e);
                    e.exit_code()
                }
            }
            None => self.run_task(),
        }
    }

    fn run_index_vector(overrides: &CliOverrides, path: &PathBuf) -> CsResult<()> {
        let repo_abs = dunce::canonicalize(path).map_err(|_| {
            CsError::new(
                REPO_NOT_FOUND,
                format!("目标仓库不存在或不可达: {}", path.display()),
            )
        })?;
        let cfg = config::load(overrides.clone())?;
        let api_key = cfg.resolve_api_key()?;
        // 嵌入可用独立供应商：[vector] base_url/api_key 缺省时跟随 [llm]
        let embed_base = cfg
            .vector
            .base_url
            .clone()
            .unwrap_or_else(|| cfg.llm.base_url.clone());
        let embed_key = cfg
            .vector
            .api_key
            .clone()
            .or_else(|| Some(api_key.clone()));
        let mode = match cfg.vector.embed_mode.as_str() {
            "raw" => vector::EmbedMode::Raw,
            _ => vector::EmbedMode::Composite,
        };
        let embed = vector::EmbedClient::new(
            &embed_base,
            embed_key.as_deref().expect("嵌入密钥缺失"),
            &cfg.vector.embed_model,
            cfg.vector.embed_dims,
        );
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CsError::new(INTERNAL, format!("tokio runtime 启动失败: {e}")))?;
        let report = rt.block_on(vector::build_vector_index(
            &repo_abs,
            &vector::store::project_index_dir(&repo_abs),
            embed,
            mode,
        ))?;
        eprintln!(
            "# 向量索引: {} chunks（嵌入 {} 复用 {} 清理 {}）→ {}",
            report.chunks_total,
            report.embedded,
            report.reused,
            report.gc_removed,
            report.index_path.display()
        );
        Ok(())
    }

    fn run_task(self) -> i32 {
        match self.run_task_inner() {
            Ok(()) => 0,
            Err(e) => {
                report_error(&e);
                e.exit_code()
            }
        }
    }

    fn run_task_inner(self) -> CsResult<()> {
        let task = self
            .task
            .clone()
            .filter(|t| !t.trim().is_empty())
            .ok_or_else(|| {
                CsError::new(USER_INPUT, "缺少侦察任务")
                    .with_hint("用法: codesleuth \"<task>\" --repo <path>")
            })?;
        let repo = self.repo.clone().ok_or_else(|| {
            CsError::new(USER_INPUT, "缺少 --repo")
                .with_hint("用法: codesleuth \"<task>\" --repo <path>")
        })?;
        let repo_abs = dunce::canonicalize(&repo).map_err(|_| {
            CsError::new(
                REPO_NOT_FOUND,
                format!("目标仓库不存在或不可达: {}", repo.display()),
            )
            .with_hint("检查路径是否正确（支持相对路径）")
        })?;

        let overrides = CliOverrides {
            base_url: self.base_url.clone(),
            model: self.model.clone(),
        };
        let cfg = config::load(overrides)?;
        let api_key = cfg.resolve_api_key()?;
        tracing::info!(
            repo = %repo_abs.display(),
            model = %cfg.llm.model,
            base_url = %cfg.llm.base_url,
            "配置就绪"
        );

        // 状态目录 + 会话：全局产物落 ~/.codesleuth/（用户拍板归位），目标仓库除 .codesleuth/ 索引外零写入
        let state_dir = crate::config::global_state_dir()
            .ok_or_else(|| CsError::new(CONFIG_MISSING, "找不到 HOME（无法定位 ~/.codesleuth）"))?;
        let session_id = audit::new_session_id();
        let audit_log = audit::Audit::create(&state_dir, &session_id)?;
        let ledger =
            audit::Ledger::load(state_dir.join("ledger").join(format!("{session_id}.json")));

        let provider: llm::SharedProvider =
            Arc::new(llm::OpenAiProvider::new(
                &cfg.llm.base_url,
                &api_key,
                2,
                cfg.thinking_disabled,
            ));

        // 层进 v0（Phase 2）：读层 + 模糊层
        let fence = Arc::new(fence::Fence::new(&repo_abs)?);
        let mut registry = tools::ToolRegistry::new();
        registry.register(Box::new(tools::read::ReadTool::new(fence.clone())));
        let engine = Arc::new(tools::fuzzy::FuzzyEngine::new(&repo_abs)?);
        registry.register(Box::new(tools::fuzzy::FileFinderTool::new(engine.clone())));
        registry.register(Box::new(tools::fuzzy::GrepTool::new(engine)));

        // D1 零写入自证：考前快照（.codegraph 豁免，其余任何差异 = 不可归因变更）
        let guard_before = crate::writeguard::snapshot(&repo_abs).ok();

        // 层进第 1 层（Phase 3 / D011）：结构图 = codegraph MCP，生命周期归 codesleuth
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CsError::new(INTERNAL, format!("tokio runtime 启动失败: {e}")))?;
        let cg: Option<Arc<tools::graph::CodegraphEngine>> = match rt.block_on(
            tools::graph::CodegraphEngine::start(&repo_abs, self.fresh_index),
        ) {
            Ok(engine) => {
                eprintln!("# codegraph MCP ready（explore/callers/callees/impact/files）");
                Some(Arc::new(engine))
            }
            Err(e) => {
                eprintln!("# codegraph 未就绪（结构工具降级，其余继续）: {e}");
                None
            }
        };
        // 诚实工具面（QA FINDING-009）：codegraph 无符号（非代码仓/未建索引）→ 不注册图工具，
        // 并给 agent 地形提示——否则它会反复调用空结果的图工具直至零增量熔断。
        let cg_db = repo_abs.join(".codegraph").join("codegraph.db");
        let graph_symbols_nonempty = cg.is_some()
            && crate::vector::repomap::repo_map_inputs(&cg_db)
                .map(|(s, _)| !s.is_empty())
                .unwrap_or(false);
        let mut parts: Vec<String> = Vec::new();
        let mut first_suffix: Option<String> = None;
        if let Some(cg) = &cg {
            if graph_symbols_nonempty {
                registry.register(Box::new(tools::graph::ExploreTool::new(cg.clone())));
                registry.register(Box::new(tools::graph::CallersTool::new(cg.clone())));
                registry.register(Box::new(tools::graph::CalleesTool::new(cg.clone())));
                registry.register(Box::new(tools::graph::ImpactTool::new(cg.clone())));
                registry.register(Box::new(tools::graph::FilesTool::new(cg.clone())));
            } else {
                eprintln!("# codegraph 无符号索引（非代码仓）：图工具未注册");
                parts.push(
                    "〔地形提示〕本仓库无结构图索引（非代码仓或未建索引）：explore/callers/callees/impact 不可用。                     请用 find_files（带关键词）与 grep（带内容模式）探索，用 read 阅读具体文件。"
                        .into(),
                );
            }
        }

        // P003 E1b/E2 v2：向量层 + 任务相关导航图（召回命中喂地图当种子，首条消息一次性注入）
        if self.vector {
            let mode = match cfg.vector.embed_mode.as_str() {
                "raw" => vector::EmbedMode::Raw,
                _ => vector::EmbedMode::Composite,
            };
            // 嵌入可用独立供应商：[vector] base_url/api_key 缺省时跟随 [llm]（FINDING-013 前瞻）
            let embed_base = cfg
                .vector
                .base_url
                .clone()
                .unwrap_or_else(|| cfg.llm.base_url.clone());
            let embed_key = cfg
                .vector
                .api_key
                .clone()
                .or_else(|| Some(api_key.clone()));
            let embed = vector::EmbedClient::new(
                &embed_base,
                embed_key.as_deref().expect("嵌入密钥缺失"),
                &cfg.vector.embed_model,
                cfg.vector.embed_dims,
            );
            // 索引补建失败不致命（弹性降级，E3-R1 engram CS4015 教训）：警告后尝试复用已有索引
            match rt.block_on(vector::build_vector_index(
                &repo_abs,
                &vector::store::project_index_dir(&repo_abs),
                embed.clone(),
                mode,
            )) {
                Ok(report) => eprintln!(
                    "# 向量索引: {} chunks（嵌入 {} 复用 {} 清理 {}）",
                    report.chunks_total, report.embedded, report.reused, report.gc_removed
                ),
                Err(e) => {
                    tracing::warn!("向量索引构建失败（降级：尝试复用已有索引）: {e}");
                }
            }
            let idx = vector::store::index_path(&vector::store::project_index_dir(&repo_abs), &vector::store::fingerprint(&repo_abs));
            match vector::VectorStore::open(&idx) {
                Ok(store) => {
                    let recall = Arc::new(vector::RecallEngine::open(store, embed)?);
                    registry.register(Box::new(tools::vector_search::VectorSearchTool::new(
                        Arc::clone(&recall),
                    )));
                    // 召回先行（P003 E2 v2）：命中喂给导航图当种子；失败降级为无后缀
                    let hits = match rt.block_on(recall.recall(&task, 10)) {
                        Ok(h) => h,
                        Err(e) => {
                            tracing::warn!("召回失败（跳过注入）: {e}");
                            Vec::new()
                        }
                    };
                    let mut parts: Vec<String> = Vec::new();
                    if !hits.is_empty() {
                        parts.push(vector::recall::RecallEngine::format_recall_block(&hits, 10));
                    }
                    if self.repo_map {
                        let budget = cfg.vector.repomap_budget;
                        match vector::repomap::repo_map_inputs(&cg_db) {
                            Ok((symbols, degrees)) => {
                                let mut seeds: Vec<String> = Vec::new();
                                let mut neighbors: std::collections::HashSet<String> =
                                    std::collections::HashSet::new();
                                for (c, _) in &hits {
                                    if c.kind == "fallback" || c.kind == "leftover" {
                                        continue;
                                    }
                                    if seeds.contains(&c.symbol) {
                                        continue;
                                    }
                                    let (callers, callees) = vector::chunk::relations_for_symbol(
                                        &cg_db, &c.file, &c.symbol,
                                    )
                                    .unwrap_or_default();
                                    for n in callers.into_iter().chain(callees) {
                                        neighbors.insert(n);
                                    }
                                    seeds.push(c.symbol.clone());
                                }
                                let map = vector::repomap::build_task_map(
                                    &symbols, &degrees, budget, &seeds, &neighbors,
                                );
                                eprintln!(
                                    "# 任务导航图: {} 字符（种子 {}，邻居 {}）",
                                    map.len(),
                                    seeds.len(),
                                    neighbors.len()
                                );
                                parts.push(vector::repomap::wrap_repo_section(&map));
                            }
                            Err(e) => {
                                tracing::warn!("导航图构建失败（跳过注入）: {e}");
                            }
                        }
                    }
                    if !parts.is_empty() {
                        first_suffix = Some(parts.join("\n\n"));
                    }
                }
                Err(e) => {
                    tracing::warn!("向量索引不可用（本会话无召回层，任务继续）: {e}");
                }
            }
        }
        // 兜底：地形提示存在但向量块未组装时，先落一份
        if first_suffix.is_none() && !parts.is_empty() {
            first_suffix = Some(parts.join("\n\n"));
        }
        // 仅 --repo-map 无向量：全局图也走首条消息（同层纪律）
        if first_suffix.is_none() && self.repo_map {
            let budget = cfg.vector.repomap_budget;
            match vector::repomap::repo_map_inputs(&cg_db) {
                Ok((symbols, degrees)) => {
                    let map = vector::repomap::build_repo_map(&symbols, &degrees, budget);
                    eprintln!("# 全局导航图: {} 字符", map.len());
                    first_suffix = Some(vector::repomap::wrap_repo_section(&map));
                }
                Err(e) => {
                    tracing::warn!("导航图构建失败（跳过注入）: {e}");
                }
            }
        }

        let agent = harness::Harness::new(
            provider,
            registry,
            audit_log,
            ledger,
            cfg.llm.model.clone(),
            cfg.context.model_context_tokens,
            cfg.context.compact_at_percent,
        );
        let agent = match first_suffix {
            Some(suffix) => agent.with_first_user_suffix(suffix),
            None => agent,
        };
        let outcome = rt.block_on(agent.run(&task))?;
        drop(agent);
        drop(cg); // MCP server 生命周期回收（D011）

        // D1：考后 diff + 归因上报（审计 write_check 行 + stderr 摘要）
        if let Some(before) = &guard_before {
            match crate::writeguard::snapshot(&repo_abs) {
                Ok(after) => {
                    let changes = crate::writeguard::diff(before, &after);
                    let samples: Vec<String> = changes
                        .iter()
                        .take(5)
                        .map(|c| format!("{:?} {}", c.kind, c.path))
                        .collect();
                    audit::append_line(
                        &outcome.audit_path,
                        "write_check",
                        &serde_json::json!({
                            "files_snapshotted": before.len(),
                            "unattributed_changes": changes.len(),
                            "samples": samples,
                        }),
                    )?;
                    if changes.is_empty() {
                        eprintln!("# 零写入自证：{} 文件快照，无变更 ✓", before.len());
                    } else {
                        eprintln!(
                            "# 零写入自证：{} 文件快照，{} 处不可归因变更（并发/外部）：",
                            before.len(),
                            changes.len()
                        );
                        for c in changes.iter().take(5) {
                            eprintln!("#   {:?} {}", c.kind, c.path);
                        }
                        if changes.len() > 5 {
                            eprintln!("#   … 其余 {} 处见审计 write_check 行", changes.len() - 5);
                        }
                    }
                }
                Err(e) => {
                    let _ = audit::append_line(
                        &outcome.audit_path,
                        "write_check",
                        &serde_json::json!({ "available": false, "error": e.message }),
                    );
                    eprintln!("# 零写入自证不可用: {e}");
                }
            }
        }

        let human = outcome.answer.clone();
        let report_json = serde_json::to_string_pretty(&outcome.report)
            .map_err(|e| CsError::new(INTERNAL, format!("报告序列化失败: {e}")))?;

        // D013：报告持久化——任务结束给出可回看的锚点
        let reports_dir = state_dir.join("reports");
        std::fs::create_dir_all(&reports_dir)
            .map_err(|e| CsError::new(INTERNAL, format!("创建报告目录失败: {e}")))?;
        let md_path = reports_dir.join(format!("{session_id}.md"));
        let json_path = reports_dir.join(format!("{session_id}.json"));
        std::fs::write(&md_path, &human)
            .map_err(|e| CsError::new(INTERNAL, format!("报告写入失败: {e}")))?;
        std::fs::write(&json_path, &report_json)
            .map_err(|e| CsError::new(INTERNAL, format!("报告写入失败: {e}")))?;
        if let Some(out_path) = &self.out {
            let body = if self.json { &report_json } else { &human };
            std::fs::write(out_path, body)
                .map_err(|e| CsError::new(INTERNAL, format!("--out 写入失败: {e}")))?;
        }

        if self.json {
            println!("{report_json}");
        } else {
            println!("{human}");
        }
        eprintln!(
            "# {} turns · {} tool calls\n# 报告: {}\n# 审计: {}",
            outcome.turns,
            outcome.tool_calls,
            md_path.display(),
            outcome.audit_path.display()
        );
        Ok(())
    }
}

fn run_config(action: ConfigAction) -> i32 {
    match action {
        ConfigAction::Path => {
            match config::global_config_path() {
                Some(g) => println!("全局: {}", g.display()),
                None => println!("全局: (无用户配置目录)"),
            }
            println!("项目: {}", config::project_config_path().display());
            0
        }
        ConfigAction::Get { key } => match config_get(key) {
            Ok(out) => {
                println!("{out}");
                0
            }
            Err(e) => {
                report_error(&e);
                e.exit_code()
            }
        },
        ConfigAction::Set { key, value } => match config_set(key, value) {
            Ok(path) => {
                println!("已写入 {}", path.display());
                0
            }
            Err(e) => {
                report_error(&e);
                e.exit_code()
            }
        },
    }
}

fn config_get(key: Option<String>) -> CsResult<String> {
    let cfg = config::load(config::CliOverrides::default())?;
    match key.as_deref() {
        None => toml::to_string_pretty(&config::to_file_view(&cfg))
            .map_err(|e| CsError::new(INTERNAL, format!("序列化失败: {e}"))),
        Some("llm.base_url") => Ok(cfg.llm.base_url.clone()),
                Some("llm.model") => Ok(cfg.llm.model.clone()),
        Some(other) => Err(CsError::new(CONFIG_INVALID, format!("未知配置键: {other}"))
            .with_hint("可用键: llm.base_url | llm.api_key | llm.model")),
    }
}

fn config_set(key: String, value: String) -> CsResult<PathBuf> {
    let path = config::global_config_path().ok_or_else(|| {
        CsError::new(CONFIG_MISSING, "找不到用户配置目录").with_hint("检查 XDG_CONFIG_HOME / HOME")
    })?;
    let mut fc = if path.exists() {
        config::parse_file(&path)?
    } else {
        config::FileConfig::default()
    };
    config::apply_set(&mut fc, &key, value)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CsError::new(CONFIG_INVALID, format!("创建配置目录失败: {e}")))?;
    }
    std::fs::write(
        &path,
        toml::to_string_pretty(&fc)
            .map_err(|e| CsError::new(INTERNAL, format!("序列化失败: {e}")))?,
    )
    .map_err(|e| CsError::new(CONFIG_INVALID, format!("写入失败: {e}")))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_task_mode() {
        let cli = Cli::try_parse_from(["codesleuth", "find retry logic", "--repo", "."]).unwrap();
        assert!(cli.command.is_none());
        assert_eq!(cli.task.as_deref(), Some("find retry logic"));
        assert_eq!(cli.repo.as_deref(), Some(std::path::Path::new(".")));
    }

    #[test]
    fn parses_full_surface() {
        let cli = Cli::try_parse_from([
            "codesleuth",
            "任务",
            "--repo",
            "r",
            "--focus",
            "a/**",
            "--focus",
            "b/**",
            "--json",
            "--out",
            "o.json",
            "--model",
            "m",
            "--base-url",
            "http://x",
            "--fresh-index",
            "-vv",
        ])
        .unwrap();
        assert_eq!(cli.focus.len(), 2);
        assert!(cli.json && cli.fresh_index);
        assert_eq!(cli.verbose, 2);
        assert_eq!(cli.model.as_deref(), Some("m"));
    }

    #[test]
    fn parses_config_subcommands() {
        let cli = Cli::try_parse_from(["codesleuth", "config", "get", "llm.model"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Config {
                action: ConfigAction::Get { key: Some(_) }
            })
        ));
        let cli = Cli::try_parse_from(["codesleuth", "config", "path"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Config {
                action: ConfigAction::Path
            })
        ));
        let cli = Cli::try_parse_from(["codesleuth", "index", ".", "--rebuild"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Index { rebuild: true, .. })
        ));
    }

    #[test]
    fn missing_task_is_usage_error_exit_1() {
        let cli = Cli::try_parse_from(["codesleuth"]).unwrap();
        let err = cli.run_task_inner().unwrap_err();
        assert_eq!(err.code, USER_INPUT);
        assert_eq!(err.exit_code(), 1);
    }
}
