use clap::Parser;
use codesleuth::cli::Cli;
use codesleuth::init_tracing;

fn main() {
    let cli = Cli::parse();
    // P005 R7.4：session id 在入口生成——日志串线与审计会话同一身份
    let session_id = codesleuth::audit::new_session_id();
    init_tracing(cli.verbose, &session_id);
    let code = cli.run(&session_id);
    std::process::exit(code);
}
