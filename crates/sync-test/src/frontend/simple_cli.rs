use std::{fmt::Display, io::Read, str::FromStr, time::Duration};

use bpaf::Bpaf;

use crate::simulation::{config::SimConfig, sim::Sim};

/// Config for the simple cli
#[derive(Debug, Clone, Bpaf)]
#[bpaf(generate(simple_cli_config_parser))]
pub struct SimpleCliConfig {
    /// When to tick the simulation
    #[bpaf(
        long("tick"),
        fallback(SimpleCliConfig::default_wait_for_tick()),
        display_fallback
    )]
    pub wait_for_tick: WaitForTick,
}

impl SimpleCliConfig {
    fn default_wait_for_tick() -> WaitForTick {
        WaitForTick::Stdin
    }
}

impl Default for SimpleCliConfig {
    fn default() -> Self {
        Self {
            wait_for_tick: Self::default_wait_for_tick(),
        }
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

/// Runs the simple CLI.
pub fn run_simple_cli(args: SimpleCliConfig, config: SimConfig) {
    let mut sim = Sim::new(config);

    loop {
        args.wait_for_tick.wait();

        sim.tick();
    }
}
