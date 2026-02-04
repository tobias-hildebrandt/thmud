pub mod cli;
mod client;
pub mod config;
mod messages;
mod server;
mod sim;
pub mod tui;
mod world;

pub use cli::run_cli;

/// Time in ticks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Tick(pub(crate) u128);

fn square(float: f32) -> f32 {
    float * float
}
