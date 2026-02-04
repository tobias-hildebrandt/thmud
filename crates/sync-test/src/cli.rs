use std::{fmt::Display, io::Read, str::FromStr, time::Duration};

use clap::Parser;

use crate::{config::SyncTestConfig, sim::Sim};

#[derive(Debug, Parser)]
pub struct CliConfig {
    #[arg(long, default_value_t = Self::default_wait_for_tick())]
    pub wait_for_tick: WaitForTick,
    #[command(flatten)]
    pub sync_config: SyncTestConfig,
}

impl CliConfig {
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
    pub(super) fn wait(&self) {
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

pub fn run_cli(config: CliConfig) {
    let mut sim = Sim::new(config.sync_config);

    loop {
        config.wait_for_tick.wait();

        sim.tick();
    }
}
