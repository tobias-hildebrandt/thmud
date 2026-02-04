use clap::{Parser, Subcommand};

use crate::{
    frontend::{
        simple_cli::{SimpleCliConfig, run_simple_cli},
        tui::Tui,
    },
    simulation::config::SyncTestConfig,
};

/// All program arguments.
#[derive(Debug, Parser)]
pub struct Arguments {
    /// Which frontend to use
    #[command(subcommand)]
    pub frontend: Frontend,
    #[command(flatten)]
    pub config: SyncTestConfig,
}

/// Which frontend to use
#[derive(Debug, Subcommand)]
pub enum Frontend {
    /// Run the simple cli.
    #[command(aliases = ["cli", "c"])]
    SimpleCli(SimpleCliConfig),
    /// Run the terminal user interface.
    #[command(aliases = ["t"])]
    Tui,
}

impl Frontend {
    pub fn run(self, config: SyncTestConfig) {
        match self {
            Frontend::SimpleCli(cli_config) => run_simple_cli(cli_config, config),
            Frontend::Tui => {
                ratatui::run(|terminal| Tui::new(config).run(terminal)).unwrap();
            }
        }
    }
}
