use clap::Parser;
use sync_test::{cli::CliConfig, run_cli};

fn main() {
    tracing_subscriber::fmt().init();

    let config = CliConfig::parse();

    run_cli(config);
}
