use clap::Parser;
use sync_test::{config::SyncTestConfig, run_sync_test};

fn main() {
    tracing_subscriber::fmt().init();

    let config = SyncTestConfig::parse();

    run_sync_test(&config);
}
