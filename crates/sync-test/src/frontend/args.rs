use clap::{Parser, Subcommand};

use crate::frontend::{
    simple_cli::{SimpleCliArgs, run_simple_cli},
    tui::{Tui, TuiArgs},
};

/// All program arguments.
#[derive(Debug, Parser)]
pub struct Arguments {
    /// Which frontend to use
    #[command(subcommand)]
    pub frontend: Frontend,
}

/// Which frontend to use
#[derive(Debug, Subcommand)]
pub enum Frontend {
    /// Run the simple cli.
    #[command(aliases = ["cli", "c"])]
    SimpleCli(SimpleCliArgs),
    /// Run the terminal user interface.
    #[command(aliases = ["t"])]
    Tui(TuiArgs),
}

impl Frontend {
    pub fn run(self) {
        match self {
            Frontend::SimpleCli(args) => run_simple_cli(args),
            Frontend::Tui(args) => {
                ratatui::run(|terminal| Tui::new(args).run(terminal)).unwrap();
            }
        }
    }
}
