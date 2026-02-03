mod client;
pub mod config;
mod messages;
mod run;
mod server;
mod world;

pub use run::run_sync_test;

/// Time in ticks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Tick(pub(crate) u128);

fn square(float: f32) -> f32 {
    float * float
}
