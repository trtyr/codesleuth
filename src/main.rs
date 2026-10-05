use clap::Parser;
use codesleuth::cli::Cli;
use codesleuth::init_tracing;

fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    let code = cli.run();
    std::process::exit(code);
}
