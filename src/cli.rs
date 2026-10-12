//! CLI 命令面（topics/cli-and-output.md）。
//! 退出码：0 就绪 · 1 用法 · 2 配置/凭据 · 3 上游 LLM · 4 目标库 · 5 索引 · 6 内部。
//! 输出面（D018）：--output-format report|raw|json（替换原 --json）；
//! 引导锁 Lost=复用继续（D020）；--require 输出契约（D021，CS2005）；
//! 索引排除链（D023，EN-6）：config [index].exclude + .codesleuthignore + .gitignore。

use crate::config::{self, CliOverrides};
use crate::errors::{
    CONFIG_INVALID, CONFIG_MISSING, CsError, CsResult, INDEX_LOCKED, INDEX_NOT_AVAILABLE, INTERNAL,
    REPO_NOT_FOUND, USER_INPUT, report_error,
};
use crate::{audit, fence, harness, llm, tools, vector};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

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
    // P007 R4.2：--focus 已删（全链路零消费的假旗标，曾误导「限定检索范围」）
    /// 输出格式（D018）：report=人类模板（默认）· raw=仅模型正文（不套报告壳）· json=结构化报告
    #[arg(long, value_enum, default_value_t = OutputFormat::Report)]
    pub output_format: OutputFormat,
    /// 将报告落盘到文件
    #[arg(long, value_name = "FILE")]
    pub out: Option<PathBuf>,
    /// 覆盖 LLM 模型
    #[arg(long, value_name = "ID")]
    pub model: Option<String>,
    /// 覆盖 LLM base url
    #[arg(long, value_name = "URL")]
    pub base_url: Option<String>,
    /// 模型档位（D017）：选用 [llm.profiles.<名字>] 定义的连接面；旗标 --model/--base-url 仍可再压
    #[arg(long, value_name = "NAME")]
    pub profile: Option<String>,
    /// 忽略 stale 索引，强制重建
    #[arg(long)]
    pub fresh_index: bool,
    /// 启用向量召回暖启动 + vector_search 工具（P003）
    #[arg(long)]
    pub vector: bool,
    /// 接受略旧索引（EN-8/D025）：--vector 引导期跳过新鲜度检查与补建，
    /// 索引在则直接用（文档直出等已知入口场景）；索引缺失仍正常构建。
    /// 与 --fresh-index 同给时以 --fresh-index 为准（显式重建优先）。
    #[arg(long)]
    pub stale_ok: bool,
    /// 引导锁等待基准（EN-9/D025）：秒；对手推进中自动延展（上限 2×），默认 300
    #[arg(long, value_name = "SECS")]
    pub lock_timeout: Option<u64>,
    /// 输出契约（D021）：最终回答必须逐字包含的标记，可多次；缺失补一轮仍缺则判 CS2005
    #[arg(long = "require", value_name = "MARKER")]
    pub require: Vec<String>,
    /// 输出契约·行首锚定（D022）：标记必须独占某行行首（防一行流糊弄），可多次
    #[arg(long = "require-line", value_name = "MARKER")]
    pub require_line: Vec<String>,
    /// 输出契约·出现次数（D027，EN-11）：标记出现 ≥ N 次才算产出（防引用式糊弄），可多次
    #[arg(long = "require-count", value_names = ["MARKER", "N"])]
    pub require_count: Vec<String>,
    /// 启用 repo map 预算化注入（P003 E2）
    #[arg(long)]
    pub repo_map: bool,
    /// 日志详细度（-v info / -vv debug）
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// 人类可读模板（含统计与证据清单）
    Report,
    /// 仅透传 report.answer 原文——交付正文不套「侦察报告」壳（C2）
    Raw,
    /// 结构化 Report JSON（stdout 仅 JSON）
    Json,
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
        // P007 R4.3：--rebuild 已删（诚实报错假旗标；实际入口 run --fresh-index）
        /// 构建向量索引（嵌入 + text_hash 增量）
        #[arg(long)]
        vector: bool,
        /// 引导锁等待基准（EN-9/D025）：秒；默认 300
        #[arg(long, value_name = "SECS")]
        lock_timeout: Option<u64>,
    },
    /// 批量模式（D028/EN-12）：单进程多会话，boot 一次全任务共享（引导成本 O(1)）
    Batch {
        /// 任务清单（JSON 数组或 {"tasks":[...]}；每条 id/task/require/require_line/require_count）
        path: PathBuf,
        /// 目标仓库根目录
        #[arg(long, value_name = "PATH")]
        repo: PathBuf,
        /// 并发会话数（boot 始终共享；jobs 只影响并发度）
        #[arg(long, value_name = "N", default_value_t = 1)]
        jobs: usize,
        /// 启用向量召回（产物全任务共享）
        #[arg(long)]
        vector: bool,
        /// 启用 repo map 导航图
        #[arg(long)]
        repo_map: bool,
        /// 接受略旧索引（EN-8/D025）
        #[arg(long)]
        stale_ok: bool,
        /// 引导锁等待基准（EN-9/D025）：秒；默认 300
        #[arg(long, value_name = "SECS")]
        lock_timeout: Option<u64>,
        /// 每任务产物目录（Q004：<id>.md + summary.json）
        #[arg(long, value_name = "DIR", default_value = "codesleuth-batch-out")]
        out_dir: PathBuf,
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
        /// 配置键（全集见未知键提示或 config get；llm.* | context.* | vector.* | behavior.thinking_on | graph.bin）
        key: String,
        /// 值
        value: String,
    },
    /// 打印配置文件路径
    Path,
}

impl Cli {
    pub fn run(self, session_id: &str) -> i32 {
        // P005 R7.4：session span——run 全程事件自动携带 session_id（结构化串线）
        let _session = tracing::info_span!("session", session_id = %session_id).entered();
        // D026：match self.command 会 move/partial-move，错误路由 flag 先拍
        let json_mode = matches!(self.output_format, OutputFormat::Json);
        let overrides = CliOverrides {
            base_url: self.base_url.clone(),
            model: self.model.clone(),
            profile: self.profile.clone(),
        };
        match self.command {
            Some(Command::Config { action }) => run_config(action, self.profile.as_deref()),
            Some(Command::Index {
                path,
                vector,
                lock_timeout,
            }) => {
                if vector {
                    match Self::run_index_vector(&overrides, &path, lock_timeout) {
                        Ok(()) => 0,
                        Err(e) => {
                            Self::report_error_for(json_mode, &e);
                            e.exit_code()
                        }
                    }
                } else {
                    // P007 R4.3：--rebuild 旗标已删，非 --vector 一律诚实报错
                    let e = index_structure_error();
                    Self::report_error_for(json_mode, &e);
                    e.exit_code()
                }
            }
            Some(Command::Batch {
                path: ref spec_path,
                repo: ref batch_repo,
                jobs,
                vector,
                repo_map,
                stale_ok,
                lock_timeout,
                ref out_dir,
            }) => {
                // EN-8/D025：--fresh-index 显式重建优先
                let stale = stale_ok && !self.fresh_index;
                match Self::run_batch(
                    &self,
                    &overrides,
                    session_id,
                    spec_path,
                    batch_repo,
                    jobs,
                    out_dir,
                    vector,
                    repo_map,
                    stale,
                    lock_timeout,
                    self.fresh_index,
                ) {
                    Ok(()) => 0,
                    Err(e) => {
                        Self::report_error_for(json_mode, &e);
                        e.exit_code()
                    }
                }
            }
            None => self.run_task(session_id),
        }
    }

    fn run_index_vector(
        overrides: &CliOverrides,
        path: &PathBuf,
        lock_timeout: Option<u64>,
    ) -> CsResult<()> {
        let repo_abs = dunce::canonicalize(path).map_err(|e| {
            CsError::new(
                REPO_NOT_FOUND,
                format!("目标仓库不存在或不可达: {}", path.display()),
            )
            .with_source(format!("canonicalize: {e}"))
        })?;
        let cfg = config::load(overrides.clone())?;
        let api_key = cfg.resolve_api_key()?;
        // 嵌入可用独立供应商：[vector] base_url/api_key 缺省时跟随 [llm]（P005 R7.1 去重）
        let (embed_base, embed_key) = resolve_embed_endpoint(
            cfg.vector.base_url.clone(),
            cfg.vector.api_key.clone(),
            &cfg.llm.base_url,
            &api_key,
        );
        let mode = match cfg.vector.embed_mode.as_str() {
            "raw" => vector::EmbedMode::Raw,
            _ => vector::EmbedMode::Composite,
        };
        let embed = vector::EmbedClient::new(
            &embed_base,
            &embed_key,
            &cfg.vector.embed_model,
            cfg.vector.embed_dims,
        );
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CsError::new(INTERNAL, format!("tokio runtime 启动失败: {e}")))?;
        // D014：手动预建索引与 run 时引导共用同一把引导锁；
        // D020：等待后获得 = 对手刚完成构建，若产物已就绪则直接复用返回。
        // P007 R2.6：产物缺失不再无锁裸建（重开竞态窗口）——重新竞争引导锁，
        // 超限后诚实判负（与 graph.rs R1.6 同语义）。
        let index_dir = vector::store::project_index_dir(&repo_abs);
        let index_file =
            vector::store::index_path(&index_dir, &vector::store::fingerprint(&repo_abs));
        let mut lost_retries: u32 = 0;
        let _guard = loop {
            let outcome = crate::bootlock::acquire_guard(
                &repo_abs,
                Duration::from_secs(lock_timeout.unwrap_or(300)),
                "向量索引构建",
            )?;
            match outcome {
                crate::bootlock::BootLockOutcome::OpponentFinished if index_file.exists() => {
                    eprintln!("# 向量索引: 另一进程刚完成构建，复用其产物（D020）");
                    break None;
                }
                crate::bootlock::BootLockOutcome::OpponentFinished => {
                    lost_retries += 1;
                    if lost_retries > 3 {
                        return Err(crate::errors::CsError::new(
                            crate::errors::INDEX_LOCKED,
                            "对手进程释放引导锁但未留下索引产物（疑似多次中途崩溃）",
                        )
                        .with_hint("检查磁盘与并发进程后重试 index --vector"));
                    }
                    tracing::warn!(
                        component = "vector_build",
                        "对手释放引导锁但产物缺失，重新竞争引导锁（{lost_retries}/3）"
                    );
                }
                crate::bootlock::BootLockOutcome::Won(g) => break Some(g),
            }
        };
        let report = rt.block_on(vector::build_vector_index(
            &repo_abs,
            &index_dir,
            embed,
            mode,
            &cfg.index.exclude,
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

    /// D026（EN-10）：错误出口按输出格式路由——json 模式走单行 JSON 独立通道
    /// （errors::structured_error_json），其余人读格式；编排层按行首 `{"code":` 稳定提取 CS 码。
    /// 关联函数带 json_mode 参数：调用方须在 move/partial-move 前先拍 flag（run/run_task）。
    fn report_error_for(json_mode: bool, e: &CsError) {
        if json_mode {
            crate::errors::report_error_structured(e);
        } else {
            report_error(e);
        }
    }

    fn run_task(self, session_id: &str) -> i32 {
        // self 被 run_task_inner 拿走，错误路由先拍 flag（D026）
        let json_mode = matches!(self.output_format, OutputFormat::Json);
        match self.run_task_inner(session_id) {
            Ok(()) => 0,
            Err(e) => {
                Self::report_error_for(json_mode, &e);
                e.exit_code()
            }
        }
    }

    /// D1 零写入自证：考后 diff + 归因上报（审计 write_check 行 + stderr 摘要）。
    /// P007 R2.3/R2.4：成功与失败会话共用本入口；考前快照失败时诚实落
    /// available:false 留痕，不静默。防线自身的失败必须可见（P004 T3/T4）。
    fn emit_write_check(
        before: Option<&crate::writeguard::Manifest>,
        repo_abs: &std::path::Path,
        audit_path: &std::path::Path,
    ) {
        let Some(before) = before else {
            if let Err(ae) = audit::append_line(
                audit_path,
                "write_check",
                &serde_json::json!({"available": false, "reason": "guard_before_unavailable"}),
            ) {
                tracing::warn!("write_check 审计留痕失败: {ae}");
            }
            return;
        };
        match crate::writeguard::snapshot(repo_abs) {
            Ok(after) => {
                let changes = crate::writeguard::diff(before, &after);
                let samples: Vec<String> = changes
                    .iter()
                    .take(5)
                    .map(|c| format!("{:?} {}", c.kind, c.path))
                    .collect();
                if let Err(ae) = audit::append_line(
                    audit_path,
                    "write_check",
                    &serde_json::json!({
                        "files_snapshotted": before.len(),
                        "unattributed_changes": changes.len(),
                        "samples": samples,
                    }),
                ) {
                    tracing::warn!("write_check 审计留痕失败: {ae}");
                }
                if changes.is_empty() {
                    eprintln!("# 零写入自证：{} 文件快照，无变更 ✓", before.len());
                } else {
                    // 完整性违规 = 会话级严重事件（P004 T4）：ERROR 级必打
                    tracing::error!(
                        "零写入自证检测到 {} 处未归属变更（完整性违规）",
                        changes.len()
                    );
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
                // write_check 留痕失败必须可见（P004 T3 吞错清零）
                if let Err(ae) = audit::append_line(
                    audit_path,
                    "write_check",
                    &serde_json::json!({ "available": false, "error": e.message }),
                ) {
                    tracing::warn!("write_check 审计留痕失败: {ae}");
                }
                tracing::warn!("零写入自证不可用: {e}");
            }
        }
    }

    fn run_task_inner(self, session_id: &str) -> CsResult<()> {
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
        let repo_abs = dunce::canonicalize(&repo).map_err(|e| {
            CsError::new(
                REPO_NOT_FOUND,
                format!("目标仓库不存在或不可达: {}", repo.display()),
            )
            .with_hint("检查路径是否正确（支持相对路径）")
            .with_source(format!("canonicalize: {e}"))
        })?;

        let overrides = CliOverrides {
            base_url: self.base_url.clone(),
            model: self.model.clone(),
            profile: self.profile.clone(),
        };
        let cfg = config::load(overrides)?;
        let api_key = cfg.resolve_api_key()?;
        tracing::info!(
            repo = %repo_abs.display(),
            model = %cfg.llm.model,
            base_url = %cfg.llm.base_url,
            profile = %cfg.active_profile.clone().unwrap_or_else(|| "default".into()),
            "配置就绪"
        );

        // 状态目录 + 会话：全局产物落 ~/.codesleuth/（用户拍板归位），目标仓库除 .codesleuth/ 索引外零写入
        // P005 R7.4：session id 由入口（main）传入——日志串线与审计同一身份
        let state_dir = crate::config::global_state_dir()
            .ok_or_else(|| CsError::new(CONFIG_MISSING, "找不到 HOME（无法定位 ~/.codesleuth）"))?;
        let audit_log = audit::Audit::create(&state_dir, session_id)?;

        let provider: llm::SharedProvider = Arc::new(llm::OpenAiProvider::new(
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
        // P007 R2.3：失败不再被 .ok() 静默吞掉——留 degraded 审计痕迹 + warn，
        // 零写入自证防线不可无声消失
        let guard_before = match crate::writeguard::snapshot(&repo_abs) {
            Ok(s) => Some(s),
            Err(e) => {
                tracing::warn!(component = "writeguard", error = %e, "考前快照失败，零写入自证将不可用");
                let _ = audit_log.record(
                    "degraded",
                    &serde_json::json!({"component": "writeguard", "stage": "before", "error": e.to_string()}),
                );
                None
            }
        };

        // 层进第 1 层（Phase 3 / D011）：结构图 = codegraph MCP，生命周期归 codesleuth
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CsError::new(INTERNAL, format!("tokio runtime 启动失败: {e}")))?;
        let cg: Option<Arc<tools::graph::CodegraphEngine>> = match rt.block_on(
            tools::graph::CodegraphEngine::start(
                &repo_abs,
                &cfg.graph.bin,
                self.fresh_index,
                Duration::from_secs(self.lock_timeout.unwrap_or(300)),
            ),
        ) {
            Ok(engine) => {
                tracing::debug!("# codegraph MCP ready（explore/callers/callees/impact/files）");
                Some(Arc::new(engine))
            }
            Err(e) if e.code == INDEX_LOCKED => {
                // D020：引导锁仅剩超时判负路径（Lost 已改为复用继续）——超时绝不带残缺工具面继续
                return Err(e);
            }
            Err(e) => {
                tracing::warn!(component = "codegraph", error = %e, "codegraph 未就绪，结构工具降级（其余继续）");
                // P005 R5.2：非致命降级进审计留痕（最佳 effort，不留痕失败不掩主流程）
                record_degraded(&audit_log, "codegraph", &e);
                None
            }
        };
        // 诚实工具面（QA FINDING-009）：codegraph 无符号 → 不注册图工具，给地形提示
        let cg_db = repo_abs.join(".codegraph").join("codegraph.db");
        let mut parts: Vec<String> = Vec::new();
        let mut first_suffix: Option<String> = None;
        register_graph_tools(&cg, &cg_db, &mut registry, &mut parts);

        // P003 E1b/E2 v2：向量层 + 任务相关导航图（召回命中喂地图当种子，首条消息一次性注入）
        if self.vector {
            match setup_vector_layer(
                &VectorLayerCtx {
                    repo_abs: &repo_abs,
                    task: &task,
                    api_key: &api_key,
                    cg_db: &cg_db,
                    rt: &rt,
                    audit: &audit_log,
                },
                &cfg,
                self.repo_map,
                // EN-8/D025：--fresh-index 显式重建优先，同给时 --stale-ok 失效
                self.stale_ok && !self.fresh_index,
                Duration::from_secs(self.lock_timeout.unwrap_or(300)),
                &mut registry,
            ) {
                Ok(Some(suffix)) => first_suffix = Some(suffix),
                Ok(None) => {}
                Err(e) if e.code == INDEX_LOCKED => {
                    // D020：引导锁仅剩超时判负路径（Lost 已改为复用继续）——不降级
                    return Err(e);
                }
                Err(e) => {
                    tracing::warn!(component = "vector_layer", error = %e, "向量层装配失败（本会话无召回层，任务继续）");
                    // P005 R5.2：非致命降级进审计留痕
                    record_degraded(&audit_log, "vector_layer", &e);
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
                    tracing::debug!("# 全局导航图: {} 字符", map.len());
                    first_suffix = Some(vector::repomap::wrap_repo_section(&map));
                }
                Err(e) => {
                    tracing::warn!(component = "repo_map", error = %e, "导航图构建失败（跳过注入）");
                    // P005 R5.2：非致命降级进审计留痕
                    record_degraded(&audit_log, "repo_map", &e);
                }
            }
        }

        // 审计文件路径提前捕获（audit_log 即将 move 进 Harness；失败路径自证要用）
        let audit_path = audit_log.path().to_path_buf();
        let agent = harness::Harness::new(
            provider,
            registry,
            audit_log,
            cfg.llm.model.clone(),
            cfg.context.model_context_tokens,
            cfg.context.compact_at_percent,
        );
        let agent = match first_suffix {
            Some(suffix) => agent.with_first_user_suffix(suffix),
            None => agent,
        };
        // D027：--require-count 成对解析（MARKER N），奇数个/N 非法 → 用法错误
        let mut require_counts: Vec<(String, usize)> = Vec::new();
        for pair in self.require_count.chunks(2) {
            if pair.len() != 2 {
                return Err(CsError::new(
                    USER_INPUT,
                    "--require-count 需要 MARKER N 成对给出",
                )
                .with_hint("示例：--require-count '<!--doc:start:a1-->' 3"));
            }
            let n: usize = pair[1].parse().map_err(|_| {
                CsError::new(USER_INPUT, format!("--require-count 的 N 非法: {}", pair[1]))
                    .with_hint("N 为非负整数（标记需出现的最少次数）")
            })?;
            require_counts.push((pair[0].clone(), n));
        }
        let agent = if self.require.is_empty()
            && self.require_line.is_empty()
            && require_counts.is_empty()
        {
            agent
        } else {
            let a = if self.require.is_empty() {
                agent
            } else {
                agent.with_required_markers(self.require.clone())
            };
            let a = if self.require_line.is_empty() {
                a
            } else {
                a.with_required_line_markers(self.require_line.clone())
            };
            if require_counts.is_empty() {
                a
            } else {
                a.with_required_count_markers(require_counts)
            }
        };
        // P007 R2.4：失败会话同样做考后自证——失败恰是最需要自证的会话
        let run_result = rt.block_on(agent.run(&task));
        drop(agent);
        drop(cg); // MCP server 生命周期回收（D011）
        let outcome = match run_result {
            Ok(o) => o,
            Err(e) => {
                Self::emit_write_check(guard_before.as_ref(), &repo_abs, &audit_path);
                return Err(e);
            }
        };
        Self::emit_write_check(guard_before.as_ref(), &repo_abs, &outcome.audit_path);

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
            let body: String = match self.output_format {
                OutputFormat::Json => report_json.clone(),
                OutputFormat::Raw => outcome.report.answer.clone(),
                OutputFormat::Report => human.clone(),
            };
            std::fs::write(out_path, body)
                .map_err(|e| CsError::new(INTERNAL, format!("--out 写入失败: {e}")))?;
        }

        // D018：stdout 纪律——report=人类模板 · raw=仅模型正文（C2 交付通道）· json=仅结构化 JSON
        match self.output_format {
            OutputFormat::Json => println!("{report_json}"),
            OutputFormat::Raw => println!("{}", outcome.report.answer),
            OutputFormat::Report => println!("{human}"),
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
    /// D028（EN-12）：batch 编排——boot 一次（graph 引导 + 向量构建），N 会话并发复用。
    /// 产物契约（Q004）：out_dir/<id>.md（每任务正文）+ summary.json（编排层对号）；
    /// 全局 reports/ 仍按 session 隔离。失败不中断批量，summary 记录，任一失败退出码 1。
    #[allow(clippy::too_many_arguments)]
    fn run_batch(
        &self,
        overrides: &CliOverrides,
        session_id: &str,
        spec_path: &Path,
        repo: &Path,
        jobs: usize,
        out_dir: &Path,
        vector: bool,
        repo_map: bool,
        stale_ok: bool,
        lock_timeout: Option<u64>,
        fresh_index: bool,
    ) -> CsResult<()> {
        let spec_text = std::fs::read_to_string(spec_path).map_err(|e| {
            CsError::new(USER_INPUT, format!("任务清单读取失败: {e}"))
                .with_hint("清单为 JSON 数组或 {\"tasks\":[...]}")
        })?;
        let specs = parse_batch_spec(&spec_text)?;
        if specs.is_empty() {
            return Err(CsError::new(USER_INPUT, "任务清单为空"));
        }
        let jobs = jobs.max(1);
        let repo_abs = dunce::canonicalize(repo).map_err(|e| {
            CsError::new(
                REPO_NOT_FOUND,
                format!("目标仓库不存在或不可达: {}", repo.display()),
            )
            .with_source(format!("canonicalize: {e}"))
        })?;
        let cfg = config::load(overrides.clone())?;
        let api_key = cfg.resolve_api_key()?;
        let state_dir = crate::config::global_state_dir()
            .ok_or_else(|| CsError::new(CONFIG_MISSING, "找不到 HOME（无法定位 ~/.codesleuth）"))?;
        let lock_timeout_d = Duration::from_secs(lock_timeout.unwrap_or(300));
        let boot_audit = audit::Audit::create(&state_dir, session_id)?;
        eprintln!(
            "# batch: {} 任务 · jobs {jobs} · boot 开始（一次共享，不随任务数膨胀）",
            specs.len()
        );

        // ---- boot（一次）：graph 引导 + 向量构建，锁内串行完成（D014/D025 语义不变）----
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CsError::new(INTERNAL, format!("tokio runtime 启动失败: {e}")))?;
        let cg = match rt.block_on(tools::graph::CodegraphEngine::start(
            &repo_abs,
            &cfg.graph.bin,
            fresh_index,
            lock_timeout_d,
        )) {
            Ok(e) => Some(Arc::new(e)),
            Err(e) if e.code == INDEX_LOCKED => return Err(e),
            Err(e) => {
                tracing::warn!(component = "codegraph", error = %e, "codegraph 未就绪，结构工具降级（其余继续）");
                record_degraded(&boot_audit, "codegraph", &e);
                None
            }
        };
        let embed = build_embed_client(&cfg, &api_key);
        if vector {
            vector_boot(
                &repo_abs,
                &cfg,
                &embed,
                stale_ok,
                lock_timeout_d,
                &boot_audit,
                &rt,
            )?;
        }
        let cg_db = repo_abs.join(".codegraph").join("codegraph.db");
        let fence = Arc::new(fence::Fence::new(&repo_abs)?);
        let fuzzy = Arc::new(tools::fuzzy::FuzzyEngine::new(&repo_abs)?);
        let provider: llm::SharedProvider = Arc::new(llm::OpenAiProvider::new(
            &cfg.llm.base_url,
            &api_key,
            2,
            cfg.thinking_disabled,
        ));
        let boot = Arc::new(BatchBoot {
            repo_abs,
            state_dir,
            cfg,
            api_key,
            provider,
            fence,
            fuzzy,
            rt,
            cg,
            cg_db,
            vector_on: vector,
            repo_map_on: repo_map,
            embed,
        });

        // ---- 会话并发：信号量限流，失败不中断批量 ----
        let sem = Arc::new(tokio::sync::Semaphore::new(jobs));
        let t0 = std::time::Instant::now();
        let outcomes = boot.rt.block_on(async {
            let mut handles = Vec::new();
            for (idx, spec) in specs.into_iter().enumerate() {
                let permit = Arc::clone(&sem).acquire_owned().await.ok();
                let boot2 = Arc::clone(&boot);
                let sid = format!("{}-b{}", audit::new_session_id(), idx);
                let id = batch_task_id(&spec, idx);
                let spec = Arc::new(spec);
                let out2 = out_dir.to_path_buf();
                handles.push(tokio::spawn(async move {
                    let _permit = permit;
                    run_batch_session(&boot2, &spec, &sid, &id, &out2).await
                }));
            }
            let mut outcomes = Vec::new();
            for h in handles {
                match h.await {
                    Ok(o) => outcomes.push(o),
                    Err(e) => outcomes.push(BatchTaskOutcome {
                        id: "?".into(),
                        session_id: "?".into(),
                        status: "error",
                        error: Some(format!("会话任务崩溃: {e}")),
                        report_md: None,
                    }),
                }
            }
            outcomes
        });

        // ---- summary.json（Q004：编排层按 id 对号）----
        let failed = outcomes.iter().filter(|o| o.status != "ok").count();
        let _ = std::fs::create_dir_all(out_dir);
        let summary = serde_json::json!({
            "total": outcomes.len(),
            "failed": failed,
            "elapsed_ms": t0.elapsed().as_millis() as u64,
            "tasks": outcomes,
        });
        let summary_path = out_dir.join("summary.json");
        let summary_json = serde_json::to_string_pretty(&summary)
            .map_err(|e| CsError::new(INTERNAL, format!("summary 序列化失败: {e}")))?;
        std::fs::write(&summary_path, summary_json)
            .map_err(|e| CsError::new(INTERNAL, format!("summary 写入失败: {e}")))?;
        eprintln!(
            "# batch 完成：{} 任务（失败 {failed}）· 耗时 {:.1}s · summary {}",
            summary["total"].as_u64().unwrap_or(0),
            t0.elapsed().as_secs_f32(),
            summary_path.display()
        );
        if failed > 0 {
            return Err(CsError::new(
                USER_INPUT,
                format!("batch 部分任务失败: {failed}"),
            )
            .with_hint(format!("详见 {}（每条含 error 字段）", summary_path.display())));
        }
        Ok(())
    }
}

/// `index` 子命令非 --vector 的诚实报错：结构索引由 codegraph 在 run 时按需自建，
/// 重建的唯一入口是 run --fresh-index（P007 R4.3：--rebuild 旗标已删，无参分支）。
fn index_structure_error() -> CsError {
    CsError::new(
        INDEX_NOT_AVAILABLE,
        "index 子命令仅支持 --vector（结构索引由 codegraph 在 run 时按需自建）",
    )
    .with_hint("结构索引重建：codesleuth run --fresh-index；向量索引请加 --vector")
}

fn run_config(action: ConfigAction, profile: Option<&str>) -> i32 {
    match action {
        ConfigAction::Path => {
            match config::global_config_path() {
                Some(g) => println!("全局: {}", g.display()),
                None => println!("全局: (无用户配置目录)"),
            }
            println!("项目: {}", config::project_config_path().display());
            0
        }
        ConfigAction::Get { key } => match config_get(key, profile) {
            Ok(out) => {
                println!("{out}");
                0
            }
            Err(e) => {
                report_error(&e);
                e.exit_code()
            }
        },
        ConfigAction::Set { key, value } => {
            // P007 R3.5：--profile 对 set 不生效（写的是全局文件视图），不再静默忽略
            if profile.is_some() {
                eprintln!("# 提示: --profile 对 config set 不生效（档位只在 run 时选用）");
            }
            match config_set(key, value) {
                Ok(path) => {
                    println!("已写入 {}", path.display());
                    0
                }
                Err(e) => {
                    report_error(&e);
                    e.exit_code()
                }
            }
        }
    }
}

/// 向量层装配上下文（P004 T6.1：从 run_task_inner 抽出，消除上帝函数）。
struct VectorLayerCtx<'a> {
    repo_abs: &'a std::path::Path,
    task: &'a str,
    api_key: &'a str,
    cg_db: &'a std::path::Path,
    rt: &'a tokio::runtime::Runtime,
    /// P005 R5.2：内层非致命降级（构建失败/召回失败）也进审计留痕
    audit: &'a audit::Audit,
}

/// P007 R3.28：非致命降级留痕单一入口——散弹式修改病灶收敛（改留痕格式只动这里）。
/// best-effort：留痕失败不掩主流程。
fn record_degraded(audit: &crate::audit::Audit, component: &str, error: impl std::fmt::Display) {
    let _ = audit.record(
        "degraded",
        &serde_json::json!({"component": component, "error": error.to_string()}),
    );
}

/// 嵌入供应商解析（P005 R7.1，coupling 审查去重）：[vector].base_url/api_key 缺省跟随 [llm]。
/// 返回 (base_url, api_key)；llm 层密钥由 resolve_api_key 保证非空，此处不再设防。
fn resolve_embed_endpoint(
    vector_base: Option<String>,
    vector_key: Option<String>,
    llm_base: &str,
    llm_key: &str,
) -> (String, String) {
    (
        vector_base.unwrap_or_else(|| llm_base.to_string()),
        vector_key.unwrap_or_else(|| llm_key.to_string()),
    )
}

/// 向量层装配：补建索引 → 开库 → 注册 vector_search → 召回暖启动 + 任务导航图。
/// 返回注入首条消息的后缀（召回块 + 导航图）；向量不可用时 Ok(None)（弹性降级，不致命）。
fn setup_vector_layer(
    ctx: &VectorLayerCtx<'_>,
    cfg: &config::Config,
    repo_map: bool,
    stale_ok: bool,
    lock_timeout: Duration,
    registry: &mut tools::ToolRegistry,
) -> CsResult<Option<String>> {
    let embed = build_embed_client(cfg, ctx.api_key);
    vector_boot(
        ctx.repo_abs,
        cfg,
        &embed,
        stale_ok,
        lock_timeout,
        ctx.audit,
        ctx.rt,
    )?;
    ctx.rt
        .block_on(vector_attach(ctx, cfg, repo_map, embed, registry))
}

/// D028：嵌入客户端构造（[vector] base_url/api_key 缺省时跟随 [llm]，P005 R7.1）。
fn build_embed_client(cfg: &config::Config, api_key: &str) -> vector::EmbedClient {
    let (embed_base, embed_key) =
        resolve_embed_endpoint(cfg.vector.base_url.clone(), cfg.vector.api_key.clone(), &cfg.llm.base_url, api_key);
    vector::EmbedClient::new(
        &embed_base,
        &embed_key,
        &cfg.vector.embed_model,
        cfg.vector.embed_dims,
    )
}

/// D028（EN-12）：向量引导段——构建/刷新只做一次，N 会话 attach 复用产物（batch 核心收益）。
fn vector_boot(
    repo_abs: &Path,
    cfg: &config::Config,
    embed: &vector::EmbedClient,
    stale_ok: bool,
    lock_timeout: Duration,
    boot_audit: &audit::Audit,
    rt: &tokio::runtime::Runtime,
) -> CsResult<()> {
    let mode = match cfg.vector.embed_mode.as_str() {
        "raw" => vector::EmbedMode::Raw,
        _ => vector::EmbedMode::Composite,
    };
    // D014：向量构建与 graph 引导共用仓库级引导锁，进程间串行化；
    // D020：等待后获得 = 对手刚完成构建，产物已就绪，跳过构建直接开库。
    // P007 R2.5：Lost≠产物就绪——对手可能中途崩溃留下缺失/残缺产物；
    // 产物缺失时重新竞争引导锁（上限 3 次），超限弹性降级（无召回层继续）。
    // EN-8/D025：--stale-ok 且产物在 → 跳过引导刷新（新鲜度检查/补建），直接开库。
    // 文档直出等已知入口场景：略旧索引完全可接受，不值得付重嵌成本与锁争用。
    // stale_ok 传入前已合并 fresh_index（显式重建优先）；索引缺失仍走正常构建。
    let index_dir = vector::store::project_index_dir(repo_abs);
    let idx = vector::store::index_path(&index_dir, &vector::store::fingerprint(repo_abs));
    let skip_refresh = stale_ok && idx.exists();
    if skip_refresh {
        eprintln!("# 向量索引: --stale-ok 跳过引导刷新，直接使用现有索引（EN-8/D025）");
    }
    if !skip_refresh {
        let mut lost_retries: u32 = 0;
        loop {
            match crate::bootlock::acquire_guard(repo_abs, lock_timeout, "向量索引构建")? {
                crate::bootlock::BootLockOutcome::Won(vguard) => {
                    // 索引补建失败不致命（弹性降级，E3-R1 engram CS4015 教训）：警告后尝试复用已有索引
                    match rt.block_on(vector::build_vector_index(
                        repo_abs,
                        &index_dir,
                        embed.clone(),
                        mode,
                        &cfg.index.exclude,
                    )) {
                        Ok(report) => eprintln!(
                            "# 向量索引: {} chunks（嵌入 {} 复用 {} 清理 {}）",
                            report.chunks_total, report.embedded, report.reused, report.gc_removed
                        ),
                        Err(e) => {
                            tracing::warn!(component = "vector_build", error = %e, "向量索引构建失败（降级：尝试复用已有索引）");
                            // P005 R5.2：非致命降级进审计留痕（best-effort）
                            record_degraded(boot_audit, "vector_build", &e);
                        }
                    }
                    drop(vguard); // 构建段结束即放锁（D014：锁不跨 LLM 调用、不罩检索）
                    break;
                }
                crate::bootlock::BootLockOutcome::OpponentFinished if idx.exists() => {
                    eprintln!("# 向量索引: 另一进程刚完成构建，复用其产物（D020）");
                    break;
                }
                crate::bootlock::BootLockOutcome::OpponentFinished => {
                    lost_retries += 1;
                    if lost_retries > 3 {
                        tracing::warn!(
                            component = "vector_build",
                            "对手释放引导锁但产物缺失（疑似中途崩溃），放弃复用，弹性降级为无召回层"
                        );
                        let _ = boot_audit.record(
                        "degraded",
                        &serde_json::json!({"component": "vector_build", "error": "opponent_lost_without_artifact"}),
                    );
                        break;
                    }
                    tracing::warn!(
                        component = "vector_build",
                        "对手释放引导锁但向量索引产物缺失，重新竞争引导锁（{lost_retries}/3）"
                    );
                }
            }
        }
    } // if !skip_refresh
    Ok(())
}

/// D028：向量挂载段（每会话）——开库→召回引擎→注册工具→召回 + 导航图后缀。
/// async 化：batch 会话跑在 tokio 任务内不能 block_on（单 run 由 setup 外层 block_on）。
async fn vector_attach(
    ctx: &VectorLayerCtx<'_>,
    cfg: &config::Config,
    repo_map: bool,
    embed: vector::EmbedClient,
    registry: &mut tools::ToolRegistry,
) -> CsResult<Option<String>> {
    let index_dir = vector::store::project_index_dir(ctx.repo_abs);
    let idx = vector::store::index_path(&index_dir, &vector::store::fingerprint(ctx.repo_abs));
    let store = match vector::VectorStore::open(&idx) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("向量索引不可用（本会话无召回层，任务继续）: {e}");
            return Ok(None);
        }
    };
    // P007 R3.27：召回引擎打开失败不再 `?` 上抛打穿整个任务（单行损坏即致命）——
    // 对齐函数承诺的「向量不可用 → Ok(None) 弹性降级不致命」
    let recall = match vector::RecallEngine::open(store, embed) {
        Ok(r) => Arc::new(r),
        Err(e) => {
            tracing::warn!(component = "vector_recall", error = %e, "召回引擎打开失败（本会话无召回层，任务继续）");
            record_degraded(ctx.audit, "vector_recall", &e);
            return Ok(None);
        }
    };
    registry.register(Box::new(tools::vector_search::VectorSearchTool::new(
        Arc::clone(&recall),
    )));
    // 召回先行（P003 E2 v2）：命中喂给导航图当种子；失败降级为无后缀
    let hits = match recall.recall(ctx.task, 10).await {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!(component = "recall", error = %e, "召回失败（跳过注入）");
            // P005 R5.2：非致命降级进审计留痕（best-effort）
            record_degraded(ctx.audit, "recall", &e);
            Vec::new()
        }
    };
    let mut parts: Vec<String> = Vec::new();
    if !hits.is_empty() {
        parts.push(vector::recall::RecallEngine::format_recall_block(&hits, 10));
    }
    if repo_map {
        let budget = cfg.vector.repomap_budget;
        match vector::repomap::repo_map_inputs(ctx.cg_db) {
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
                    let (callers, callees) =
                        vector::chunk::relations_for_symbol(ctx.cg_db, &c.file, &c.symbol)
                            .unwrap_or_default();
                    for n in callers.into_iter().chain(callees) {
                        neighbors.insert(n);
                    }
                    seeds.push(c.symbol.clone());
                }
                let map =
                    vector::repomap::build_task_map(&symbols, &degrees, budget, &seeds, &neighbors);
                eprintln!(
                    "# 任务导航图: {} 字符（种子 {}，邻居 {}）",
                    map.len(),
                    seeds.len(),
                    neighbors.len()
                );
                parts.push(vector::repomap::wrap_repo_section(&map));
            }
            Err(e) => {
                // P007 R3.17：五处降级留痕唯一漏的这一处补齐——与同函数其他降级一致
                tracing::warn!("导航图构建失败（跳过注入）: {e}");
                record_degraded(ctx.audit, "repo_map", &e);
            }
        }
    }
    if parts.is_empty() {
        Ok(None)
    } else {
        Ok(Some(parts.join("\n\n")))
    }
}

/// D028：图工具装配（每会话注册表）——codegraph 有符号才注册，否则地形提示（QA FINDING-009）。
fn register_graph_tools(
    cg: &Option<Arc<tools::graph::CodegraphEngine>>,
    cg_db: &Path,
    registry: &mut tools::ToolRegistry,
    parts: &mut Vec<String>,
) {
    let graph_symbols_nonempty = cg.is_some()
        && crate::vector::repomap::repo_map_inputs(cg_db)
            .map(|(s, _)| !s.is_empty())
            .unwrap_or(false);
    if let Some(cg) = cg {
        if graph_symbols_nonempty {
            registry.register(Box::new(tools::graph::ExploreTool::new(cg.clone())));
            registry.register(Box::new(tools::graph::CallersTool::new(cg.clone())));
            registry.register(Box::new(tools::graph::CalleesTool::new(cg.clone())));
            registry.register(Box::new(tools::graph::ImpactTool::new(cg.clone())));
            registry.register(Box::new(tools::graph::FilesTool::new(cg.clone())));
        } else {
            tracing::info!("# codegraph 无符号索引（非代码仓）：图工具未注册");
            parts.push(
                "〔地形提示〕本仓库无结构图索引（非代码仓或未建索引）：explore/callers/callees/impact 不可用。请用 find_files（带关键词）与 grep（带内容模式）探索，用 read 阅读具体文件。"
                    .into(),
            );
        }
    }
}

// ---- D028（EN-12）：codesleuth batch——单进程多会话，boot 一次全任务共享 ----

/// 批量任务条目（Q004 拍板：id 为产物命名元数据；契约三件套按任务声明）。
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct BatchTaskSpec {
    /// 产物命名（缺省 task-<序号>）。
    pub id: Option<String>,
    pub task: String,
    #[serde(default)]
    pub require: Vec<String>,
    #[serde(default)]
    pub require_line: Vec<String>,
    #[serde(default)]
    pub require_count: Vec<(String, usize)>,
}

/// 解析任务清单：顶层对象 {"tasks":[...]} 或裸数组均可（纯函数可测）。
fn parse_batch_spec(text: &str) -> CsResult<Vec<BatchTaskSpec>> {
    let v: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| CsError::new(USER_INPUT, format!("任务清单不是合法 JSON: {e}")))?;
    let tasks_val = if v.is_array() {
        v
    } else {
        v.get("tasks")
            .cloned()
            .ok_or_else(|| {
                CsError::new(USER_INPUT, "任务清单缺 tasks 字段")
                    .with_hint("格式：{\"tasks\":[{\"id\":\"doc-01\",\"task\":\"...\"}]}，或直接给数组")
            })?
    };
    serde_json::from_value::<Vec<BatchTaskSpec>>(tasks_val).map_err(|e| {
        CsError::new(USER_INPUT, format!("任务条目解析失败: {e}"))
            .with_hint("每条需含 task 字段；可选 id/require/require_line/require_count")
    })
}

/// 产物命名（Q004）：id 缺省 task-<序号>（从 1 起，编排层友好）。
fn batch_task_id(spec: &BatchTaskSpec, index: usize) -> String {
    spec.id
        .clone()
        .unwrap_or_else(|| format!("task-{}", index + 1))
}

/// batch 进程级共享引导（构建一次，N 会话复用）：provider/fence/fuzzy/rt/graph/向量 boot。
struct BatchBoot {
    repo_abs: PathBuf,
    state_dir: PathBuf,
    cfg: config::Config,
    api_key: String,
    provider: llm::SharedProvider,
    fence: Arc<fence::Fence>,
    fuzzy: Arc<tools::fuzzy::FuzzyEngine>,
    rt: tokio::runtime::Runtime,
    cg: Option<Arc<tools::graph::CodegraphEngine>>,
    cg_db: PathBuf,
    vector_on: bool,
    repo_map_on: bool,
    embed: vector::EmbedClient,
}

/// 单任务执行结果（summary.json 条目）。
#[derive(Debug, serde::Serialize)]
struct BatchTaskOutcome {
    id: String,
    session_id: String,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    report_md: Option<String>,
}

/// 单会话执行（async：跑在 batch 的 tokio 任务内）。产物隔离：audit/report 按各自 session 分文件。
async fn run_batch_session(
    boot: &BatchBoot,
    spec: &BatchTaskSpec,
    session_id: &str,
    id: &str,
    out_dir: &Path,
) -> BatchTaskOutcome {
    let fail = |session_id: &str, msg: String| BatchTaskOutcome {
        id: id.to_string(),
        session_id: session_id.to_string(),
        status: "error",
        error: Some(msg),
        report_md: None,
    };
    let audit = match audit::Audit::create(&boot.state_dir, session_id) {
        Ok(a) => a,
        Err(e) => return fail(session_id, format!("审计创建失败: {e}")),
    };
    let guard_before = match crate::writeguard::snapshot(&boot.repo_abs) {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::warn!(component = "writeguard", error = %e, "考前快照失败，零写入自证将不可用");
            let _ = audit.record("degraded", &serde_json::json!({"component": "writeguard", "stage": "before", "error": e.to_string()}));
            None
        }
    };
    let mut registry = tools::ToolRegistry::new();
    registry.register(Box::new(tools::read::ReadTool::new(boot.fence.clone())));
    registry.register(Box::new(tools::fuzzy::FileFinderTool::new(boot.fuzzy.clone())));
    registry.register(Box::new(tools::fuzzy::GrepTool::new(boot.fuzzy.clone())));
    let mut parts: Vec<String> = Vec::new();
    register_graph_tools(&boot.cg, &boot.cg_db, &mut registry, &mut parts);
    let ctx = VectorLayerCtx {
        repo_abs: &boot.repo_abs,
        task: &spec.task,
        api_key: &boot.api_key,
        cg_db: &boot.cg_db,
        rt: &boot.rt,
        audit: &audit,
    };
    let mut first_suffix: Option<String> = None;
    if boot.vector_on {
        match vector_attach(&ctx, &boot.cfg, boot.repo_map_on, boot.embed.clone(), &mut registry).await {
            Ok(Some(s)) => first_suffix = Some(s),
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(component = "vector_layer", error = %e, "向量层装配失败（本会话无召回层，任务继续）");
                record_degraded(&audit, "vector_layer", &e);
            }
        }
    }
    if first_suffix.is_none() && !parts.is_empty() {
        first_suffix = Some(parts.join("\n\n"));
    }
    let audit_path = audit.path().to_path_buf();
    let agent = harness::Harness::new(
        boot.provider.clone(),
        registry,
        audit,
        boot.cfg.llm.model.clone(),
        boot.cfg.context.model_context_tokens,
        boot.cfg.context.compact_at_percent,
    );
    let agent = match first_suffix {
        Some(s) => agent.with_first_user_suffix(s),
        None => agent,
    };
    let agent = {
        let a = if spec.require.is_empty() {
            agent
        } else {
            agent.with_required_markers(spec.require.clone())
        };
        let a = if spec.require_line.is_empty() {
            a
        } else {
            a.with_required_line_markers(spec.require_line.clone())
        };
        if spec.require_count.is_empty() {
            a
        } else {
            a.with_required_count_markers(spec.require_count.clone())
        }
    };
    let run_result = agent.run(&spec.task).await;
    drop(agent);
    let outcome = match run_result {
        Ok(o) => o,
        Err(e) => {
            Cli::emit_write_check(guard_before.as_ref(), &boot.repo_abs, &audit_path);
            return fail(session_id, e.to_string());
        }
    };
    Cli::emit_write_check(guard_before.as_ref(), &boot.repo_abs, &outcome.audit_path);
    // 全局 reports/ 按 session 落盘（隔离）；out_dir/<id>.md 面向编排层（Q004）
    let human = outcome.answer.clone();
    let reports_dir = boot.state_dir.join("reports");
    let _ = std::fs::create_dir_all(&reports_dir);
    let md_path = reports_dir.join(format!("{session_id}.md"));
    let _ = std::fs::write(&md_path, &human);
    let _ = std::fs::write(reports_dir.join(format!("{session_id}.json")),
        serde_json::to_string_pretty(&outcome.report).unwrap_or_default());
    let _ = std::fs::create_dir_all(out_dir);
    let id_md = out_dir.join(format!("{id}.md"));
    if let Err(e) = std::fs::write(&id_md, &human) {
        return fail(session_id, format!("批量产物写入失败: {e}"));
    }
    eprintln!(
        "# [batch/{id}] 完成：{} turns · {} tool calls · 报告 {}",
        outcome.turns,
        outcome.tool_calls,
        id_md.display()
    );
    BatchTaskOutcome {
        id: id.to_string(),
        session_id: session_id.to_string(),
        status: "ok",
        error: None,
        report_md: Some(id_md.to_string_lossy().to_string()),
    }
}

fn config_get(key: Option<String>, profile: Option<&str>) -> CsResult<String> {
    let cfg = config::load(config::CliOverrides {
        profile: profile.map(str::to_string),
        ..Default::default()
    })?;
    match key.as_deref() {
        None => toml::to_string_pretty(&config::to_file_view(&cfg))
            .map_err(|e| CsError::new(INTERNAL, format!("序列化失败: {e}"))),
        // P005 R7.2：键表驱动，与 config set 同表（context.* 等 13 键全可读）
        Some(k) => config::resolved_get(&cfg, k),
    }
}

fn config_set(key: String, value: String) -> CsResult<PathBuf> {
    let path = config::global_config_path().ok_or_else(|| {
        // P007 R3.6：hint 不再提从不参与的 XDG_CONFIG_HOME（实际只 dirs::home_dir()）
        CsError::new(CONFIG_MISSING, "找不到用户主目录")
            .with_hint("检查 HOME 环境变量或用户目录配置")
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
    // P007 R2.8：临时文件 + rename 原子写——直接 fs::write 中途崩溃会留半截 TOML，
    // 后续所有 config::load 全局报 CS1012
    let body = toml::to_string_pretty(&fc)
        .map_err(|e| CsError::new(INTERNAL, format!("序列化失败: {e}")))?;
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, &body)
        .map_err(|e| CsError::new(CONFIG_INVALID, format!("写入失败: {e}")))?;
    std::fs::rename(&tmp, &path)
        .map_err(|e| CsError::new(CONFIG_INVALID, format!("写入失败: {e}")))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// P007 R2.3/R2.4 回归：emit_write_check 两种留痕路径
    #[test]
    fn emit_write_check_records_changes_and_unavailable() {
        let repo = tempfile::tempdir().unwrap();
        let audit_dir = tempfile::tempdir().unwrap();
        std::fs::write(repo.path().join("a.txt"), "v1").unwrap();
        let before = crate::writeguard::snapshot(repo.path()).unwrap();

        // append_line 要求审计文件已存在（先落一行种子）
        let audit_path = audit_dir.path().join("s.log");
        std::fs::write(&audit_path, "{\"seq\":1}\n").unwrap();

        // ① 变更被检出并落 write_check 行
        std::fs::write(repo.path().join("a.txt"), "v2").unwrap();
        Cli::emit_write_check(Some(&before), repo.path(), &audit_path);
        let line = std::fs::read_to_string(&audit_path).unwrap();
        assert!(
            line.contains("\"unattributed_changes\":1"),
            "变更应被检出: {line}"
        );

        // ② 考前快照不可用 → available:false 而非静默
        Cli::emit_write_check(None, repo.path(), &audit_path);
        let line = std::fs::read_to_string(&audit_path).unwrap();
        assert!(
            line.contains("\"available\":false") && line.contains("guard_before_unavailable"),
            "快照不可用应诚实留痕: {line}"
        );
    }

    /// P005 R7.3：index 非 --vector 的报错必须诚实——不再声称「--rebuild 已记录」。
    #[test]
    fn index_structure_hint_is_honest() {
        // P007 R4.3：--rebuild 已删，无参分支；hint 指向唯一真实入口
        let e = index_structure_error();
        assert!(e.to_string().contains("仅支持 --vector"));
        let hint = e.hint.as_deref().unwrap();
        assert!(hint.contains("--fresh-index"));
        assert!(hint.contains("--vector"));
        assert!(!hint.contains("已记录"), "不许再撒谎: {hint}");
    }

    /// P005 R7.1：嵌入供应商缺省跟随 [llm]，显式 [vector] 覆盖优先。
    #[test]
    fn embed_endpoint_fallback_follows_llm() {
        let (b, k) = resolve_embed_endpoint(None, None, "https://llm", "llm-key");
        assert_eq!((b.as_str(), k.as_str()), ("https://llm", "llm-key"));
        let (b, k) = resolve_embed_endpoint(
            Some("https://vec".into()),
            Some("vec-key".into()),
            "https://llm",
            "llm-key",
        );
        assert_eq!((b.as_str(), k.as_str()), ("https://vec", "vec-key"));
    }

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

    /// D018：--output-format 三态解析——默认 report，raw/json 可显式选中。
    #[test]
    fn parses_output_format() {
        let cli = Cli::try_parse_from([
            "codesleuth",
            "任务",
            "--repo",
            ".",
            "--output-format",
            "raw",
        ])
        .unwrap();
        assert_eq!(cli.output_format, OutputFormat::Raw);
        let cli = Cli::try_parse_from(["codesleuth", "任务"]).unwrap();
        assert_eq!(cli.output_format, OutputFormat::Report);
    }

    #[test]
    fn parses_full_surface() {
        let cli = Cli::try_parse_from([
            "codesleuth",
            "任务",
            "--repo",
            "r",
            // P007 R4.2：--focus 已删
            "--output-format",
            "json",
            "--out",
            "o.json",
            "--model",
            "m",
            "--base-url",
            "http://x",
            "--fresh-index",
            "--stale-ok",
            "-vv",
        ])
        .unwrap();
        assert_eq!(cli.output_format, OutputFormat::Json);
        assert!(cli.fresh_index);
        // EN-8/D025：旗标可达；与 --fresh-index 同给时的优先级合并在调用点
        // （setup_vector_layer 收 stale_ok && !fresh_index），解析层不互斥
        assert!(cli.stale_ok);
        assert_eq!(cli.verbose, 2);
        assert_eq!(cli.model.as_deref(), Some("m"));
    }

    /// EN-8/D025：默认不带旗标 = false（不改变存量行为）
    #[test]
    fn stale_ok_defaults_false() {
        let cli = Cli::try_parse_from(["codesleuth", "任务"]).unwrap();
        assert!(!cli.stale_ok);
        assert!(!cli.vector);
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
        let cli = Cli::try_parse_from(["codesleuth", "index", ".", "--vector"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Index { vector: true, .. })
        ));
        // P007 R4.3：--rebuild 已删，传它应直接解析失败
        assert!(Cli::try_parse_from(["codesleuth", "index", ".", "--rebuild"]).is_err());
    }

    #[test]
    fn missing_task_is_usage_error_exit_1() {
        let cli = Cli::try_parse_from(["codesleuth"]).unwrap();
        let err = cli.run_task_inner("").unwrap_err();
        assert_eq!(err.code, USER_INPUT);
        assert_eq!(err.exit_code(), 1);
    }

    // ---- D028（EN-12）batch ----

    #[test]
    fn parses_batch_spec_array_and_object_forms() {
        // 裸数组
        let specs = parse_batch_spec(r#"[{"task":"a"},{"id":"doc-1","task":"b"}]"#
        )
        .expect("裸数组形式应可解析");
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].task, "a");
        assert_eq!(specs[1].id.as_deref(), Some("doc-1"));
        // 对象包裹
        let specs = parse_batch_spec(r#"{"tasks":[{"task":"a"}]}"#).expect("对象包裹应可解析");
        assert_eq!(specs.len(), 1);
        // 契约字段可带（require_count 元组数组）
        let specs = parse_batch_spec(
            r#"[{"task":"a","require":["m1"],"require_count":[["n",2]]}]"#,
        )
        .expect("带契约字段应可解析");
        assert_eq!(specs[0].require, vec!["m1".to_string()]);
        assert_eq!(specs[0].require_count, vec![("n".to_string(), 2)]);
    }

    #[test]
    fn batch_spec_errors_are_usage_level() {
        let e = parse_batch_spec("not json").unwrap_err();
        assert_eq!(e.code, USER_INPUT);
        assert_eq!(parse_batch_spec("{}").unwrap_err().code, USER_INPUT, "缺 tasks 字段");
        assert_eq!(
            parse_batch_spec(r#"[{"no_task":1}]"#).unwrap_err().code,
            USER_INPUT,
            "缺 task 字段"
        );
    }

    #[test]
    fn batch_task_id_defaults_to_index() {
        let s = BatchTaskSpec {
            id: None,
            task: "t".into(),
            require: vec![],
            require_line: vec![],
            require_count: vec![],
        };
        assert_eq!(batch_task_id(&s, 0), "task-1");
        assert_eq!(batch_task_id(&s, 32), "task-33");
        let named = BatchTaskSpec {
            id: Some("doc-01".into()),
            ..s
        };
        assert_eq!(batch_task_id(&named, 5), "doc-01");
    }

    /// batch 子命令解析：旗标可达 + 默认值
    #[test]
    fn parses_batch_subcommand() {
        let cli = Cli::try_parse_from([
            "codesleuth",
            "batch",
            "tasks.json",
            "--repo",
            ".",
            "--jobs",
            "3",
            "--vector",
        ])
        .unwrap();
        assert!(matches!(cli.command, Some(Command::Batch { jobs: 3, vector: true, .. })));
        let cli = Cli::try_parse_from(["codesleuth", "batch", "t.json", "--repo", "."])
            .unwrap();
        assert!(matches!(cli.command, Some(Command::Batch { jobs: 1, vector: false, .. })));
    }
}
