use std::time::Duration;

use sync_test::{
    config::{StaticOrRandom, SyncTestConfig, WaitForTick},
    run_sync_test,
};

fn main() {
    tracing_subscriber::fmt().init();

    let config = SyncTestConfig {
        world_size: 32,
        num_sync_updates: 20,
        latency: StaticOrRandom::RandomRange(3..10),
        wait_for_tick: WaitForTick::Sleep(Duration::from_millis(10)),
        ..Default::default()
    };

    run_sync_test(&config);
}
