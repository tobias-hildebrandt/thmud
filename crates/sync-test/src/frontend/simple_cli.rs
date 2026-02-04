use std::{fmt::Display, io::Read, str::FromStr, time::Duration};

use clap::Parser;

use crate::simulation::{config::SyncTestConfig, sim::Sim};

/// Config for the simple cli.
#[derive(Debug, Parser)]
pub struct SimpleCliArgs {
    /// How the cli should wait for the next tick.
    #[arg(long, default_value_t = Self::default_wait_for_tick())]
    pub wait_for_tick: WaitForTick,
    #[command(flatten)]
    pub config: SyncTestConfig,
}

impl SimpleCliArgs {
    fn default_wait_for_tick() -> WaitForTick {
        WaitForTick::Stdin
    }
}

#[derive(Debug, Clone)]

pub enum WaitForTick {
    Sleep(Duration),
    Stdin,
}

impl FromStr for WaitForTick {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == "stdin" {
            return Ok(Self::Stdin);
        }
        Ok(Self::Sleep(Duration::from_millis(s.parse()?)))
    }
}

impl Display for WaitForTick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaitForTick::Sleep(duration) => write!(f, "{}ms", duration.as_millis()),
            WaitForTick::Stdin => write!(f, "stdin"),
        }
    }
}

impl WaitForTick {
    pub(crate) fn wait(&self) {
        match self {
            WaitForTick::Sleep(duration) => std::thread::sleep(*duration),
            WaitForTick::Stdin => {
                let mut read_char = [0u8];
                let mut read = std::io::stdin().lock();
                loop {
                    let bytes_read = read
                        .read(&mut read_char)
                        .expect("unable to read from stdin");
                    if bytes_read == 0 {
                        continue;
                    }
                    break;
                }
            }
        }
    }
}

pub fn run_simple_cli(cli_config: SimpleCliArgs) {
    let mut sim = Sim::new(cli_config.config);

    loop {
        cli_config.wait_for_tick.wait();

        sim.tick();
    }
}
