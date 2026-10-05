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
pub mod mcp;
pub mod prompt;
pub mod report;
pub mod tools;
pub mod vector;
pub mod writeguard;

/// 按 -v 计数初始化日志（0=warn, 1=info, 2+=debug）。
pub fn init_tracing(verbose: u8) {
    use tracing_subscriber::EnvFilter;
    let level = match verbose {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };
    let filter = EnvFilter::try_new(format!("codesleuth={level}"))
        .unwrap_or_else(|_| EnvFilter::new("warn"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        // stdout 纪律（E3 消歧验证发现）：--json 模式下 stdout 只准有报告 JSON，日志一律走 stderr
        .with_writer(std::io::stderr)
        .init();
}
