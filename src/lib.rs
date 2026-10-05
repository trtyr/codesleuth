//! codesleuth —— 只读代码侦察 Agent harness。
//!
//! 模块地图与权威方案原文在作者本地的 docs/plantree/（规划态，.gitignore 排除，不随仓库分发）；
//! 公开仓的事实权威 = README + 本仓库代码与注释。

pub mod audit;
pub mod bootlock;
pub mod cli;
pub mod config;
pub mod context;
pub mod errors;
pub mod evidence;
pub mod fence;
pub mod harness;
pub mod llm;
pub mod logs;
pub mod mcp;
pub mod prompt;
pub mod report;
pub mod tools;
pub mod vector;
pub mod writeguard;

/// 初始化日志（P005 R7.4）：stderr 人读 + `~/.codesleuth/logs/<session>.log` 归档双轨，
/// 同过滤同格式；事件在 session span 内自动携带 session_id（结构化串线）。
/// stdout 纪律（E3 消歧验证发现）：--json 模式下 stdout 只准有报告 JSON，日志一律 stderr + 文件。
pub fn init_tracing(verbose: u8, session_id: &str) {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let level = match verbose {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };
    let filter = EnvFilter::try_new(format!("codesleuth={level}"))
        .unwrap_or_else(|_| EnvFilter::new("warn"));

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_writer(std::io::stderr);

    // 文件归档层：打开失败不阻断（仅 stderr 降级，警告可见）
    let file_layer = crate::config::global_state_dir().and_then(|dir| {
        let path = dir.join("logs").join(format!("{session_id}.log"));
        match logs::SessionLog::open(&path, logs::LOG_ROTATE_BYTES) {
            Ok(w) => Some(
                tracing_subscriber::fmt::layer()
                    .with_target(false)
                    .with_ansi(false)
                    .with_writer(w),
            ),
            Err(e) => {
                eprintln!("warn: 会话日志文件打开失败 {path:?}（仅 stderr）: {e}");
                None
            }
        }
    });

    // 过滤器挂在 registry 层，stderr / 文件双轨共用
    tracing_subscriber::registry()
        .with(filter)
        .with(stderr_layer)
        .with(file_layer)
        .init();
}
