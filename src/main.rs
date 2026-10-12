use clap::Parser;
use codesleuth::cli::{Cli, OutputFormat};
use codesleuth::init_tracing;

fn main() {
    let cli = Cli::parse();
    // P005 R7.4：session id 在入口生成——日志串线与审计会话同一身份
    let session_id = codesleuth::audit::new_session_id();
    // D026（EN-10）：json 输出模式关 stderr ANSI——错误单行 JSON 通道不被转义码污染
    let ansi = !matches!(cli.output_format, OutputFormat::Json);
    init_tracing(cli.verbose, &session_id, ansi);
    let code = cli.run(&session_id);
    std::process::exit(code);
}
